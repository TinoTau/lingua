use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use futures::future::join_all;
use serde_json::json;

use crate::asr_streaming::AsrResult;
use crate::asr_whisper::{WhisperAsrStreaming, FasterWhisperAsrStreaming};
use crate::asr_streaming::AsrStreamingExt;
use crate::asr_filters::is_meaningless_transcript as is_meaningless_transcript_filter;
use crate::audio_buffer::merge_frames;
use crate::emotion_adapter::{EmotionRequest, EmotionResponse};
use crate::error::{EngineError, EngineResult};
use crate::event_bus::{CoreEvent, EventTopic};
use crate::nmt_incremental::{TranslationRequest, TranslationResponse};
use crate::persona_adapter::PersonaContext;
use crate::telemetry::TelemetryDatum;
use crate::tts_streaming::{TtsRequest, TtsStreamChunk};
use crate::types::{PartialTranscript, StableTranscript};
use crate::health_check::HealthChecker;
use crate::performance_logger::PerformanceLog;
use crate::vad::VadFeedbackType;


use super::core::CoreEngine;
use super::process_result::ProcessResult;

impl CoreEngine {
    // boot() 和 shutdown() 方法已移至 lifecycle.rs 模块
    // 以下代码已删除，请使用 lifecycle 模块中的实现
    /*
    pub async fn boot(&self) -> EngineResult<()> {
        self.event_bus.start().await?;
        let config = self.config.load().await?;
        self.cache.warm_up().await?;
        self.asr.initialize().await?;
        self.nmt.initialize().await?;
        
        // 健康检查：检查 NMT 和 TTS 服务（带重试机制，等待服务就绪）
        if let (Some(nmt_url), Some(tts_url)) = (&self.nmt_service_url, &self.tts_service_url) {
            let checker = HealthChecker::new();
            
            // 等待服务就绪，最多重试 15 次，每次间隔 1 秒（总共最多 15 秒）
            const MAX_RETRIES: u32 = 15;
            const RETRY_DELAY_MS: u64 = 1000;
            
            let mut nmt_healthy = false;
            let mut tts_healthy = false;
            let mut final_attempt = 0;
            
            eprintln!("[INFO] Waiting for NMT and TTS services to be ready...");
            
            for attempt in 1..=MAX_RETRIES {
                final_attempt = attempt;
                let (nmt_health, tts_health) = checker.check_all_services(nmt_url, tts_url).await;
                
                nmt_healthy = nmt_health.is_healthy;
                tts_healthy = tts_health.is_healthy;
                
                if nmt_healthy && tts_healthy {
                    // 所有服务都健康，退出重试循环
                    break;
                }
                
                if attempt < MAX_RETRIES {
                    // 等待后重试（不打印中间结果，避免日志混乱）
                    tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
                }
            }
            
            // 报告最终状态
            if nmt_healthy {
                eprintln!("[INFO] NMT service health check passed: {} (attempt {}/{})", nmt_url, final_attempt, MAX_RETRIES);
            } else {
                eprintln!("[WARN] NMT service is not healthy after {} attempts: {} - Please ensure the service is running", final_attempt, nmt_url);
                // 不阻止启动，但记录警告
            }
            
            if tts_healthy {
                eprintln!("[INFO] TTS service health check passed: {} (attempt {}/{})", tts_url, final_attempt, MAX_RETRIES);
            } else {
                eprintln!("[WARN] TTS service is not healthy after {} attempts: {} - Please ensure the service is running", final_attempt, tts_url);
                // 不阻止启动，但记录警告
            }
        }
        
        self.telemetry
            .record(TelemetryDatum {
                name: "core_engine.boot".to_string(),
                value: 1.0,
                unit: "count".to_string(),
            })
            .await?;
        self.telemetry
            .record(TelemetryDatum {
                name: format!("core_engine.mode.{}", config.mode),
                value: 1.0,
                unit: "count".to_string(),
            })
            .await?;
        Ok(())
    }
    */

    /// 处理音频帧（完整业务流程：VAD → ASR → NMT → 事件发布）
    /// 
    /// 流程：
    /// 1. 通过 VAD 检测语音活动
    /// 2. 如果检测到语音，累积到 ASR 缓冲区
    /// 3. 如果检测到语音边界（is_boundary），触发 ASR 推理
    /// 4. 如果 ASR 返回最终结果，自动触发 NMT 翻译
    /// 5. 发布事件到 EventBus（ASR 部分结果、ASR 最终结果、翻译结果）
    /// 
    /// # Arguments
    /// * `frame` - 音频帧
    /// * `language_hint` - 语言提示（可选）
    /// 
    /// # Returns
    /// 返回处理结果（包含 ASR 和 NMT 结果）
    pub async fn process_audio_frame(
        &self,
        frame: crate::types::AudioFrame,
        language_hint: Option<String>,
    ) -> EngineResult<Option<ProcessResult>> {
        // 如果启用了连续模式，使用连续处理逻辑
        if self.continuous_mode {
            return self.process_audio_frame_continuous(frame, language_hint).await;
        }
        
        // 原有的处理逻辑（非连续模式）
        // 性能日志：记录总耗时
        let total_start = Instant::now();
        let request_id = Uuid::new_v4().to_string();
        
        // 1. 通过 VAD 检测语音活动
        let vad_result = self.vad.detect(frame).await?;

        // 2. 累积音频帧到 ASR 缓冲区
        // 使用 AsrStreamingExt trait 来统一处理不同的 ASR 实现
        // 尝试将 ASR 转换为支持扩展方法的类型
        let asr_ptr = Arc::as_ptr(&self.asr);
        
        // 尝试转换为 WhisperAsrStreaming
        let whisper_asr_ptr = asr_ptr as *const WhisperAsrStreaming;
        let faster_whisper_ptr = asr_ptr as *const crate::asr_whisper::FasterWhisperAsrStreaming;
        
        // 检查是否支持扩展方法（FasterWhisperAsrStreaming 或 WhisperAsrStreaming）
        let supports_ext = unsafe {
            let faster_whisper_ref = faster_whisper_ptr.as_ref();
            let whisper_asr_ref = whisper_asr_ptr.as_ref();
            faster_whisper_ref.is_some() || whisper_asr_ref.is_some()
        };
        
        if supports_ext {
        unsafe {
            // 优先尝试 FasterWhisperAsrStreaming
            let faster_whisper_ref = faster_whisper_ptr.as_ref();
            let whisper_asr_ref = whisper_asr_ptr.as_ref();
            
            if let Some(asr_ext) = faster_whisper_ref {
                // 使用 FasterWhisperAsrStreaming
                // 2.1. 如果提供了语言提示，设置 ASR 语言
                static LAST_LANGUAGE: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
                if let Some(ref lang_hint) = language_hint {
                    let normalized_lang = if lang_hint.starts_with("zh") {
                        Some("zh".to_string())
                    } else if lang_hint.starts_with("en") {
                        Some("en".to_string())
                    } else {
                        Some(lang_hint.clone())
                    };
                    
                    let mut last_lang = LAST_LANGUAGE.lock().unwrap();
                    let should_set = last_lang.as_ref() != normalized_lang.as_ref();
                    if should_set {
                        if let Err(e) = asr_ext.set_language(normalized_lang.clone()) {
                            eprintln!("[ASR] Warning: Failed to set language: {}", e);
                        } else {
                            *last_lang = normalized_lang;
                        }
                    }
                }
                
                asr_ext.accumulate_frame(vad_result.frame.clone())?;
            } else if let Some(whisper_asr) = whisper_asr_ref {
                // 使用 WhisperAsrStreaming（向后兼容）
                // 2.1. 如果提供了语言提示，设置 ASR 语言
                static LAST_LANGUAGE: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
                if let Some(ref lang_hint) = language_hint {
                    // 将语言代码标准化（例如 "zh-CN" -> "zh"）
                    let normalized_lang = if lang_hint.starts_with("zh") {
                        Some("zh".to_string())
                    } else if lang_hint.starts_with("en") {
                        Some("en".to_string())
                    } else {
                        Some(lang_hint.clone())
                    };
                    
                    // 检查语言是否改变
                    let mut last_lang = LAST_LANGUAGE.lock().unwrap();
                    let should_set = last_lang.as_ref() != normalized_lang.as_ref();
                    if should_set {
                        if let Err(e) = whisper_asr.set_language(normalized_lang.clone()) {
                        eprintln!("[ASR] Warning: Failed to set language: {}", e);
                        } else {
                            *last_lang = normalized_lang;
                        }
                    }
                }
                
                // 累积帧
                whisper_asr.accumulate_frame(vad_result.frame.clone())?;
            }
                
                // 3. 如果检测到语音边界，触发 ASR 推理（返回最终结果）
            // 注意：边界检测应该在静音达到阈值时立即触发，不应该有延迟
            // 如果用户每个短句之间都停了1秒，VAD应该能检测到边界
                if vad_result.is_boundary {
                // 使用统一的扩展方法获取缓冲区大小
                let buffer_size = if let Some(asr_ext) = faster_whisper_ref {
                    asr_ext.get_accumulated_frames().map(|f| f.len()).unwrap_or(0)
                } else if let Some(whisper_asr) = whisper_asr_ref {
                    whisper_asr.get_accumulated_frames().map(|f| f.len()).unwrap_or(0)
                } else {
                    0
                };
                eprintln!("[ASR] 🎯 Boundary detected at {}ms, will process {} accumulated frames", 
                         vad_result.frame.timestamp_ms, buffer_size);
                    // 3.1. 识别说话者（如果启用了说话者识别）
                    // 在非连续模式下，从 ASR 缓冲区获取累积的音频片段
                    let (speaker_result, speaker_embedding_ms) = if let Some(ref identifier) = self.speaker_identifier {
                        let speaker_start = Instant::now();
                        eprintln!("[SPEAKER] ===== Speaker Identification Started =====");
                        eprintln!("[SPEAKER] Boundary detected at timestamp: {}ms (confidence: {:.3})", 
                                 vad_result.frame.timestamp_ms, vad_result.confidence);
                        
                        // 从 ASR 缓冲区获取累积的音频帧（用于说话者识别）
                        // 过滤掉静音帧，只使用包含语音的帧
                        let all_frames = if let Some(asr_ext) = faster_whisper_ref {
                            asr_ext.get_accumulated_frames()
                        } else if let Some(whisper_asr) = whisper_asr_ref {
                            whisper_asr.get_accumulated_frames()
                        } else {
                            Ok(vec![vad_result.frame.clone()])
                        }
                            .unwrap_or_else(|e| {
                                eprintln!("[SPEAKER] ⚠ Warning: Failed to get accumulated frames: {}, using current frame only", e);
                                vec![vad_result.frame.clone()]
                            });
                        
                        // 尝试从 VAD 获取上一个语音帧的时间戳，用于过滤静音帧
                        let last_speech_ts = {
                            let vad_ptr = Arc::as_ptr(&self.vad);
                            let silero_vad_ptr = vad_ptr as *const crate::vad::SileroVad;
                            unsafe {
                                if let Some(silero_vad) = silero_vad_ptr.as_ref() {
                                    silero_vad.get_last_speech_timestamp()
                                } else {
                                    None
                                }
                            }
                        };
                        
                        // 过滤音频帧：只保留包含语音的帧（在最后一个语音帧之前的帧）
                        // 如果无法确定，则使用所有帧（除了明显的静音帧）
                        let audio_frames: Vec<_> = if let Some(speech_ts) = last_speech_ts {
                            // 只使用最后一个语音帧之前的帧（排除静音帧）
                            all_frames.iter()
                                .filter(|f| f.timestamp_ms <= speech_ts)
                                .cloned()
                                .collect()
                        } else {
                            // 如果无法确定，使用所有帧（但排除当前边界帧，因为它可能是静音）
                            // 保留除了最后一个边界帧之外的所有帧
                            if all_frames.len() > 1 {
                                all_frames[..all_frames.len() - 1].to_vec()
                            } else {
                                all_frames.clone()
                            }
                        };
                        
                        // 计算过滤后的音频时长
                        let filtered_duration_ms = if !audio_frames.is_empty() {
                            let total_samples: usize = audio_frames.iter().map(|f| f.data.len()).sum();
                            let sample_rate = audio_frames[0].sample_rate;
                            (total_samples as f32 / sample_rate as f32 * 1000.0) as u64
                        } else {
                            0
                        };
                        
                        // 如果过滤后的音频太短（< 1000ms），使用所有帧（包括静音帧）
                        // 这样可以确保说话者识别能获取到足够长的音频
                        let final_audio_frames = if filtered_duration_ms < 1000 && !all_frames.is_empty() {
                            eprintln!("[SPEAKER] ⚠ Filtered audio too short ({}ms < 1000ms), using all frames (including silence) to ensure sufficient length", filtered_duration_ms);
                            // 使用所有帧，但排除最后一个边界帧（因为它可能是纯静音）
                            if all_frames.len() > 1 {
                                all_frames[..all_frames.len() - 1].to_vec()
                            } else {
                                all_frames.clone()
                            }
                        } else if !audio_frames.is_empty() {
                            audio_frames
                        } else {
                            eprintln!("[SPEAKER] ⚠ Warning: No speech frames found after filtering, using all frames as fallback");
                            all_frames.clone()
                        };
                        
                        // 计算输入音频的总时长
                        if !final_audio_frames.is_empty() {
                            let total_samples: usize = final_audio_frames.iter().map(|f| f.data.len()).sum();
                            let sample_rate = final_audio_frames[0].sample_rate;
                            let total_duration_ms = (total_samples as f32 / sample_rate as f32 * 1000.0) as u64;
                            let total_duration_sec = total_samples as f32 / sample_rate as f32;
                            eprintln!("[SPEAKER] Input audio: {} frames (filtered from {} total), {} samples, {:.2}s ({:.0}ms) at {}Hz", 
                                     final_audio_frames.len(), all_frames.len(), total_samples, total_duration_sec, total_duration_ms, sample_rate);
                            
                            // 如果过滤后的音频仍然太短，记录警告
                            if total_duration_ms < 1000 {
                                eprintln!("[SPEAKER] ⚠ Warning: Filtered audio is still too short ({}ms < 1000ms required for speaker embedding). History buffer may need more time to accumulate.", total_duration_ms);
                            }
                        } else {
                            eprintln!("[SPEAKER] ⚠ Warning: No frames available for speaker identification");
                        }
                        
                        let audio_frames = final_audio_frames;
                        
                        let result = identifier.identify_speaker(&audio_frames, vad_result.frame.timestamp_ms).await;
                        let speaker_ms = speaker_start.elapsed().as_millis() as u64;
                        
                        match result {
                            Ok(speaker_result) => {
                                eprintln!("[SPEAKER] ✅ Identified speaker: {} (is_new: {}, confidence: {:.2})", 
                                    speaker_result.speaker_id, speaker_result.is_new_speaker, speaker_result.confidence);
                                eprintln!("[SPEAKER] Voice embedding: {} (dim: {})", 
                                    if speaker_result.voice_embedding.is_some() { "Yes" } else { "No" },
                                    speaker_result.voice_embedding.as_ref().map(|v| v.len()).unwrap_or(0));
                                eprintln!("[SPEAKER] Reference audio: {} (samples: {})", 
                                    if speaker_result.reference_audio.is_some() { "Yes" } else { "No" },
                                    speaker_result.reference_audio.as_ref().map(|a| a.len()).unwrap_or(0));
                                
                                // 注意：不再需要设置当前说话者ID，因为使用全局语速历史
                                eprintln!("[SPEAKER] ⏱️  Speaker identification completed in {}ms", speaker_ms);
                                eprintln!("[SPEAKER] ==============================================");
                                (Some(speaker_result), Some(speaker_ms))
                            }
                            Err(e) => {
                                eprintln!("[SPEAKER] ❌ Identification failed after {}ms: {}", speaker_ms, e);
                                eprintln!("[SPEAKER] ==============================================");
                                (None, Some(speaker_ms))
                            }
                        }
                    } else {
                        (None, None)
                    };
                    
                    let speaker_id = speaker_result.as_ref().map(|r| r.speaker_id.clone());
                    let voice_embedding = speaker_result.as_ref().and_then(|r| r.voice_embedding.clone());
                    let reference_audio = speaker_result.as_ref().and_then(|r| r.reference_audio.clone());
                    let is_new_speaker = speaker_result.as_ref().map(|r| r.is_new_speaker).unwrap_or(false);
                    let estimated_gender = speaker_result.as_ref().and_then(|r| r.estimated_gender.clone());
                    
                    // 如果是新说话者，或者参考音频足够长（合并后的），异步注册/更新到 YourTTS 服务（不阻塞主流程）
                    let should_register = is_new_speaker || (reference_audio.as_ref().map(|a| a.len() >= 160000).unwrap_or(false));  // 10秒 @ 16kHz
                    if should_register {
                        if let (Some(sid), Some(ref_audio)) = (speaker_id.clone(), reference_audio.clone()) {
                            let is_update = !is_new_speaker;
                            let action = if is_update { "Updating" } else { "Registering new" };
                            let sid_clone = sid.clone();
                            let ref_audio_clone = ref_audio.clone();
                            let voice_embedding_clone = voice_embedding.clone();
                            let tts_endpoint = self.tts_service_url.clone();
                            
                            // 检查是否使用 YourTTS 服务
                            let use_yourtts = tts_endpoint.as_ref()
                                .map(|url| url.contains("5004") || url.contains("yourtts"))
                                .unwrap_or(false);
                            
                            if use_yourtts {
                                if is_update {
                                    eprintln!("[SPEAKER] 🔄 Updating speaker '{}' in YourTTS service with merged reference audio ({} samples, {:.2}s, async, non-blocking)...", 
                                             sid_clone, ref_audio_clone.len(), ref_audio_clone.len() as f32 / 16000.0);
                                } else {
                                    eprintln!("[SPEAKER] 🚀 Registering new speaker '{}' to YourTTS service (async, non-blocking)...", sid_clone);
                                }
                                
                                // 异步任务：注册 speaker（不阻塞主流程）
                                tokio::spawn(async move {
                                    // 创建新的 YourTTS 客户端用于注册
                                    use crate::tts_streaming::yourtts_http::{YourTtsHttp, YourTtsHttpConfig};
                                    
                                    let endpoint = tts_endpoint.unwrap_or_else(|| "http://127.0.0.1:5004".to_string());
                                    let config = YourTtsHttpConfig {
                                        endpoint: endpoint.clone(),
                                        timeout_ms: 30000,  // 30秒超时（注册可能较慢）
                                    };
                                    
                                    match YourTtsHttp::new(config) {
                                        Ok(client) => {
                                            if let Err(e) = client.register_speaker(
                                                sid_clone.clone(),
                                                ref_audio_clone,
                                                16000,  // 参考音频采样率（从 ASR/VAD 来的）
                                                voice_embedding_clone,
                                            ).await {
                                                eprintln!("[SPEAKER] ⚠️  Failed to register speaker '{}' to YourTTS (async, non-blocking): {}", sid_clone, e);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("[SPEAKER] ⚠️  Failed to create YourTTS client for registration: {}", e);
                                        }
                                    }
                                });
                            }
                        }
                    }
                    
                    // 立即开始 ASR 转录（使用已累积的音频）
                    // 注意：在检测到边界后立即开始处理，不等待后续音频输入
                    // 这样可以实现流式处理：用户说完话后立即开始翻译，无需等待完整音频
                    // 对于手机端 AEC（声学回响消除）场景，这可以显著减少延迟
                    
                    // ⚠️ 重要：在调用 infer_on_boundary() 之前先获取音频帧信息
                    // 因为 infer_on_boundary() 会在推理完成后清空缓冲区
                    let audio_frames_for_speech_rate = if let Some(asr_ext) = faster_whisper_ref {
                        asr_ext.get_accumulated_frames()
                    } else if let Some(whisper_asr) = whisper_asr_ref {
                        whisper_asr.get_accumulated_frames()
                    } else {
                        Ok(vec![])
                    }
                        .unwrap_or_else(|e| {
                            eprintln!("[CoreEngine] ⚠️  get_accumulated_frames failed (before inference): {:?}", e);
                            vec![]
                        });
                    let audio_duration_ms_for_speech_rate = if !audio_frames_for_speech_rate.is_empty() {
                        let total_samples: usize = audio_frames_for_speech_rate.iter().map(|f| f.data.len()).sum();
                        let sample_rate = audio_frames_for_speech_rate[0].sample_rate;
                        Some((total_samples as f32 / sample_rate as f32 * 1000.0) as u64)
                    } else {
                        None
                    };
                    
                    let asr_start = Instant::now();
                    eprintln!("[ASR] 🚀 Starting transcription immediately after boundary detection...");
                    // 使用统一的扩展方法进行推理
                    let asr_result = if let Some(asr_ext) = faster_whisper_ref {
                        asr_ext.infer_on_boundary().await?
                    } else if let Some(whisper_asr) = whisper_asr_ref {
                        whisper_asr.infer_on_boundary().await?
                    } else {
                        return Err(EngineError::new("ASR implementation does not support boundary inference"));
                    };
                    let asr_ms = asr_start.elapsed().as_millis() as u64;
                    eprintln!("[ASR] ✅ Transcription completed in {}ms", asr_ms);
                    
                    // 打印 ASR 结果
                    if let Some(ref partial) = asr_result.partial {
                        eprintln!("[ASR] 📝 Partial transcript: \"{}\" (confidence: {:.2})", partial.text, partial.confidence);
                    }
                    if let Some(ref final_transcript) = asr_result.final_transcript {
                        eprintln!("[ASR] ✅ Final transcript: \"{}\" (language: {}, speaker_id: {:?})", 
                                 final_transcript.text, final_transcript.language, final_transcript.speaker_id);
                    }
                    
                    // 3.5. 过滤无意义的 ASR 结果（在进入翻译/TTS 之前）
                    if let Some(ref final_transcript) = asr_result.final_transcript {
                        if is_meaningless_transcript_filter(&final_transcript.text) {
                            eprintln!("[ASR] ⛔ Filtered meaningless transcript: \"{}\" (skipping translation/TTS)", final_transcript.text);
                            // 直接返回，不进入后续处理
                            return Ok(Some(ProcessResult {
                                asr: asr_result,
                                emotion: None,
                                translation: None,
                                tts: None,
                            }));
                        }
                    }
                    
                    // 4. 发布 ASR 最终结果事件（包含 speaker_id）
                    // 4.1. 更新语速（如果启用了自适应VAD）
                    // 注意：不区分说话者，每个短句都根据上一个短句的语速调整
                    if let Some(ref final_transcript) = asr_result.final_transcript {
                        eprintln!("[CoreEngine] 🔍 Attempting to update speech rate: final_transcript exists, text='{}'", 
                                 final_transcript.text.chars().take(30).collect::<String>());
                        
                        if let Some(audio_duration_ms) = audio_duration_ms_for_speech_rate {
                            let char_count = final_transcript.text.chars().count();
                            let speech_rate = if audio_duration_ms > 0 {
                                (char_count as f32 * 1000.0) / (audio_duration_ms as f32)
                            } else {
                                0.0
                            };
                            
                            eprintln!("[CoreEngine] 📊 Calculating speech rate: text='{}' ({} chars), audio={}ms ({} frames, {} samples), rate={:.2} chars/s", 
                                     final_transcript.text.chars().take(30).collect::<String>(), 
                                     char_count, 
                                     audio_duration_ms, 
                                     audio_frames_for_speech_rate.len(), 
                                     audio_frames_for_speech_rate.iter().map(|f| f.data.len()).sum::<usize>(),
                                     speech_rate);
                            
                            // 更新VAD中的全局语速
                            Self::update_vad_speech_rate(self, &final_transcript.text, audio_duration_ms);
                            eprintln!("[CoreEngine] ✅ Speech rate updated successfully");
                        } else {
                            eprintln!("[CoreEngine] ⚠️  Cannot update speech rate: audio_frames is empty (captured before inference)");
                        }
                    } else {
                        eprintln!("[CoreEngine] ⚠️  Cannot update speech rate: final_transcript is None (ASR returned empty result)");
                        if let Some(audio_duration_ms) = audio_duration_ms_for_speech_rate {
                            eprintln!("[CoreEngine] 📊 Audio duration was {}ms ({} frames), but no transcript to calculate speech rate", 
                                     audio_duration_ms, 
                                     audio_frames_for_speech_rate.len());
                        }
                    }
                    
                    // 更新 final_transcript 的 speaker_id 并发布事件
                    if let Some(ref final_transcript) = asr_result.final_transcript {
                        let mut transcript_with_speaker = final_transcript.clone();
                        transcript_with_speaker.speaker_id = speaker_id.clone();
                        Self::publish_asr_final_event(self, &transcript_with_speaker, vad_result.frame.timestamp_ms).await?;
                    }
                    
                    // 5. 如果 ASR 返回最终结果，进行 Emotion 分析、Persona 个性化，然后触发 NMT 翻译
                    let (emotion_result, translation_result, tts_result, _nmt_ms, _tts_ms) = if let Some(ref final_transcript) = asr_result.final_transcript {
                        // 5.1. Emotion 情感分析
                        let emotion_result = self.analyze_emotion(final_transcript, vad_result.frame.timestamp_ms).await.ok();
                        
                        // 5.2. 应用 Persona 个性化
                        let personalized_transcript = self.personalize_transcript(final_transcript).await?;
                        
                        // 5.3. 立即开始翻译（流式处理：ASR 完成后立即翻译，不等待）
                    // 优化：如果ASR识别出多个短句，按句子边界分割，逐句翻译和TTS，实现增量处理
                    // 这样可以实现：用户说一句 → 立即翻译 → 立即TTS → 立即听到，而不是等待所有句子说完
                        let mut personalized_with_speaker = personalized_transcript;
                        personalized_with_speaker.speaker_id = speaker_id.clone();
                        // 计算原始音频时长（用于后续计算每个 segment 的语速）
                        let source_audio_duration_ms = if let Some(ref _final_transcript) = asr_result.final_transcript {
                        let audio_frames = if let Some(asr_ext) = faster_whisper_ref {
                            asr_ext.get_accumulated_frames().unwrap_or_else(|_| vec![])
                        } else if let Some(whisper_asr) = whisper_asr_ref {
                            whisper_asr.get_accumulated_frames().unwrap_or_else(|_| vec![])
                        } else {
                            vec![]
                        };
                            if !audio_frames.is_empty() {
                                let total_samples: usize = audio_frames.iter().map(|f| f.data.len()).sum();
                                let sample_rate = audio_frames[0].sample_rate;
                                Some((total_samples as f32 / sample_rate as f32 * 1000.0) as u64)
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                    
                    // 按句子边界分割文本（支持中英文标点）
                    // 注意：ASR可能已经识别出多个segments，但合并成了一个文本
                    // 我们使用智能分割来恢复原始segments
                    let sentences = Self::split_into_sentences(&personalized_with_speaker.text);
                    eprintln!("[NMT] 📝 Split ASR text into {} sentences for incremental translation", sentences.len());
                    for (i, sentence) in sentences.iter().enumerate() {
                        eprintln!("[NMT]   Sentence {}: '{}'", i + 1, sentence);
                    }
                    
                    // 如果分割后只有一个句子，但原始文本很长（>30个字符），尝试更细粒度的分割
                    let sentences = if sentences.len() == 1 && personalized_with_speaker.text.chars().count() > 30 {
                        eprintln!("[NMT] ⚠️  Long text with only one sentence detected, trying finer-grained splitting...");
                        Self::split_into_sentences_fine_grained(&personalized_with_speaker.text)
                    } else {
                        sentences
                    };
                    
                    if sentences.len() > 1 {
                        eprintln!("[NMT] 📝 After fine-grained splitting: {} sentences", sentences.len());
                        for (i, sentence) in sentences.iter().enumerate() {
                            eprintln!("[NMT]   Sentence {}: '{}'", i + 1, sentence);
                        }
                    }
                        
                        let nmt_start = Instant::now();
                    eprintln!("[NMT] 🚀 Starting incremental translation for {} sentences...", sentences.len());
                    
                    // 如果只有一个句子，使用原有逻辑
                    // 如果有多个句子，逐句翻译和TTS，实现增量处理
                    let (translation_result, tts_result, nmt_ms, tts_ms, yourtts_ms) = if sentences.len() == 1 {
                        // 单句模式：原有逻辑
                        let mut translation_result = self.translate_and_publish(&personalized_with_speaker, vad_result.frame.timestamp_ms).await.ok();
                        
                        // 将原始音频信息添加到翻译结果中
                        if let Some(ref mut translation) = translation_result {
                            if let Some(ref final_transcript) = asr_result.final_transcript {
                                translation.source_audio_duration_ms = source_audio_duration_ms;
                                translation.source_text = Some(final_transcript.text.clone());
                                translation.source_language = Some(final_transcript.language.clone());
                            }
                        }
                        
                        let nmt_ms = nmt_start.elapsed().as_millis() as u64;
                        eprintln!("[NMT] ✅ Translation completed in {}ms", nmt_ms);
                        
                        // 基于ASR/NMT反馈调整VAD阈值
                        if let Some(ref final_transcript) = asr_result.final_transcript {
                            let translation_stable: Option<StableTranscript> = translation_result.as_ref().map(|t| StableTranscript {
                                text: t.translated_text.clone(),
                                speaker_id: t.speaker_id.clone(),
                                language: final_transcript.language.clone(), // 使用ASR检测到的目标语言
                            });
                            Self::adjust_vad_threshold_by_feedback(
                                self,
                                &asr_result,
                                translation_stable.as_ref(),
                                translation_result.as_ref(), // 传递完整的 TranslationResponse 以获取质量指标
                                vad_result.frame.timestamp_ms,
                                vad_result.frame.timestamp_ms, // 使用边界时间戳作为ASR开始时间
                            );
                        }
                        
                        // TTS合成
                        let (tts_result, tts_ms, yourtts_ms) = if let Some(ref translation) = translation_result {
                            let tts_start = Instant::now();
                            eprintln!("[TTS] 🚀 Starting synthesis immediately after translation...");
                            match self.synthesize_and_publish(translation, vad_result.frame.timestamp_ms, reference_audio.clone(), voice_embedding.clone(), estimated_gender.clone()).await {
                                Ok((result, yt_ms)) => {
                                    let tts_ms = tts_start.elapsed().as_millis() as u64;
                                    eprintln!("[TTS] Synthesis completed in {}ms (audio size: {} bytes)", tts_ms, result.audio.len());
                                    (Some(result), tts_ms, yt_ms)
                                }
                                Err(e) => {
                                    let tts_ms = tts_start.elapsed().as_millis() as u64;
                                    eprintln!("[TTS] Synthesis failed in {}ms: {}", tts_ms, e);
                                    (None, tts_ms, None)
                                }
                            }
                        } else {
                            eprintln!("[TTS] Skipped (no translation result)");
                            (None, 0, None)
                        };
                        
                        (translation_result, tts_result, nmt_ms, tts_ms, yourtts_ms)
                    } else {
                        // 多句模式：增量处理，逐句翻译和TTS
                        self.translate_and_publish_incremental(
                            &sentences,
                            &personalized_with_speaker,
                            vad_result.frame.timestamp_ms,
                            source_audio_duration_ms,
                            reference_audio.clone(),
                            voice_embedding.clone(),
                            estimated_gender.clone(),
                        ).await
                        };
                        
                        // 性能日志记录
                        let total_ms = total_start.elapsed().as_millis() as u64;
                        eprintln!("[PERF] ===== Pipeline timing summary =====");
                        if let Some(se_ms) = speaker_embedding_ms {
                            eprintln!("[PERF] Speaker Embedding: {}ms", se_ms);
                        }
                        eprintln!("[PERF] ASR:                {}ms", asr_ms);
                        eprintln!("[PERF] NMT:                {}ms", nmt_ms);
                        eprintln!("[PERF] TTS:                {}ms", tts_ms);
                        eprintln!("[PERF] Total:              {}ms", total_ms);
                        eprintln!("[PERF] Note: Adaptive VAD overhead < 0.2ms (not shown separately)");
                        if let Some(yt_ms) = yourtts_ms {
                            eprintln!("[PERF] YourTTS:            {}ms", yt_ms);
                        }
                        eprintln!("[PERF] Total:              {}ms", total_ms);
                        eprintln!("[PERF] =====================================");
                        
                        if let Some(ref logger) = self.perf_logger {
                            let config = self.config.current().await.ok();
                            let src_lang = final_transcript.language.clone();
                            let tgt_lang = config.as_ref().map(|c| c.target_language.clone()).unwrap_or_else(|| "zh".to_string());
                            
                            let mut perf_log = PerformanceLog::new(
                                request_id.clone(),
                                src_lang,
                                tgt_lang,
                                asr_ms,
                                nmt_ms,
                                tts_ms,
                                total_ms,
                                translation_result.is_some(),
                            );
                            
                            if let Some(ref translation) = translation_result {
                                perf_log.check_suspect_translation(&final_transcript.text, &translation.translated_text);
                            }
                            
                            logger.log(&perf_log);
                        }
                        
                        (emotion_result, translation_result, tts_result, nmt_ms, tts_ms)
                    } else {
                        (None, None, None, 0, 0)
                    };
                    
                    return Ok(Some(ProcessResult {
                        asr: asr_result,
                        emotion: emotion_result,
                        translation: translation_result,
                        tts: tts_result,
                    }));
                } else {
                    // 未检测到边界，检查是否需要输出部分结果（如果启用流式推理）
                // 注意：仅 WhisperAsrStreaming 支持流式推理
                if let Some(whisper_asr) = whisper_asr_ref {
                    if whisper_asr.is_streaming_enabled() {
                        if let Some(partial) = whisper_asr.infer_partial(vad_result.frame.timestamp_ms).await? {
                            // 发布 ASR 部分结果事件
                            self.publish_asr_partial_event(&partial, vad_result.frame.timestamp_ms).await?;
                            
                            return Ok(Some(ProcessResult {
                                asr: AsrResult {
                                    partial: Some(partial),
                                    final_transcript: None,
                                },
                                emotion: None,
                                translation: None,
                                tts: None,
                            }));
                        }
                        }
                    }
                    // 不需要输出部分结果，返回 None
                    return Ok(None);
                }
                }
            } else {
            // 如果不是支持扩展方法的 ASR 实现，使用原来的 infer 方法
                // 在移动 frame 之前保存需要的信息
                let frame_timestamp = vad_result.frame.timestamp_ms;
                let frame_data_len = vad_result.frame.data.len();
                let frame_sample_rate = vad_result.frame.sample_rate;
                let asr_result = self.asr.infer(crate::asr_streaming::AsrRequest {
                    frame: vad_result.frame,
                    language_hint: language_hint.clone(),
                }).await?;
                
                // 打印 ASR 结果
                if let Some(ref partial) = asr_result.partial {
                    eprintln!("[ASR] 📝 Partial transcript: \"{}\" (confidence: {:.2})", partial.text, partial.confidence);
                }
                if let Some(ref final_transcript) = asr_result.final_transcript {
                    eprintln!("[ASR] ✅ Final transcript: \"{}\" (language: {}, speaker_id: {:?})", 
                             final_transcript.text, final_transcript.language, final_transcript.speaker_id);
                }
                
                // 过滤无意义的 ASR 结果（在进入翻译/TTS 之前）
                if let Some(ref final_transcript) = asr_result.final_transcript {
                if is_meaningless_transcript_filter(&final_transcript.text) {
                        eprintln!("[ASR] ⛔ Filtered meaningless transcript: \"{}\" (skipping translation/TTS)", final_transcript.text);
                        // 直接返回，不进入后续处理
                        return Ok(Some(ProcessResult {
                            asr: asr_result,
                            emotion: None,
                            translation: None,
                            tts: None,
                        }));
                    }
                }
                
                // 如果检测到边界且有最终结果，进行 Emotion 分析、Persona 个性化，然后触发翻译
                if vad_result.is_boundary {
                    if let Some(ref final_transcript) = asr_result.final_transcript {
                    Self::publish_asr_final_event(self, final_transcript, frame_timestamp).await?;
                        
                        // Emotion 情感分析
                        let emotion_result = self.analyze_emotion(final_transcript, frame_timestamp).await.ok();
                        
                        // 应用 Persona 个性化
                        let personalized_transcript = self.personalize_transcript(final_transcript).await?;
                        
                        // 计算原始音频时长（用于后续计算每个 segment 的语速）
                        let source_audio_duration_ms = {
                            Some((frame_data_len as f32 / frame_sample_rate as f32 * 1000.0) as u64)
                        };
                        
                        // 使用个性化后的 transcript 进行翻译
                        let mut translation_result = self.translate_and_publish(&personalized_transcript, frame_timestamp).await.ok();
                        
                        // 将原始音频信息添加到翻译结果中（用于计算每个 segment 的语速）
                        if let Some(ref mut translation) = translation_result {
                            if let Some(ref final_transcript) = asr_result.final_transcript {
                                translation.source_audio_duration_ms = source_audio_duration_ms;
                                translation.source_text = Some(final_transcript.text.clone());
                            }
                        }
                        
                        // 如果翻译成功，进行 TTS 合成
                        let tts_result = if let Some(ref translation) = translation_result {
                        self.synthesize_and_publish(translation, frame_timestamp, None, None, None).await.ok().map(|(chunk, _)| chunk)
                        } else {
                            None
                        };
                        
                        return Ok(Some(ProcessResult {
                            asr: asr_result,
                            emotion: emotion_result,
                            translation: translation_result,
                            tts: tts_result,
                        }));
                    }
                }
                
                // 如果有部分结果，发布事件
                if let Some(ref partial) = asr_result.partial {
                Self::publish_asr_partial_event(self, partial, frame_timestamp).await?;
                }
                
                return Ok(Some(ProcessResult {
                    asr: asr_result,
                    emotion: None,
                    translation: None,
                    tts: None,
                }));
        }
    }

    /// 处理音频帧（连续输入输出模式）
    /// 
    /// 在此模式下：
    /// 1. 将音频帧添加到缓冲区
    /// 2. 通过 VAD 检测边界
    /// 3. 如果检测到边界，异步处理当前片段（不阻塞音频接收）
    /// 4. 继续接收新的音频输入
    async fn process_audio_frame_continuous(
        &self,
        frame: crate::types::AudioFrame,
        language_hint: Option<String>,
    ) -> EngineResult<Option<ProcessResult>> {
        // 获取音频缓冲管理器
        let buffer = self.audio_buffer.as_ref()
            .ok_or_else(|| EngineError::new("Audio buffer not initialized in continuous mode"))?;
        
        // 保存 frame 的 timestamp（在移动之前）
        let current_frame_timestamp = frame.timestamp_ms;
        
        // 1. 将帧添加到缓冲区
        let mut force_boundary = false;
        match buffer.push_frame(frame.clone()).await {
            Ok(()) => {
                // 正常添加
            }
            Err(e) => {
                // 缓冲区溢出，强制触发边界处理
                eprintln!("[VAD] Buffer overflow detected, forcing boundary: {}", e);
                force_boundary = true;
                // 继续执行下面的边界检测逻辑
            }
        }
        
        // 2. VAD 检测（仅在非强制边界时执行）
        let vad_result = if !force_boundary {
            self.vad.detect(frame.clone()).await?
        } else {
            // 强制边界时，创建一个假的检测结果
            crate::vad::DetectionOutcome {
                is_boundary: true,
                confidence: 1.0,
                frame: frame.clone(),
                boundary_type: Some(crate::vad::BoundaryType::ForcedCutoff),
            }
        };
        
        // 3. 如果检测到边界或强制触发，提交当前缓冲区
        if vad_result.is_boundary || force_boundary {
            // 检查最小片段时长（强制触发时跳过此检查，因为已经溢出）
            if !force_boundary && !buffer.check_min_duration().await {
                // 片段太短，继续累积
                eprintln!("[VAD] Segment too short, continuing accumulation");
                return Ok(None);
            }
            
            // 获取当前缓冲区的所有帧
            let frames = buffer.take_current_buffer().await;
            
            if frames.is_empty() {
                // 如果缓冲区为空，但强制触发边界，说明可能是溢出导致的
                // 在这种情况下，如果当前帧存在，应该将其添加到新缓冲区
                if force_boundary {
                    buffer.swap_buffers().await;
                    // 将导致溢出的当前帧添加到新缓冲区
                    if let Err(e) = buffer.push_frame(frame.clone()).await {
                        eprintln!("[VAD] ⚠️ Failed to add overflow frame to new buffer: {}", e);
                    }
                }
                return Ok(None);
            }
            
            // 切换到下一个缓冲区（继续接收新音频）
            buffer.swap_buffers().await;
            
            // 如果是强制边界（溢出），将导致溢出的当前帧添加到新缓冲区
            if force_boundary {
                if let Err(e) = buffer.push_frame(frame.clone()).await {
                    eprintln!("[VAD] ⚠️ Failed to add overflow frame to new buffer: {}", e);
                }
            }
            
            // 合并所有帧为单个音频数据
            let merged_audio = merge_frames(&frames);
            
            // 创建合并后的 AudioFrame
            let merged_frame = crate::types::AudioFrame {
                sample_rate: frames[0].sample_rate,
                channels: frames[0].channels,
                data: merged_audio,
                timestamp_ms: frames[0].timestamp_ms,
            };
            
            // 识别说话者（如果启用了说话者识别）
            let boundary_timestamp = frames.last().map(|f| f.timestamp_ms).unwrap_or(current_frame_timestamp);
            let (speaker_result, speaker_embedding_ms) = if let Some(ref identifier) = self.speaker_identifier {
                let speaker_start = Instant::now();
                eprintln!("[SPEAKER] ===== Speaker Identification Started =====");
                eprintln!("[SPEAKER] Boundary timestamp: {}ms", boundary_timestamp);
                
                // 计算输入音频的总时长
                let total_samples: usize = frames.iter().map(|f| f.data.len()).sum();
                let sample_rate = frames.first().map(|f| f.sample_rate).unwrap_or(16000);
                let total_duration_ms = (total_samples as f32 / sample_rate as f32 * 1000.0) as u64;
                let total_duration_sec = total_samples as f32 / sample_rate as f32;
                eprintln!("[SPEAKER] Input audio: {} frames, {} samples, {:.2}s ({:.0}ms) at {}Hz", 
                         frames.len(), total_samples, total_duration_sec, total_duration_ms, sample_rate);
                
                let result = identifier.identify_speaker(&frames, boundary_timestamp).await;
                let speaker_ms = speaker_start.elapsed().as_millis() as u64;
                
                match result {
                    Ok(speaker_result) => {
                        eprintln!("[SPEAKER] ✅ Identified speaker: {} (is_new: {}, confidence: {:.2})", 
                            speaker_result.speaker_id, speaker_result.is_new_speaker, speaker_result.confidence);
                        eprintln!("[SPEAKER] Voice embedding: {} (dim: {})", 
                            if speaker_result.voice_embedding.is_some() { "Yes" } else { "No" },
                            speaker_result.voice_embedding.as_ref().map(|v| v.len()).unwrap_or(0));
                        eprintln!("[SPEAKER] Reference audio: {} (samples: {})", 
                            if speaker_result.reference_audio.is_some() { "Yes" } else { "No" },
                            speaker_result.reference_audio.as_ref().map(|a| a.len()).unwrap_or(0));
                        eprintln!("[SPEAKER] ⏱️  Speaker identification completed in {}ms", speaker_ms);
                        eprintln!("[SPEAKER] ==============================================");
                        
                        // 设置当前说话者ID（用于VAD自适应调整）
                        // 注意：不再需要设置当前说话者ID，因为使用全局语速历史
                        
                        (Some(speaker_result), Some(speaker_ms))
                    }
                    Err(e) => {
                        eprintln!("[SPEAKER] ❌ Identification failed after {}ms: {}", speaker_ms, e);
                        eprintln!("[SPEAKER] ==============================================");
                        (None, Some(speaker_ms))
                    }
                }
            } else {
                eprintln!("[SPEAKER] Speaker identification disabled");
                (None, None)
            };
            
            let speaker_id = speaker_result.as_ref().map(|r| r.speaker_id.clone());
            let voice_embedding = speaker_result.as_ref().and_then(|r| r.voice_embedding.clone());
            let reference_audio = speaker_result.as_ref().and_then(|r| r.reference_audio.clone());
            let is_new_speaker = speaker_result.as_ref().map(|r| r.is_new_speaker).unwrap_or(false);
            let estimated_gender = speaker_result.as_ref().and_then(|r| r.estimated_gender.clone());
            
            // 如果是新说话者，或者参考音频足够长（合并后的），异步注册/更新到 YourTTS 服务（不阻塞主流程）
            let should_register = is_new_speaker || (reference_audio.as_ref().map(|a| a.len() >= 160000).unwrap_or(false));  // 10秒 @ 16kHz
            if should_register {
                if let (Some(sid), Some(ref_audio)) = (speaker_id.clone(), reference_audio.clone()) {
                    let is_update = !is_new_speaker;
                    let sid_clone = sid.clone();
                    let ref_audio_clone = ref_audio.clone();
                    let voice_embedding_clone = voice_embedding.clone();
                    let tts_endpoint = self.tts_service_url.clone();
                    
                    // 检查是否使用 YourTTS 服务
                    let use_yourtts = tts_endpoint.as_ref()
                        .map(|url| url.contains("5004") || url.contains("yourtts"))
                        .unwrap_or(false);
                    
                    if use_yourtts {
                        if is_update {
                            eprintln!("[SPEAKER] 🔄 Updating speaker '{}' in YourTTS service with merged reference audio ({} samples, {:.2}s, async, non-blocking, continuous mode)...", 
                                     sid_clone, ref_audio_clone.len(), ref_audio_clone.len() as f32 / 16000.0);
                        } else {
                            eprintln!("[SPEAKER] 🚀 Registering new speaker '{}' to YourTTS service (async, non-blocking, continuous mode)...", sid_clone);
                        }
                        
                        // 异步任务：注册 speaker（不阻塞主流程）
                        tokio::spawn(async move {
                            // 创建新的 YourTTS 客户端用于注册
                            use crate::tts_streaming::yourtts_http::{YourTtsHttp, YourTtsHttpConfig};
                            
                            let endpoint = tts_endpoint.unwrap_or_else(|| "http://127.0.0.1:5004".to_string());
                            let config = YourTtsHttpConfig {
                                endpoint: endpoint.clone(),
                                timeout_ms: 30000,  // 30秒超时（注册可能较慢）
                            };
                            
                            match YourTtsHttp::new(config) {
                                Ok(client) => {
                                    if let Err(e) = client.register_speaker(
                                        sid_clone.clone(),
                                        ref_audio_clone,
                                        16000,  // 参考音频采样率（从 ASR/VAD 来的）
                                        voice_embedding_clone,
                                    ).await {
                                        eprintln!("[SPEAKER] ⚠️  Failed to register speaker '{}' to YourTTS (async, non-blocking, continuous mode): {}", sid_clone, e);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[SPEAKER] ⚠️  Failed to create YourTTS client for registration (continuous mode): {}", e);
                                }
                            }
                        });
                    }
                }
            }
            
            // 异步处理当前片段（不阻塞音频接收）
            // 克隆所有需要的组件用于异步任务
            eprintln!("[CONTINUOUS] Processing segment (frame count: {}, duration: {}ms, speaker_id: {:?})...", 
                frames.len(), 
                frames.last().map(|f| f.timestamp_ms - frames[0].timestamp_ms).unwrap_or(0),
                speaker_id
            );
            
            // 处理音频片段并等待结果（这样 WebSocket 可以收到结果）
            let result = self.process_audio_segment(
                merged_frame,
                language_hint,
                speaker_id,
                voice_embedding,
                reference_audio,
                speaker_embedding_ms,
                estimated_gender,
            ).await?;
            
            // 返回处理结果（WebSocket 会收到并发送给客户端）
            return Ok(result);
        }
        
        // 4. 未检测到边界，继续累积
        Ok(None)
    }
    
    /// 处理音频片段（ASR → NMT → TTS）
    /// 
    /// 这是从连续处理模式中分离出来的方法，用于异步处理音频片段
    async fn process_audio_segment(
        &self,
        frame: crate::types::AudioFrame,
        language_hint: Option<String>,
        speaker_id: Option<String>,
        voice_embedding: Option<Vec<f32>>,
        reference_audio: Option<Vec<f32>>,
        speaker_embedding_ms: Option<u64>,
        estimated_gender: Option<String>,
    ) -> EngineResult<Option<ProcessResult>> {
        // 使用原有的 process_audio_frame 逻辑，但需要先通过 VAD 检测
        // 由于我们已经有了完整的音频片段，这里需要特殊处理
        
        // 性能日志：记录总耗时
        let total_start = Instant::now();
        let request_id = Uuid::new_v4().to_string();
        
        // 对于连续模式，我们需要将整个片段传递给 ASR
        // 使用 AsrStreamingExt trait 来统一处理不同的 ASR 实现
        let asr_ptr = Arc::as_ptr(&self.asr);
        let faster_whisper_ptr = asr_ptr as *const FasterWhisperAsrStreaming;
        let whisper_asr_ptr = asr_ptr as *const WhisperAsrStreaming;
        
        // 使用 infer 方法处理整个片段
        let asr_start = Instant::now();
        let segment_duration_ms = frame.data.len() as f32 / frame.sample_rate as f32 * 1000.0;
        eprintln!("[ASR] 🚀 Starting transcription (continuous mode, segment duration: {:.2}ms, samples: {})...", 
                 segment_duration_ms, frame.data.len());
        
        // 设置语言（如果支持）
        if let Some(ref lang_hint) = language_hint {
            let normalized_lang = if lang_hint.starts_with("zh") {
                Some("zh".to_string())
            } else if lang_hint.starts_with("en") {
                Some("en".to_string())
            } else {
                Some(lang_hint.clone())
            };
            
            unsafe {
                // 优先尝试 FasterWhisperAsrStreaming
                let faster_whisper_ref = faster_whisper_ptr.as_ref();
                if let Some(asr_ext) = faster_whisper_ref {
                    if let Err(e) = asr_ext.set_language(normalized_lang.clone()) {
                        eprintln!("[ASR] ⚠️ Warning: Failed to set language on FasterWhisper: {}", e);
                    }
                } else if let Some(whisper_asr) = whisper_asr_ptr.as_ref() {
                    if let Err(e) = whisper_asr.set_language(normalized_lang) {
                        eprintln!("[ASR] ⚠️ Warning: Failed to set language on Whisper: {}", e);
                    }
                }
            }
        }
        
        // 调用 ASR infer 方法（不需要 unsafe，因为使用的是 trait 方法）
        eprintln!("[ASR] Calling ASR infer method...");
        let asr_result = match self.asr.infer(crate::asr_streaming::AsrRequest {
            frame: frame.clone(),
            language_hint: language_hint.clone(),
        }).await {
            Ok(result) => {
                eprintln!("[ASR] ✅ ASR infer call succeeded");
                result
            }
            Err(e) => {
                eprintln!("[ASR] ❌ ASR infer call failed: {}", e);
                return Err(e);
            }
        };
                
        let asr_ms = asr_start.elapsed().as_millis() as u64;
        eprintln!("[ASR] ✅ Transcription completed in {}ms", asr_ms);
        
        // 打印 ASR 结果
        if let Some(ref partial) = asr_result.partial {
            eprintln!("[ASR] 📝 Partial transcript: \"{}\" (confidence: {:.2})", partial.text, partial.confidence);
        }
        if let Some(ref final_transcript) = asr_result.final_transcript {
            eprintln!("[ASR] ✅ Final transcript: \"{}\" (language: {}, speaker_id: {:?})", 
                     final_transcript.text, final_transcript.language, final_transcript.speaker_id);
        } else {
            eprintln!("[ASR] ⚠️ No final transcript received");
        }
        
        // 发布 ASR 最终结果事件
        if let Some(mut final_transcript) = asr_result.final_transcript.clone() {
            // 🔍 过滤无意义的识别文本（在继续处理之前）
            if is_meaningless_transcript_filter(&final_transcript.text) {
                eprintln!("[ASR] 🚫 Filtered meaningless transcript: \"{}\"", final_transcript.text);
                // 返回 None，表示这个结果被过滤掉了
                return Ok(None);
            }
            
            // 设置说话者 ID（如果已识别）
            if final_transcript.speaker_id.is_none() {
                final_transcript.speaker_id = speaker_id.clone();
            }
            
            // 更新语速（如果启用了自适应VAD）
            if let Some(ref _sid) = speaker_id {
                // 计算音频时长
                let total_samples = frame.data.len();
                let sample_rate = frame.sample_rate;
                let audio_duration_ms = (total_samples as f32 / sample_rate as f32 * 1000.0) as u64;
                
                eprintln!("[CoreEngine] 📊 Calculating speech rate (continuous mode): text='{}' ({} chars), audio={}ms ({} samples)", 
                         final_transcript.text.chars().take(30).collect::<String>(), 
                         final_transcript.text.chars().count(), 
                         audio_duration_ms, 
                         total_samples);
                
                // 更新VAD中的全局语速
                // 注意：update_speech_rate 内部会检查语速是否在合理范围内
                // 如果语速异常（可能是误识别），会被自动过滤
                Self::update_vad_speech_rate(self, &final_transcript.text, audio_duration_ms);
            }
            
            let timestamp = frame.timestamp_ms;
            Self::publish_asr_final_event(self, &final_transcript, timestamp).await?;
            
            // 继续处理：Emotion → Persona → NMT → TTS
            let emotion_result = self.analyze_emotion(&final_transcript, timestamp).await.ok();
            let personalized_transcript = self.personalize_transcript(&final_transcript).await?;
            
            // 计算原始音频时长（用于后续计算每个 segment 的语速）
            let source_audio_duration_ms = {
                let total_samples = frame.data.len();
                let sample_rate = frame.sample_rate;
                Some((total_samples as f32 / sample_rate as f32 * 1000.0) as u64)
            };
            
            let nmt_start = Instant::now();
            eprintln!("[NMT] Starting translation (continuous mode, speaker_id: {:?})...", personalized_transcript.speaker_id);
            let mut translation_result = self.translate_and_publish(&personalized_transcript, timestamp).await.ok();
            
            // 将原始音频信息添加到翻译结果中（用于计算每个 segment 的语速）
            if let Some(ref mut translation) = translation_result {
                if let Some(ref final_transcript) = asr_result.final_transcript {
                    translation.source_audio_duration_ms = source_audio_duration_ms;
                    translation.source_text = Some(final_transcript.text.clone());
                    translation.source_language = Some(final_transcript.language.clone());
                }
            }
            
            let nmt_ms = nmt_start.elapsed().as_millis() as u64;
            eprintln!("[NMT] Translation completed in {}ms", nmt_ms);
            
            let (tts_result, tts_ms, yourtts_ms) = if let Some(ref translation) = translation_result {
                let tts_start = Instant::now();
                eprintln!("[TTS] ===== TTS Synthesis Started =====");
                eprintln!("[TTS] Text: '{}'", translation.translated_text);
                eprintln!("[TTS] Speaker ID: {:?}", translation.speaker_id);
                eprintln!("[TTS] Reference audio: {} (samples: {})", 
                    if reference_audio.is_some() { "Yes" } else { "No" },
                    reference_audio.as_ref().map(|a| a.len()).unwrap_or(0));
                let voice_embedding_for_tts = voice_embedding.clone();
                match self.synthesize_and_publish(translation, timestamp, reference_audio.clone(), voice_embedding_for_tts, estimated_gender.clone()).await {
                    Ok((result, yourtts_time)) => {
                        let tts_ms = tts_start.elapsed().as_millis() as u64;
                        // 注意：在增量模式下，result 只是一个占位符（第一个 segment）
                        // 实际所有 segments 已通过事件独立发布，客户端应该通过事件总线接收
                        if self.tts_incremental_enabled {
                            eprintln!("[TTS] ✅ Incremental synthesis completed in {}ms (segments published independently, placeholder size: {} bytes)", 
                                tts_ms, result.audio.len());
                        } else {
                            eprintln!("[TTS] ✅ Synthesis completed in {}ms (audio size: {} bytes)", tts_ms, result.audio.len());
                        }
                        eprintln!("[TTS] ==========================================");
                        (Some(result), tts_ms, yourtts_time)
                    }
                    Err(e) => {
                        let tts_ms = tts_start.elapsed().as_millis() as u64;
                        eprintln!("[TTS] ❌ Synthesis failed in {}ms: {}", tts_ms, e);
                        eprintln!("[TTS] ==========================================");
                        (None, tts_ms, None)
                    }
                }
            } else {
                eprintln!("[TTS] Skipped (no translation result)");
                (None, 0, None)
            };
            
            // 性能日志
            let total_ms = total_start.elapsed().as_millis() as u64;
            eprintln!("[PERF] ===== Continuous mode timing summary =====");
            eprintln!("[PERF] ASR:                {}ms", asr_ms);
            if let Some(se_ms) = speaker_embedding_ms {
                eprintln!("[PERF] Speaker Embedding: {}ms", se_ms);
            }
            eprintln!("[PERF] NMT:                {}ms", nmt_ms);
            eprintln!("[PERF] TTS:                {}ms", tts_ms);
            if let Some(yt_ms) = yourtts_ms {
                eprintln!("[PERF] YourTTS:            {}ms", yt_ms);
            }
            eprintln!("[PERF] Total:              {}ms", total_ms);
            eprintln!("[PERF] ===========================================");
            
            if let Some(ref logger) = self.perf_logger {
                let config = self.config.current().await.ok();
                let src_lang = final_transcript.language.clone();
                let tgt_lang = config.as_ref().map(|c| c.target_language.clone()).unwrap_or_else(|| "zh".to_string());
                
                let mut perf_log = PerformanceLog::new(
                    request_id.clone(),
                    src_lang,
                    tgt_lang,
                    asr_ms,
                    nmt_ms,
                    tts_ms,
                    total_ms,
                    translation_result.is_some(),
                );
                
                if let Some(ref translation) = translation_result {
                    perf_log.check_suspect_translation(&final_transcript.text, &translation.translated_text);
                }
                
                logger.log(&perf_log);
            }
            
            // 更新 ASR 结果中的 speaker_id
            let mut asr_result_with_speaker = asr_result;
            if let Some(ref mut final_t) = asr_result_with_speaker.final_transcript {
                final_t.speaker_id = speaker_id.clone();
            }
            
            return Ok(Some(ProcessResult {
                asr: asr_result_with_speaker,
                emotion: emotion_result,
                translation: translation_result,
                tts: tts_result,
            }));
        }
        
        Ok(None)
    }

    /// 分析情感
    async fn analyze_emotion(
        &self,
        transcript: &StableTranscript,
        timestamp_ms: u64,
    ) -> EngineResult<EmotionResponse> {
        // 构造 Emotion 请求（根据 Emotion_Adapter_Spec.md）
        let request = EmotionRequest {
            text: transcript.text.clone(),
            lang: transcript.language.clone(),
        };
        
        // 执行情感分析
        let response = self.emotion.analyze(request).await?;
        
        // 发布 Emotion 事件
        Self::publish_emotion_event(self, &response, timestamp_ms).await?;
        
        Ok(response)
    }

    /// 应用 Persona 个性化
    async fn personalize_transcript(
        &self,
        transcript: &StableTranscript,
    ) -> EngineResult<StableTranscript> {
        // 从配置中获取 PersonaContext（简化版：使用默认值）
        // TODO: 后续可以从用户配置或数据库获取真实的 PersonaContext
        let context = PersonaContext {
            user_id: "default_user".to_string(),
            tone: "formal".to_string(),  // 默认使用正式语调
            culture: transcript.language.clone(),
        };
        
        // 应用个性化
        self.persona.personalize(transcript.clone(), context).await
    }

    // split_into_sentences 和 split_into_sentences_fine_grained 已移至 text_utils.rs 模块

    /// 增量翻译并发布事件（逐句翻译和TTS）
    /// 
    /// 对每个句子分别进行翻译和TTS，实现实时反馈
    async fn translate_and_publish_incremental(
        &self,
        sentences: &[String],
        original_transcript: &StableTranscript,
        timestamp_ms: u64,
        source_audio_duration_ms: Option<u64>,
        reference_audio: Option<Vec<f32>>,
        voice_embedding: Option<Vec<f32>>,
        estimated_gender: Option<String>,
    ) -> (Option<TranslationResponse>, Option<TtsStreamChunk>, u64, u64, Option<u64>) {
        use futures::future::join_all;
        
        let nmt_start = Instant::now();
        let mut all_translations = Vec::new();
        let mut all_tts_chunks = Vec::new();
        let mut total_yourtts_ms = 0u64;
        let mut yourtts_call_count = 0u64;
        
        // 计算每个句子对应的音频时长（用于语速计算）
        let total_chars = sentences.iter().map(|s| s.chars().count()).sum::<usize>();
        let sentence_durations: Vec<Option<u64>> = if let Some(total_duration) = source_audio_duration_ms {
            sentences.iter().map(|sentence| {
                let sentence_chars = sentence.chars().count();
                if total_chars > 0 {
                    let ratio = sentence_chars as f32 / total_chars as f32;
                    Some((total_duration as f32 * ratio) as u64)
                } else {
                    None
                }
            }).collect()
        } else {
            sentences.iter().map(|_| None).collect()
        };
        
        // 逐句翻译和TTS（并行处理以提高效率）
        eprintln!("[NMT] ⚡ Starting parallel translation and TTS for {} sentences...", sentences.len());
        
        // 在闭包外部克隆 estimated_gender，确保它在作用域内
        let estimated_gender_clone_for_tasks = estimated_gender.clone();
        
        let sentence_tasks: Vec<_> = sentences.iter().enumerate().map(|(idx, sentence)| {
            let sentence_clone = sentence.clone();
            let transcript_clone = original_transcript.clone();
            let engine_clone = self.clone();
            let reference_audio_clone = reference_audio.clone();
            let voice_embedding_clone = voice_embedding.clone();
            let estimated_gender_clone = estimated_gender_clone_for_tasks.clone();
            let sentence_duration = sentence_durations.get(idx).and_then(|d| *d);
            
            async move {
                // 创建单个句子的transcript
                let mut sentence_transcript = transcript_clone.clone();
                sentence_transcript.text = sentence_clone.clone();
                
                // 翻译单个句子
                let sentence_nmt_start = Instant::now();
                eprintln!("[NMT] ⚡ Translating sentence {}/{}: '{}'", idx + 1, sentences.len(), sentence_clone);
                let translation_result = engine_clone.translate_and_publish(&sentence_transcript, timestamp_ms + (idx as u64 * 100)).await.ok();
                let sentence_nmt_ms = sentence_nmt_start.elapsed().as_millis() as u64;
                
                if let Some(ref translation) = translation_result {
                    eprintln!("[NMT] ✅ Sentence {}/{} translated in {}ms: '{}'", 
                        idx + 1, sentences.len(), sentence_nmt_ms, translation.translated_text);
                    
                    // 设置源音频时长（用于语速计算）
                    let mut translation_with_duration = translation.clone();
                    translation_with_duration.source_audio_duration_ms = sentence_duration;
                    translation_with_duration.source_text = Some(sentence_clone.clone());
                    
                    // TTS合成
                    let sentence_tts_start = Instant::now();
                    eprintln!("[TTS] ⚡ Synthesizing sentence {}/{}: '{}'", idx + 1, sentences.len(), translation.translated_text);
                    match engine_clone.synthesize_and_publish(
                        &translation_with_duration,
                        timestamp_ms + (idx as u64 * 100),
                        reference_audio_clone.clone(),
                        voice_embedding_clone.clone(),
                        estimated_gender_clone.clone()
                    ).await {
                        Ok((tts_chunk, yourtts_ms)) => {
                            let sentence_tts_ms = sentence_tts_start.elapsed().as_millis() as u64;
                            eprintln!("[TTS] ✅ Sentence {}/{} synthesized in {}ms (audio size: {} bytes)", 
                                idx + 1, sentences.len(), sentence_tts_ms, tts_chunk.audio.len());
                            
                            if yourtts_ms.is_some() {
                                yourtts_call_count += 1;
                                total_yourtts_ms += yourtts_ms.unwrap_or(0);
                            }
                            
                            Ok((translation_with_duration, tts_chunk, yourtts_ms))
                        }
                        Err(e) => {
                            let sentence_tts_ms = sentence_tts_start.elapsed().as_millis() as u64;
                            eprintln!("[TTS] ❌ Sentence {}/{} synthesis failed in {}ms: {}", 
                                idx + 1, sentences.len(), sentence_tts_ms, e);
                            Err(e)
                        }
                    }
                } else {
                    eprintln!("[NMT] ❌ Sentence {}/{} translation failed", idx + 1, sentences.len());
                    Err(EngineError::new("Translation failed"))
                }
            }
        }).collect();
        
        // 等待所有句子处理完成
        let results = join_all(sentence_tasks).await;
        
        // 收集结果
        for result in results {
            if let Ok((translation, tts_chunk, _)) = result {
                all_translations.push(translation);
                all_tts_chunks.push(tts_chunk);
            }
        }
        
        let nmt_ms = nmt_start.elapsed().as_millis() as u64;
        
        // 合并所有翻译结果
        let merged_translation = if !all_translations.is_empty() {
            let merged_text = all_translations.iter()
                .map(|t| t.translated_text.clone())
                .collect::<Vec<_>>()
                .join(" ");
            
            let mut merged = all_translations[0].clone();
            merged.translated_text = merged_text;
            merged.source_text = Some(original_transcript.text.clone());
            merged.source_audio_duration_ms = source_audio_duration_ms;
            
            Some(merged)
        } else {
            None
        };
        
        // 合并所有TTS音频
        let merged_tts = if !all_tts_chunks.is_empty() {
            let mut merged_audio = Vec::new();
            for chunk in &all_tts_chunks {
                merged_audio.extend_from_slice(&chunk.audio);
            }
            
            Some(TtsStreamChunk {
                audio: merged_audio,
                timestamp_ms: all_tts_chunks[0].timestamp_ms,
                is_last: true,
            })
        } else {
            None
        };
        
        let tts_ms = if let Some(ref tts) = merged_tts {
            // 估算TTS总耗时（实际是并行处理的）
            nmt_ms // 简化：使用NMT耗时作为参考
        } else {
            0
        };
        
        let yourtts_ms = if yourtts_call_count > 0 {
            Some(total_yourtts_ms)
        } else {
            None
        };
        
        eprintln!("[NMT] ✅ Incremental translation completed: {} sentences in {}ms", sentences.len(), nmt_ms);
        
        (merged_translation, merged_tts, nmt_ms, tts_ms, yourtts_ms)
    }

    /// 翻译并发布事件
    async fn translate_and_publish(
        &self,
        transcript: &StableTranscript,
        timestamp_ms: u64,
    ) -> EngineResult<TranslationResponse> {
        // 1. 获取目标语言（从配置中，使用 .ok() 避免阻塞）
        // ⚠️ 优化：如果配置获取失败，使用默认值而不是阻塞整个流程
        let target_language = self.config.current().await
            .map(|c| c.target_language)
            .unwrap_or_else(|_| {
                eprintln!("[NMT] ⚠️  Failed to get config, using default target_language: zh");
                "zh".to_string()
            });
        
        // 2. 构造翻译请求（传递 speaker_id）
        let translation_request = TranslationRequest {
            transcript: PartialTranscript {
                text: transcript.text.clone(),
                confidence: 1.0,  // 最终转录的置信度
                is_final: true,
            },
            target_language: target_language.clone(),
            wait_k: None,
            speaker_id: transcript.speaker_id.clone(),  // 传递 speaker_id
        };
        
        // 3. 执行翻译
        let mut translation_response = self.nmt.translate(translation_request).await?;
        
        // 3.1. 确保 speaker_id 被传递到 TranslationResponse
        if translation_response.speaker_id.is_none() {
            translation_response.speaker_id = transcript.speaker_id.clone();
        }
        eprintln!("[NMT] Raw translation result: '{}'", translation_response.translated_text);
        
        // 4. 应用翻译质量检查
        if let Some(ref checker) = self.quality_checker {
            let before_check = translation_response.translated_text.clone();
            let checked_text = checker.check_and_fix(
                &transcript.text,
                &translation_response.translated_text,
                &target_language,
            );
            if before_check != checked_text {
                eprintln!("[NMT] After quality check: '{}' (was: '{}')", checked_text, before_check);
            }
            translation_response.translated_text = checked_text;
        }
        
        // 5. 应用文本后处理
        if let Some(ref processor) = self.post_processor {
            let before_process = translation_response.translated_text.clone();
            let processed_text = processor.process(&translation_response.translated_text, &target_language);
            if before_process != processed_text {
                eprintln!("[NMT] After post-processing: '{}' (was: '{}')", processed_text, before_process);
            }
            translation_response.translated_text = processed_text;
        }
        
        eprintln!("[NMT] Final translation: '{}'", translation_response.translated_text);
        
        // 6. 发布翻译事件
        Self::publish_translation_event(self, &translation_response, timestamp_ms).await?;
        
        Ok(translation_response)
    }

    // publish_asr_partial_event, publish_asr_final_event, publish_emotion_event, publish_translation_event 已移至 events.rs 模块

    /// TTS 合成并发布事件
    /// 
    /// 返回 (TtsStreamChunk, YourTTS耗时)
    async fn synthesize_and_publish(
        &self,
        translation: &TranslationResponse,
        timestamp_ms: u64,
        reference_audio: Option<Vec<f32>>,
        voice_embedding: Option<Vec<f32>>,
        estimated_gender: Option<String>,
    ) -> EngineResult<(TtsStreamChunk, Option<u64>)> {
        // 如果启用增量播放，使用增量合成方法
        if self.tts_incremental_enabled {
            return self.synthesize_and_publish_incremental(translation, timestamp_ms, reference_audio, voice_embedding, estimated_gender).await;
        }

        // 原有的一次性合成逻辑
        // 1. 获取目标语言（用于 TTS locale）
        // ⚠️ 优化：如果配置获取失败，使用默认值而不是阻塞整个流程
        let target_language = self.config.current().await
            .map(|c| c.target_language)
            .unwrap_or_else(|_| {
                eprintln!("[TTS] ⚠️  Failed to get config, using default target_language: zh");
                "zh".to_string()
            });
        
        // 2. 对中文文本进行预处理：将小数转换为中文读法
        let processed_text = if target_language.starts_with("zh") {
            Self::convert_decimals_to_chinese(&translation.translated_text)
        } else {
            translation.translated_text.clone()
        };
        
        // 3. 使用传入的 reference_audio（用于 zero-shot TTS）
        eprintln!("[TTS] Reference audio: {} (samples: {})", 
            if reference_audio.is_some() { "Yes" } else { "No" },
            reference_audio.as_ref().map(|a| a.len()).unwrap_or(0));
        
        // 4. 选择 voice（如果启用了说话者音色映射）
        // 策略：
        // 1. 如果有 reference_audio，优先使用 zero-shot TTS（voice 可以为空）
        // 2. 如果没有 reference_audio 但有 speaker_id，使用说话者音色映射
        // 3. 如果都没有，根据 estimated_gender 选择默认音色（男/女）
        let voice = if reference_audio.is_none() {
            if let Some(ref speaker_id) = translation.speaker_id {
                if let Some(ref mapper) = self.speaker_voice_mapper {
                    let assigned_voice = mapper.get_or_assign_voice(speaker_id).await;
                    eprintln!("[TTS] Assigned voice: '{}' for speaker: {}", assigned_voice, speaker_id);
                    assigned_voice
                } else {
                    // 没有 voice mapper，根据性别选择默认音色
                    let default_voice = Self::get_default_voice_by_gender(estimated_gender.as_ref());
                    eprintln!("[TTS] No voice mapper, using default voice based on gender: '{}'", default_voice);
                    default_voice
                }
            } else {
                // 没有 speaker_id，根据性别选择默认音色
                let default_voice = Self::get_default_voice_by_gender(estimated_gender.as_ref());
                eprintln!("[TTS] No speaker_id, using default voice based on gender: '{}' (estimated_gender: {:?})", 
                         default_voice, estimated_gender);
                default_voice
            }
        } else {
            // 使用 zero-shot TTS，voice 可以为空
            eprintln!("[TTS] Using zero-shot TTS mode (reference audio provided)");
            String::new()
        };
        
        // 5. 构造 TTS 请求
        // 安全截取字符串：使用字符边界而不是字节边界
        let text_preview = if processed_text.chars().count() > 50 {
            processed_text.chars().take(50).collect::<String>()
        } else {
            processed_text.clone()
        };
        eprintln!("[TTS] Constructing TTS request: text='{}', voice='{}', locale='{}'", 
            text_preview, voice, target_language);
        
        // 5.1. 获取全局语速（如果启用了自适应VAD）
        // 注意：不区分说话者，使用全局语速历史
        let speech_rate = Self::get_vad_speech_rate(self);
        
        if let Some(rate) = speech_rate {
            eprintln!("[TTS] ✅ Using source speech rate: {:.2} chars/s (from user's input, will be applied to translated text)", 
                     rate);
        } else {
            eprintln!("[TTS] ⚠️  ⚠️  ⚠️  No speech rate available (VAD adaptive may be disabled or insufficient samples)");
            eprintln!("[TTS]    This means TTS will use default/normal rate instead of matching user's speech rate");
        }
        
        // 确定是否使用 speaker_id 或 reference_audio
        // 策略：
        // 1. 如果提供了 speaker_id，优先使用它（服务端会查找缓存的 reference_audio）
        //    如果缓存中没有，服务端会使用默认音色（不阻塞合成）
        // 2. 如果没有 speaker_id 但有 reference_audio，使用 reference_audio
        // 3. 如果都没有，使用 speaker 参数（预定义音色）
        // 
        // 多人模式特殊处理：
        // - 多人模式下，speaker_id 是 default_male/default_female/default_speaker
        // - 这些 speaker_id 不应该传递给 YourTTS（因为 YourTTS 会查找缓存的 reference_audio）
        // - 应该使用 speaker 参数（从 voice 字段映射），让 YourTTS 使用预定义的音色
        let is_multi_user_mode = translation.speaker_id.as_ref()
            .map(|s| s.starts_with("default_"))
            .unwrap_or(false);
        
        let use_speaker_id = translation.speaker_id.is_some() && !is_multi_user_mode;
        let has_reference_audio = reference_audio.is_some();
        
        // 在多人模式下，从 voice 字段获取 speaker 参数
        // 如果使用 YourTTS，需要使用 YourTTS 的预定义 speaker 名称（如 "male-en-5", "female-en-5"）
        // 而不是从 speaker_voice_mapper 获取的 voice（那是给 Piper TTS 用的）
        let is_yourtts = self.tts_service_url.as_ref()
            .map(|url| url.contains("5004") || url.contains("yourtts"))
            .unwrap_or(false);
        
        let speaker_for_request = if is_multi_user_mode {
            if is_yourtts {
                // 多人模式 + YourTTS：使用 YourTTS 的预定义 speaker
                Some(Self::get_yourtts_speaker_by_gender(estimated_gender.as_ref()))
            } else {
                // 多人模式 + Piper TTS：使用从 voice 字段映射的 speaker
                if !voice.is_empty() {
                    Some(voice.clone())
                } else {
                    // 如果 voice 为空，根据性别选择默认音色
                    Some(Self::get_default_voice_by_gender(estimated_gender.as_ref()))
                }
            }
        } else if !use_speaker_id && !has_reference_audio {
            if is_yourtts {
                Some(Self::get_yourtts_speaker_by_gender(estimated_gender.as_ref()))
            } else {
                Some(Self::get_default_speaker_by_gender(estimated_gender.as_ref()))
            }
        } else {
            None
        };
        
        let tts_request = TtsRequest {
            text: processed_text,
            voice: voice.clone(),  // 克隆 voice，因为后面可能还需要使用
            locale: target_language.clone(),
            speaker_id: if is_multi_user_mode {
                None  // 多人模式下不传递 speaker_id，使用 speaker 参数
            } else {
                translation.speaker_id.clone()  // 单人模式下传递 speaker_id
            },
            reference_audio: if !use_speaker_id {
                reference_audio  // 只有在没有 speaker_id 时才传递 reference_audio
            } else {
                None  // 如果有 speaker_id，不传递 reference_audio（使用缓存的）
            },
            voice_embedding: if !use_speaker_id && has_reference_audio {
                voice_embedding  // 只有在使用 reference_audio 时才传递 voice_embedding
            } else {
                None
            },
            speaker: speaker_for_request,
            speech_rate,
        };
        
        // 6. 执行 TTS 合成
        let tts_synth_start = Instant::now();
        eprintln!("[TTS] Calling TTS service...");
        eprintln!("[TTS] Request details: speaker_id={:?}, reference_audio={}, voice_embedding={}, speaker={:?}, voice='{}'", 
                 tts_request.speaker_id,
                 if tts_request.reference_audio.is_some() { 
                     format!("Yes ({} samples)", tts_request.reference_audio.as_ref().map(|a| a.len()).unwrap_or(0))
                 } else { 
                     "No".to_string() 
                 },
                 if tts_request.voice_embedding.is_some() {
                     format!("Yes ({} dims)", tts_request.voice_embedding.as_ref().map(|e| e.len()).unwrap_or(0))
                 } else { 
                     "No".to_string() 
                 },
                 tts_request.speaker,
                 tts_request.voice);
        
        // 检查是否使用 YourTTS
        let is_yourtts = self.tts_service_url.as_ref()
            .map(|url| url.contains("5004") || url.contains("yourtts"))
            .unwrap_or(false);
        if is_yourtts {
            eprintln!("[TTS] ✅ Using YourTTS service (zero-shot TTS with reference audio support)");
        } else {
            eprintln!("[TTS] ⚠️  Using non-YourTTS service (Piper or other), reference audio will NOT be used!");
        }
        
        // 尝试使用主 TTS 服务，如果失败且是语言不支持的错误，使用 fallback TTS
        let tts_chunk = match self.tts.synthesize(tts_request.clone()).await {
            Ok(chunk) => chunk,
            Err(e) => {
                // 检查是否是语言不支持的错误（YourTTS 不支持中文）
                let error_msg = e.to_string().to_lowercase();
                let is_language_error = error_msg.contains("does not support chinese") ||
                                        error_msg.contains("language") || 
                                        error_msg.contains("not in the available languages") ||
                                        error_msg.contains("不支持") ||
                                        error_msg.contains("dict_keys") ||
                                        error_msg.contains("dimension out of range");
                
                if is_language_error && self.fallback_tts.is_some() {
                    eprintln!("[TTS] ⚠️  TTS failed due to unsupported language, trying fallback TTS...");
                    if let Some(ref fallback_tts) = self.fallback_tts {
                        let mut fallback_request = tts_request.clone();
                        fallback_request.reference_audio = None;  // Piper 不支持 reference_audio
                        fallback_request.speaker_id = None;  // Piper 不支持 speaker_id
                        fallback_request.voice_embedding = None;  // Piper 不支持 voice_embedding
                        eprintln!("[TTS] ⚡ Fallback: Using Piper TTS with speech_rate={:?}", fallback_request.speech_rate);
                        fallback_tts.synthesize(fallback_request).await?
                    } else {
                        return Err(e);
                    }
                } else {
                    return Err(e);
                }
            }
        };
        let tts_synth_ms = tts_synth_start.elapsed().as_millis() as u64;
        eprintln!("[TTS] TTS service call completed in {}ms", tts_synth_ms);
        
        // 6. 发布 TTS 事件（包含 speaker_id 信息）
        Self::publish_tts_event(self, &tts_chunk, timestamp_ms).await?;
        
        // 如果使用 YourTTS，记录 YourTTS 的耗时（从日志中提取或使用总耗时）
        // 注意：YourTTS 的耗时已经在 yourtts_http.rs 中记录，这里我们使用总耗时作为近似值
        // 如果 TTS 服务是 YourTTS，则返回耗时；否则返回 None
        let yourtts_ms = if self.tts_service_url.as_ref()
            .map(|url| url.contains("5004") || url.contains("yourtts"))
            .unwrap_or(false) {
            Some(tts_synth_ms)
        } else {
            None
        };
        
        Ok((tts_chunk, yourtts_ms))
    }

    /// TTS 增量合成并发布事件
    /// 
    /// 将文本分割成短句，每个短句合成完成后立即发布（或缓冲后发布）
    /// 
    /// 返回 (TtsStreamChunk, YourTTS耗时)
    async fn synthesize_and_publish_incremental(
        &self,
        translation: &TranslationResponse,
        timestamp_ms: u64,
        reference_audio: Option<Vec<f32>>,
        voice_embedding: Option<Vec<f32>>,
        estimated_gender: Option<String>,
    ) -> EngineResult<(TtsStreamChunk, Option<u64>)> {
        // 1. 获取目标语言（用于 TTS locale）
        // ⚠️ 优化：如果配置获取失败，使用默认值而不是阻塞整个流程
        let target_language = self.config.current().await
            .map(|c| c.target_language)
            .unwrap_or_else(|_| {
                eprintln!("[TTS] ⚠️  Failed to get config, using default target_language: zh");
                "zh".to_string()
            });
        
        // 2. 分割文本为短句（使用带停顿类型的分段）
        let segmenter = self.text_segmenter.as_ref()
            .ok_or_else(|| EngineError::new("Text segmenter not initialized".to_string()))?;
        
        eprintln!("[TTS] Input text for segmentation: '{}'", translation.translated_text);
        
        // 尝试使用带停顿类型的分段（如果支持）
        let segments_with_pause = if segmenter.split_on_comma {
            let segments = segmenter.segment_with_pause_type(&translation.translated_text);
            eprintln!("[TTS] Segmented into {} parts:", segments.len());
            for (i, seg) in segments.iter().enumerate() {
                eprintln!("[TTS]   Segment {}: '{}' (pause_type: {:?})", i + 1, seg.text, seg.pause_type);
            }
            segments
        } else {
            // 向后兼容：使用旧的分段方法
            let segments = segmenter.segment(&translation.translated_text);
            eprintln!("[TTS] Segmented into {} parts (legacy method):", segments.len());
            for (i, text) in segments.iter().enumerate() {
                eprintln!("[TTS]   Segment {}: '{}'", i + 1, text);
            }
            segments
                .into_iter()
                .map(|text| {
                    let pause_type = if text.ends_with('.') 
                        || text.ends_with('!') 
                        || text.ends_with('?')
                        || text.ends_with('。')
                        || text.ends_with('！')
                        || text.ends_with('？') {
                        crate::text_segmentation::PauseType::SentenceEnd
                    } else {
                        crate::text_segmentation::PauseType::None
                    };
                    crate::text_segmentation::TextSegment { text, pause_type }
                })
                .collect()
        };
        
        if segments_with_pause.is_empty() {
            return Err(EngineError::new("No segments to synthesize".to_string()));
        }

        // 3. 准备阶段：预先准备所有 TTS 请求参数（包括异步的 voice 获取）
        let tts_incremental_start = Instant::now();
        
        // 3.1. 预先获取 voice（如果需要，且只获取一次）
        let use_reference_audio = reference_audio.clone();
        let use_voice_embedding = voice_embedding.clone();
        let common_voice = if use_reference_audio.is_none() {
            if let Some(ref speaker_id) = translation.speaker_id {
                if let Some(ref mapper) = self.speaker_voice_mapper {
                    mapper.get_or_assign_voice(speaker_id).await
                } else {
                    // 没有 voice mapper，根据性别选择默认音色
                    Self::get_default_voice_by_gender(estimated_gender.as_ref())
                }
            } else {
                // 没有 speaker_id，根据性别选择默认音色
                Self::get_default_voice_by_gender(estimated_gender.as_ref())
            }
        } else {
            String::new()  // 使用 zero-shot TTS，voice 可以为空
        };
        
        // 3.2. 计算每个 segment 的独立语速
        let segment_speech_rates: Vec<Option<f32>> = if let (Some(source_duration_ms), Some(source_text)) = 
            (translation.source_audio_duration_ms, translation.source_text.clone()) {
            let source_text_len = source_text.chars().count() as f32;
            let source_duration_sec = source_duration_ms as f32 / 1000.0;
            let overall_speech_rate = if source_duration_sec > 0.0 {
                source_text_len / source_duration_sec
            } else {
                0.0
            };
            
            // ⚠️ 重要：语速应该基于原始输入文本和音频时长，而不是翻译后的文本
            // 翻译后的文本长度可能完全不同，直接用翻译文本计算语速会导致错误
            // 
            // 语速换算说明：
            // - 中文语速：通常 3-6 字符/秒（正常说话）
            // - 英文语速：通常 2-4 词/秒，约 10-20 字符/秒（正常说话）
            // - 当前实现：统一使用字符/秒，但中英文的"字符"含义不同
            //   * 中文：1个汉字 = 1个字符
            //   * 英文：1个字母 = 1个字符（但实际语速单位应该是"词/秒"）
            // 
            // 问题：如果输入是英文（如 15 字符/秒），直接用于中文TTS会过快
            // 解决方案：根据源语言和目标语言进行语速换算
            // - 如果源语言是英文，目标语言是中文：需要降低语速（英文字符/秒通常比中文高）
            // - 如果源语言是中文，目标语言是英文：可以保持或略微提高语速
            //
            // 换算比例（经验值）：
            // - 英文 -> 中文：英文语速 / 2.5（因为英文字符/秒通常比中文高2-3倍）
            // - 中文 -> 英文：中文语速 * 1.2（略微提高，因为英文单词通常比中文字符长）
            // - 同语言：保持原语速
            let adjusted_speech_rate = if let Some(ref source_lang) = translation.source_language {
                let is_source_english = source_lang.starts_with("en");
                let is_target_chinese = target_language.starts_with("zh");
                let is_target_english = target_language.starts_with("en");
                
                if is_source_english && is_target_chinese {
                    // 英文 -> 中文：降低语速
                    let adjusted = overall_speech_rate / 2.5;
                    eprintln!("[TTS] 🔄 Speech rate conversion: EN->ZH: {:.2} -> {:.2} chars/s (divided by 2.5)", 
                             overall_speech_rate, adjusted);
                    adjusted
                } else if !is_source_english && is_target_english {
                    // 中文 -> 英文：提高语速 50%（乘以 1.5）
                    let adjusted = overall_speech_rate * 1.5;
                    eprintln!("[TTS] 🔄 Speech rate conversion: ZH->EN: {:.2} -> {:.2} chars/s (multiplied by 1.5, +50%)", 
                             overall_speech_rate, adjusted);
                    adjusted
                } else if !is_source_english && is_target_chinese {
                    // 中文 -> 中文：保持原语速
                    overall_speech_rate
                } else {
                    // 其他情况：保持原语速（或可以添加更多换算规则）
                    overall_speech_rate
                }
            } else {
                // 如果没有源语言信息，假设是中文输入
                overall_speech_rate
            };
            
            // 为每个 segment 计算语速
            // 注意：语速（chars/s）是说话者的特征，应该保持一致
            // 但每个 segment 的文本长度不同，所以播放时长会不同
            segments_with_pause.iter().map(|seg| {
                if source_duration_sec > 0.0 && adjusted_speech_rate > 0.0 {
                    let segment_text_len = seg.text.chars().count() as f32;
                    let expected_duration_sec = segment_text_len / adjusted_speech_rate;
                    eprintln!("[TTS] 📊 Segment '{}' ({} chars): speech_rate={:.2} chars/s, expected_duration={:.2}s", 
                             seg.text, segment_text_len, adjusted_speech_rate, expected_duration_sec);
                    Some(adjusted_speech_rate)
                } else {
                    None
                }
            }).collect()
        } else {
            segments_with_pause.iter().map(|_| {
                // 注意：不区分说话者，使用全局语速历史
                Self::get_vad_speech_rate(self)
            }).collect()
        };
        
        // 3.3. 创建所有 segment 的并行处理任务
        eprintln!("[TTS] ⚡ Starting parallel synthesis of {} segments...", segments_with_pause.len());
        let segment_futures: Vec<_> = segments_with_pause.iter().enumerate().map(|(idx, segment)| {
            let is_last = idx == segments_with_pause.len() - 1;
            let segment_text = segment.text.clone();
            let segment_pause_type = segment.pause_type;
            
            // 预处理文本
            let processed_text = if target_language.starts_with("zh") {
                Self::convert_decimals_to_chinese(&segment_text)
            } else {
                segment_text.clone()
            };
            
            // 获取语速
            let speech_rate = segment_speech_rates.get(idx)
                .and_then(|rate| *rate)
                .or_else(|| {
                    // 注意：不区分说话者，使用全局语速历史
                    let rate = Self::get_vad_speech_rate(self);
                    if rate.is_none() {
                        eprintln!("[TTS] ⚠️  No speech rate available for segment {} (VAD adaptive may be disabled or insufficient samples)", idx + 1);
                    }
                    rate
                });
            
            // 构造 TTS 请求
            // 多人模式特殊处理：
            // - 多人模式下，speaker_id 是 default_male/default_female/default_speaker
            // - 这些 speaker_id 不应该传递给 YourTTS（因为 YourTTS 会查找缓存的 reference_audio）
            // - 应该使用 speaker 参数（从 voice 字段映射），让 YourTTS 使用预定义的音色
            let is_multi_user_mode = translation.speaker_id.as_ref()
                .map(|s| s.starts_with("default_"))
                .unwrap_or(false);
            
            let use_speaker_id = translation.speaker_id.is_some() && !is_multi_user_mode;
            let has_reference_audio = reference_audio.is_some();
            
            // 在多人模式下，从 voice 字段获取 speaker 参数
            // 如果使用 YourTTS，需要使用 YourTTS 的预定义 speaker 名称（如 "male-en-5", "female-en-5"）
            // 而不是从 speaker_voice_mapper 获取的 voice（那是给 Piper TTS 用的）
            let is_yourtts = self.tts_service_url.as_ref()
                .map(|url| url.contains("5004") || url.contains("yourtts"))
                .unwrap_or(false);
            
            let speaker_for_request = if is_multi_user_mode {
                if is_yourtts {
                    // 多人模式 + YourTTS：使用 YourTTS 的预定义 speaker
                    Some(Self::get_yourtts_speaker_by_gender(estimated_gender.as_ref()))
                } else {
                    // 多人模式 + Piper TTS：使用从 voice 字段映射的 speaker
                    if !common_voice.is_empty() {
                        Some(common_voice.clone())
                    } else {
                        // 如果 voice 为空，根据性别选择默认音色
                        Some(Self::get_default_voice_by_gender(estimated_gender.as_ref()))
                    }
                }
            } else if !use_speaker_id && !has_reference_audio {
                if is_yourtts {
                    Some(Self::get_yourtts_speaker_by_gender(estimated_gender.as_ref()))
                } else {
                    Some(Self::get_default_speaker_by_gender(estimated_gender.as_ref()))
                }
            } else {
                None
            };
            
            let tts_request = TtsRequest {
                text: processed_text.clone(),
                voice: common_voice.clone(),
                locale: target_language.clone(),
                speaker_id: if is_multi_user_mode {
                    None  // 多人模式下不传递 speaker_id，使用 speaker 参数
                } else {
                    translation.speaker_id.clone()  // 单人模式下传递 speaker_id
                },
                reference_audio: if !use_speaker_id {
                    reference_audio.clone()  // 只有在没有 speaker_id 时才传递 reference_audio
                } else {
                    None  // 如果有 speaker_id，不传递 reference_audio（使用缓存的）
                },
                voice_embedding: if !use_speaker_id && has_reference_audio {
                    use_voice_embedding.clone()  // 只有在使用 reference_audio 时才传递 voice_embedding
                } else {
                    None
                },
                speaker: speaker_for_request,
                speech_rate,
            };
            
            // 记录日志（包含原始语速信息，用于调试）
            if let Some(rate) = speech_rate {
                eprintln!("[TTS] ⚡ Queueing segment {:2} for parallel synthesis: '{}' (speech_rate: {:.2} chars/s from source)", 
                    idx + 1, segment_text, rate);
            } else {
                eprintln!("[TTS] ⚡ Queueing segment {:2} for parallel synthesis: '{}' (⚠️  NO SPEECH_RATE - will use default)", idx + 1, segment_text);
            }
            
            // 创建异步任务：合成 + 增强
            let tts_clone = Arc::clone(&self.tts);
            let fallback_tts_clone = self.fallback_tts.as_ref().map(Arc::clone);
            let enhancer_clone = self.audio_enhancer.as_ref().map(Arc::clone);
            
            async move {
                let segment_tts_start = Instant::now();
                
                // 合成音频（带回退机制）
                let mut chunk = match tts_clone.synthesize(tts_request.clone()).await {
                    Ok(chunk) => chunk,
                    Err(e) => {
                        // 检查是否是语言不支持的错误（YourTTS 不支持中文）
                        let error_msg = e.to_string().to_lowercase();
                        let is_language_error = error_msg.contains("does not support chinese") ||
                                                error_msg.contains("language") || 
                                                error_msg.contains("not in the available languages") ||
                                                error_msg.contains("不支持") ||
                                                error_msg.contains("dict_keys") ||
                                                error_msg.contains("dimension out of range");
                        
                        if is_language_error {
                            eprintln!("[TTS] ⚠️  Segment {:2} failed due to unsupported language, trying fallback TTS...", idx + 1);
                            // 如果有回退 TTS，尝试使用它（不传递 reference_audio，因为 Piper 不支持）
                            // 但保留语速信息，确保输出语速匹配输入
                            if let Some(ref fallback) = fallback_tts_clone {
                                let mut fallback_request = tts_request.clone();
                                fallback_request.reference_audio = None;  // Piper 不支持 reference_audio
                                fallback_request.speaker_id = None;  // Piper 不支持 speaker_id
                                fallback_request.voice_embedding = None;  // Piper 不支持 voice_embedding
                                // 语速信息已保留在 tts_request 中，会传递给 Piper
                                
                                eprintln!("[TTS] ⚡ Fallback: Using Piper TTS with speech_rate={:?}", fallback_request.speech_rate);
                                
                                match fallback.synthesize(fallback_request).await {
                                    Ok(fallback_chunk) => {
                                        eprintln!("[TTS] ✅ Segment {:2} fallback TTS succeeded", idx + 1);
                                        fallback_chunk
                                    }
                                    Err(fallback_err) => {
                                        eprintln!("[TTS] ❌ Segment {:2} fallback TTS also failed: {}", idx + 1, fallback_err);
                                        return Err(fallback_err);
                                    }
                                }
                            } else {
                                eprintln!("[TTS] ❌ No fallback TTS available, segment {:2} failed", idx + 1);
                                return Err(e);
                            }
                        } else {
                            eprintln!("[TTS] ❌ Segment {:2} synthesis failed: {}", idx + 1, e);
                            return Err(e);
                        }
                    }
                };
                
                let segment_tts_ms = segment_tts_start.elapsed().as_millis() as u64;
                
                // 应用音频增强
                if let Some(ref enhancer) = enhancer_clone {
                    let pause_type = if segment_pause_type != crate::text_segmentation::PauseType::None {
                        Some(segment_pause_type)
                    } else {
                        None
                    };
                    
                    match enhancer.enhance_audio_with_pause_type(
                        &chunk.audio,
                        idx == 0,  // is_first
                        is_last,   // is_last
                        pause_type,
                    ).await {
                        Ok(enhanced_audio) => {
                            chunk.audio = enhanced_audio;
                            eprintln!("[TTS] ✅ Segment {:2} completed in {}ms: '{}' (audio_size: {} bytes)", 
                                idx + 1, segment_tts_ms, segment_text, chunk.audio.len());
                        }
                        Err(e) => {
                            eprintln!("[TTS] ⚠️  Segment {:2} enhancement failed: {}, using original audio", idx + 1, e);
                        }
                    }
                } else {
                    eprintln!("[TTS] ✅ Segment {:2} completed in {}ms: '{}' (audio_size: {} bytes)", 
                        idx + 1, segment_tts_ms, segment_text, chunk.audio.len());
                }
                
                Ok((idx, chunk, segment_tts_ms, is_last))
            }
        }).collect();
        
        // 3.4. 并行执行所有任务并等待完成
        eprintln!("[TTS] ⚡ Executing {} segments in parallel...", segment_futures.len());
        let segment_results = join_all(segment_futures).await;
        
        // 3.5. 按顺序处理结果（保持播放顺序）
        let mut ordered_chunks = Vec::new();
        let mut total_yourtts_ms = 0u64;
        let mut yourtts_call_count = 0u64;
        
        // 收集所有结果（包含索引，用于排序）
        let mut results_with_idx: Vec<_> = Vec::new();
        for result in segment_results {
            results_with_idx.push(result?);
        }
        
        // 按索引排序以确保顺序
        results_with_idx.sort_by_key(|(idx, _, _, _)| *idx);
        
        // 按顺序处理每个结果
        let mut current_timestamp = timestamp_ms;
        for (idx, mut chunk, segment_tts_ms, is_last) in results_with_idx {
            // 累计 YourTTS 耗时
            if self.tts_service_url.as_ref()
                .map(|url| url.contains("5004") || url.contains("yourtts"))
                .unwrap_or(false) {
                total_yourtts_ms += segment_tts_ms;
                yourtts_call_count += 1;
            }
            
            // 设置时间戳和 is_last 标志
            chunk.timestamp_ms = current_timestamp;
            chunk.is_last = is_last;
            
            // 立即发布（buffer_sentences == 0）
            if self.tts_buffer_sentences == 0 {
                Self::publish_tts_event(self, &chunk, current_timestamp).await?;
                eprintln!("[TTS] 📤 Published segment {:2} immediately (timestamp: {}ms)", idx + 1, current_timestamp);
            }
            
            // 保存到 ordered_chunks（用于合并和缓冲模式）
            ordered_chunks.push(chunk);
            
            // 更新时间戳
            current_timestamp += 100; // 每个短句间隔 100ms
        }
        
        // 3.6. 缓冲模式：发布剩余的短句（如果需要）
        if self.tts_buffer_sentences > 0 {
            for (idx, chunk) in ordered_chunks.iter().enumerate() {
                Self::publish_tts_event(self, chunk, chunk.timestamp_ms).await?;
                eprintln!("[TTS] 📤 Published segment {:2} from buffer (timestamp: {}ms)", idx + 1, chunk.timestamp_ms);
            }
        }
        
        // 3.7. 不合并音频，每个 segment 已经通过 publish_tts_event 独立发布
        // 这样用户体验更接近连续输出，而不是等好几秒才听到完整的话
        let tts_incremental_total_ms = tts_incremental_start.elapsed().as_millis() as u64;
        eprintln!("[TTS] ⚡ Parallel synthesis completed: {} segments in {}ms (avg: {:.1}ms/segment)", 
            ordered_chunks.len(), 
            tts_incremental_total_ms,
            if ordered_chunks.len() > 0 { tts_incremental_total_ms as f32 / ordered_chunks.len() as f32 } else { 0.0 });
        
        // 计算总音频大小（仅用于日志）
        let total_audio_size: usize = ordered_chunks.iter().map(|c| c.audio.len()).sum();
        eprintln!("[TTS] Total audio size: {} bytes ({} segments, each published independently)", 
            total_audio_size, ordered_chunks.len());
        
        // 如果使用 YourTTS，记录 YourTTS 的耗时
        let yourtts_ms = if self.tts_service_url.as_ref()
            .map(|url| url.contains("5004") || url.contains("yourtts"))
            .unwrap_or(false) {
            if yourtts_call_count > 0 {
                Some(total_yourtts_ms)
            } else {
                None
            }
        } else {
            None
        };
        
        if let Some(yt_ms) = yourtts_ms {
            eprintln!("[TTS] YourTTS total time: {}ms ({} calls, avg: {:.1}ms per call)", 
                yt_ms, yourtts_call_count, if yourtts_call_count > 0 { yt_ms as f64 / yourtts_call_count as f64 } else { 0.0 });
        }
        
        // 返回 None，因为每个 segment 已经通过事件独立发布
        // 客户端应该通过事件总线接收并按 timestamp_ms 顺序播放
        // 返回第一个 chunk 作为占位符（仅用于兼容性，实际音频通过事件发布）
        let placeholder_chunk = if let Some(first_chunk) = ordered_chunks.first() {
            // 只返回第一个 chunk 的音频（作为占位符），实际所有 chunks 已通过事件发布
            TtsStreamChunk {
                audio: first_chunk.audio.clone(),  // 只包含第一个 segment 的音频
                timestamp_ms: first_chunk.timestamp_ms,
                is_last: false,  // 标记为 false，表示还有更多 chunks 通过事件发布
            }
        } else {
            return Err(EngineError::new("No chunks to synthesize".to_string()));
        };
        
        eprintln!("[TTS] ✅ Incremental TTS completed: {} segments published independently (placeholder chunk returned for compatibility)", 
            ordered_chunks.len());
        
        // 返回占位符 chunk（实际音频已通过事件发布）
        Ok((placeholder_chunk, yourtts_ms))
    }

    // convert_decimals_to_chinese 已移至 text_utils.rs 模块
    // adjust_vad_threshold_by_feedback, apply_vad_feedback, update_vad_speech_rate, get_vad_speech_rate 已移至 vad_utils.rs 模块
    // publish_tts_event 已移至 events.rs 模块
}
