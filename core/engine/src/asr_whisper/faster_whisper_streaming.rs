// core/engine/src/asr_whisper/faster_whisper_streaming.rs
// Faster-Whisper ASR 的流式实现（通过 HTTP 调用 Python 服务）

use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::asr_streaming::{AsrRequest, AsrResult, AsrStreaming, AsrStreamingExt};
use crate::asr_filters::is_meaningless_transcript_with_context;
use crate::asr_http_client::AsrHttpClient;
use crate::error::{EngineError, EngineResult};
use crate::types::{AudioFrame, PartialTranscript, StableTranscript};
use crate::asr_whisper::audio_preprocessing::{preprocess_audio_frame, accumulate_audio_frames};

/// 流式推理配置（基于自然停顿）
#[derive(Debug, Clone)]
struct StreamingConfig {
    /// 部分结果更新间隔（秒）
    partial_update_interval_seconds: f64,
    /// 上次部分结果更新的时间戳（毫秒）
    last_partial_update_ms: u64,
    /// 是否启用流式推理（部分结果输出）
    enabled: bool,
}

/// Faster-Whisper ASR 的流式实现（通过 HTTP 调用 Python 服务）
/// 
/// 支持三种模式：
/// 1. 基础模式：每次 `infer()` 调用时进行完整推理（当前默认）
/// 2. VAD 集成模式：使用 `accumulate_frame()` 累积帧，在 `infer_on_boundary()` 时推理
/// 3. 流式模式：使用滑动窗口定期推理，返回部分结果（步骤 3.2）
pub struct FasterWhisperAsrStreaming {
    /// HTTP 客户端
    http_client: Arc<AsrHttpClient>,
    /// 音频帧缓冲区（累积所有收到的帧）
    audio_buffer: Arc<Mutex<Vec<AudioFrame>>>,
    /// 历史音频帧缓冲区（用于说话者识别，保留最近 2-3 秒的音频）
    history_buffer: Arc<Mutex<Vec<AudioFrame>>>,
    /// 是否已初始化
    initialized: Arc<Mutex<bool>>,
    /// 流式推理配置
    streaming_config: Arc<Mutex<StreamingConfig>>,
    /// 上下文缓存（最近 2-3 句的文本，用于提供上下文参考）
    context_cache: Arc<Mutex<Vec<String>>>,
    /// 语言设置（可选）
    language: Arc<Mutex<Option<String>>>,
}

impl FasterWhisperAsrStreaming {
    /// 简单的句子分割函数（用于提取最后一句）
    /// 按句号、问号、感叹号等标点符号分割
    fn split_into_sentences_simple(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        
        for ch in text.chars() {
            current_sentence.push(ch);
            
            // 检查是否为句子结束标点
            let is_sentence_end = matches!(
                ch,
                '.' | '!' | '?' | '。' | '！' | '？'
            );
            
            if is_sentence_end {
                let trimmed = current_sentence.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current_sentence.clear();
            }
        }
        
        // 处理最后一个句子（如果没有结束标点）
        let trimmed = current_sentence.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }
        
        sentences
    }
    /// 创建新的 FasterWhisperAsrStreaming 实例
    /// 
    /// # Arguments
    /// * `service_url` - ASR 服务的 URL（例如："http://127.0.0.1:6006"）
    /// * `timeout_secs` - HTTP 请求超时时间（秒）
    pub fn new(service_url: String, timeout_secs: u64) -> Self {
        let http_client = Arc::new(AsrHttpClient::new(service_url, timeout_secs));
        
        Self {
            http_client,
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            history_buffer: Arc::new(Mutex::new(Vec::new())),
            initialized: Arc::new(Mutex::new(false)),
            streaming_config: Arc::new(Mutex::new(StreamingConfig {
                partial_update_interval_seconds: 1.0,
                last_partial_update_ms: 0,
                enabled: false,
            })),
            context_cache: Arc::new(Mutex::new(Vec::new())),
            language: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取音频缓冲区中的所有帧并预处理为音频数据
    /// 
    /// # Returns
    /// 返回预处理后的音频数据（16kHz 单声道 PCM f32）
    pub(crate) fn get_and_preprocess_audio(&self) -> EngineResult<Vec<f32>> {
        let frames = {
            let buffer = self.audio_buffer.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
            buffer.clone()
        };

        if frames.is_empty() {
            return Ok(Vec::new());
        }

        // 预处理所有累积的帧
        let audio_data = accumulate_audio_frames(&frames)
            .map_err(|e| EngineError::new(format!("Failed to preprocess audio frames: {}", e)))?;

        Ok(audio_data)
    }

    /// 将音频数据转换为 WAV 格式的字节
    /// 
    /// # Arguments
    /// * `audio_data` - 音频数据（16kHz 单声道 PCM f32）
    /// 
    /// # Returns
    /// 返回 WAV 格式的字节数据
    fn audio_to_wav_bytes(&self, audio_data: &[f32]) -> EngineResult<Vec<u8>> {
        use hound::{WavWriter, WavSpec};
        use std::io::Cursor;

        let spec = WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut buffer = Vec::new();
        {
            let mut writer = WavWriter::new(Cursor::new(&mut buffer), spec)
                .map_err(|e| EngineError::new(format!("Failed to create WAV writer: {}", e)))?;
            
            for &sample in audio_data {
                // 将 f32 (-1.0 到 1.0) 转换为 i16
                let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
                writer.write_sample(sample_i16)
                    .map_err(|e| EngineError::new(format!("Failed to write WAV sample: {}", e)))?;
            }
            
            writer.finalize()
                .map_err(|e| EngineError::new(format!("Failed to finalize WAV: {}", e)))?;
        }

        Ok(buffer)
    }

    /// 获取上下文缓存（前 2 句的文本）
    /// 
    /// # Returns
    /// 返回上下文字符串（如果缓存不为空），否则返回空字符串
    pub(crate) fn get_context_prompt(&self) -> EngineResult<String> {
        let cache = self.context_cache.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock context cache: {}", e)))?;
        
        if !cache.is_empty() {
            // 只发送最后一句作为上下文（避免重复和污染）
            // 这样既能提供上下文提高准确度，又能避免发送多句导致的重复识别
            let last_sentence = cache.last().unwrap().clone();
            let context_preview = last_sentence.chars().take(100).collect::<String>();
            
            eprintln!("[ASR] 📚 Context Cache: Found {} previous sentence(s), using last one only", cache.len());
            eprintln!("[ASR] 📚 Using context ({} chars): \"{}\"", last_sentence.len(), context_preview);
            
            Ok(last_sentence)
        } else {
            eprintln!("[ASR] 📚 Context Cache: Empty (no previous sentences)");
            Ok(String::new())
        }
    }

    /// 更新上下文缓存（添加新句子，只保留最后 1 句）
    /// 
    /// # Arguments
    /// * `text` - 要添加到缓存的文本
    /// 
    /// # Note
    /// 只保留最后 1 句，因为发送给 faster-whisper 的上下文只需要最后一句
    /// 这样可以避免缓存累积导致的重复识别问题
    pub(crate) fn update_context_cache(&self, text: &str) -> EngineResult<()> {
        let trimmed_text = text.trim();
        if trimmed_text.is_empty() {
            eprintln!("[ASR] ⚠️  Context Cache: Skipped update (empty transcript)");
            return Ok(());
        }

        let mut cache = self.context_cache.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock context cache: {}", e)))?;
        
        // 只保留最后 1 句（替换而不是追加）
        // 这样每次发送给 faster-whisper 的上下文都是最新的，不会累积重复
        cache.clear();
        cache.push(trimmed_text.to_string());
        
        eprintln!("[ASR Faster-Whisper] 💾 Context Cache: Updated (keeping only last sentence)");
        eprintln!("[ASR Faster-Whisper]   Last sentence: \"{}\"", trimmed_text.chars().take(80).collect::<String>());
        
        Ok(())
    }

    /// 在 VAD 检测到边界时进行推理
    /// 
    /// # Returns
    /// 返回 ASR 结果（包含部分结果和最终结果）
    pub async fn infer_on_boundary(&self) -> EngineResult<AsrResult> {
        eprintln!("[ASR] ==========================================");
        eprintln!("[ASR] 🚀 Starting ASR inference on boundary...");
        
        // 1. 先获取并清空缓冲区（确保即使后续失败，缓冲区也被清空）
        // 这样可以防止缓冲区累积，即使请求失败也不会导致下次处理更长的音频
        let (audio_data, frames_to_keep) = {
            let mut buffer = self.audio_buffer.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
            
            // 克隆缓冲区内容用于处理
            let frames = buffer.clone();
            
            // 立即清空缓冲区（防止累积）
            buffer.clear();
            
            drop(buffer);
            
            // 预处理音频数据
            if frames.is_empty() {
                eprintln!("[ASR] ⚠️  Audio buffer is empty, skipping inference");
                eprintln!("[ASR] ==========================================");
                return Ok(AsrResult {
                    partial: None,
                    final_transcript: None,
                });
            }
            
            let audio_data = accumulate_audio_frames(&frames)
                .map_err(|e| EngineError::new(format!("Failed to preprocess audio frames: {}", e)))?;
            
            (audio_data, frames)
        };
        
        let audio_duration_sec = audio_data.len() as f32 / 16000.0;
        eprintln!("[ASR] 📊 Preprocessed audio: {} samples ({:.2}s @ 16kHz)", 
                 audio_data.len(), audio_duration_sec);

        // 2. 获取上下文缓存（用于 faster-whisper 和过滤判断）
        // 注意：上下文可以提高识别准确度，但需要确保缓存不被污染
        let context_prompt = self.get_context_prompt()?;
        let context_for_filter = context_prompt.clone(); // 克隆用于后续过滤判断
        
        // 3. 将音频转换为 WAV 字节
        let wav_bytes = self.audio_to_wav_bytes(&audio_data)?;
        eprintln!("[ASR] 📦 Converted audio to WAV: {} bytes (sending to Faster-Whisper service)", wav_bytes.len());
        
        // 4. 获取语言设置
        let language = {
            let lang = self.language.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock language: {}", e)))?;
            lang.clone()
        };
        
        // 5. 调用 HTTP 服务进行转录
        let asr_response = self.http_client.transcribe(
            wav_bytes,
            context_prompt,
            language,
        ).await.map_err(|e| {
            eprintln!("[ASR] ❌ HTTP request failed: {}", e);
            // 注意：缓冲区已经在步骤1中清空了，这里不需要再次清空
            e
        })?;
        
        // 6. 处理识别结果
        let transcript_text = asr_response.text.trim().to_string();
        eprintln!("[ASR] ✅ Transcription completed: {} segment(s)", asr_response.segments.len());
        if asr_response.segments.len() > 1 {
            for (i, seg) in asr_response.segments.iter().enumerate() {
                eprintln!("[ASR]   Segment {}: \"{}\"", i + 1, seg.chars().take(80).collect::<String>());
            }
        }
        eprintln!("[ASR] 📝 Final transcript: \"{}\"", transcript_text.chars().take(100).collect::<String>());
        if let Some(ref lang) = asr_response.language {
            eprintln!("[ASR] 🌐 Detected language: {}", lang);
        }

        // 7. 更新上下文缓存（只更新有意义的文本）
        // 关键：只存储和传递最后一句，而不是完整的识别结果
        // faster-whisper 的 initial_prompt 应该只包含"上一条语句"，而不是"上一次的完整识别结果"
        // 使用带上下文的过滤函数，对"谢谢大家"、"感谢观看"等感谢语进行上下文判断
        if !is_meaningless_transcript_with_context(&transcript_text, &context_for_filter) {
            // 从识别结果中提取最后一句（如果包含多个句子）
            let last_sentence = if asr_response.segments.len() > 1 {
                // 如果有多个 segments，使用最后一个 segment
                asr_response.segments.last().unwrap().clone()
            } else {
                // 如果只有一个 segment，尝试按句子分割，取最后一句
                let sentences = Self::split_into_sentences_simple(&transcript_text);
                if sentences.len() > 1 {
                    sentences.last().unwrap().clone()
                } else {
                    transcript_text.trim().to_string()
                }
            };
            
            // 检查是否与当前缓存的内容相同
            let should_update = {
                let cache = self.context_cache.lock()
                    .map_err(|e| EngineError::new(format!("Failed to lock context cache: {}", e)))?;
                
                // 如果缓存为空，直接添加
                if cache.is_empty() {
                    true
                } else {
                    // 只检查是否与最后一句完全相同
                    let last_sentence_trimmed = last_sentence.trim();
                    let cached_sentence = cache.last().unwrap().trim();
                    last_sentence_trimmed.to_lowercase() != cached_sentence.to_lowercase()
                }
            };
            
            if should_update {
                self.update_context_cache(&last_sentence)?;
            } else {
                eprintln!("[ASR] ⚠️  Context Cache: Skipped update (duplicate text: \"{}\")", 
                         last_sentence.chars().take(50).collect::<String>());
            }
        } else {
            eprintln!("[ASR] ⚠️  Context Cache: Skipped update (meaningless text: \"{}\")", 
                     transcript_text.chars().take(50).collect::<String>());
        }

        // 8. 将已处理的帧添加到历史缓冲区（用于上下文）
        {
            let mut history = self.history_buffer.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock history buffer: {}", e)))?;
            history.extend(frames_to_keep);
            
            // 只保留最近 2-3 秒的音频（假设 16kHz，约 32000-48000 样本）
            let max_samples = 48000;
            let mut total_samples = 0;
            let mut keep_from = 0;
            for (i, frame) in history.iter().rev().enumerate() {
                total_samples += frame.data.len();
                if total_samples > max_samples {
                    keep_from = history.len() - i;
                    break;
                }
            }
            if keep_from > 0 {
                history.drain(0..keep_from);
            }
        }
        
        // 注意：缓冲区已经在步骤1中清空了，这里不需要再次清空
        
        eprintln!("[ASR] ✅ ASR inference completed successfully");
        eprintln!("[ASR] ==========================================");

        // 11. 构造结果
        if transcript_text.is_empty() {
            return Ok(AsrResult {
                partial: None,
                final_transcript: None,
            });
        }

        let result = AsrResult {
            partial: Some(PartialTranscript {
                text: transcript_text.clone(),
                confidence: 0.95,  // faster-whisper 不直接提供置信度，使用默认值
                is_final: true,
            }),
            final_transcript: Some(StableTranscript {
                text: transcript_text,
                speaker_id: None,
                language: asr_response.language.unwrap_or_else(|| "unknown".to_string()),
            }),
        };

        Ok(result)
    }

    /// 获取累积的音频帧（用于说话者识别等）
    pub fn get_accumulated_frames(&self) -> EngineResult<Vec<AudioFrame>> {
        let buffer = self.audio_buffer.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
        Ok(buffer.clone())
    }

    /// 设置语言
    /// 
    /// # Arguments
    /// * `language` - 语言代码（如 "en", "zh", "ja"），`None` 表示自动检测
    pub fn set_language(&self, language: Option<String>) -> EngineResult<()> {
        let mut lang = self.language.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock language: {}", e)))?;
        *lang = language;
        Ok(())
    }

    /// 获取当前设置的语言
    pub fn get_language(&self) -> EngineResult<Option<String>> {
        let lang = self.language.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock language: {}", e)))?;
        Ok(lang.clone())
    }

    /// 累积音频帧到缓冲区（用于 VAD 集成模式）
    /// 
    /// # Arguments
    /// * `frame` - 音频帧
    pub fn accumulate_frame(&self, frame: AudioFrame) -> EngineResult<()> {
        let mut buffer = self.audio_buffer.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
        buffer.push(frame);
        Ok(())
    }
    
    /// 清空音频缓冲区
    pub fn clear_buffer(&self) -> EngineResult<()> {
        let mut buffer = self.audio_buffer.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
        buffer.clear();
        Ok(())
    }
}

#[async_trait]
impl AsrStreaming for FasterWhisperAsrStreaming {
    async fn initialize(&self) -> EngineResult<()> {
        // 检查服务健康状态（在锁之外，避免跨越 await）
        eprintln!("[ASR] 🔍 Checking Faster-Whisper service health...");
        match self.http_client.health_check().await {
            Ok(true) => {
                eprintln!("[ASR] ✅ Service health check passed (Faster-Whisper)");
            }
            Ok(false) => {
                eprintln!("[ASR] ⚠️  Service health check returned false (Faster-Whisper)");
                eprintln!("[ASR] ⚠️  Please ensure the ASR service is running on the configured port");
            }
            Err(e) => {
                eprintln!("[ASR] ⚠️  Service health check failed: {} (Faster-Whisper)", e);
                eprintln!("[ASR] ⚠️  Please ensure the ASR service is running. Check:");
                eprintln!("[ASR]    1. Is the Python ASR service started? (port 6006 by default)");
                eprintln!("[ASR]    2. Is the service URL correct? (check ASR_SERVICE_URL env var)");
                eprintln!("[ASR]    3. Is the model loaded? (check ASR service logs)");
                eprintln!("[ASR] ⚠️  Continuing anyway, but ASR requests may fail...");
            }
        }
        
        // 在 await 之后设置初始化标志
        let mut initialized = self.initialized.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock initialized flag: {}", e)))?;
        *initialized = true;
        Ok(())
    }

    async fn infer(&self, request: AsrRequest) -> EngineResult<AsrResult> {
        // 1. 将新的音频帧添加到缓冲区
        {
            let mut buffer = self.audio_buffer.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
            buffer.push(request.frame.clone());
        }

        // 2. 检查是否启用流式推理
        let config = {
            let config_guard = self.streaming_config.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock streaming config: {}", e)))?;
            config_guard.clone()
        };

        if !config.enabled {
            // 流式推理未启用，返回空结果
            return Ok(AsrResult {
                partial: None,
                final_transcript: None,
            });
        }

        // 3. 检查是否应该输出部分结果
        let current_timestamp_ms = request.frame.timestamp_ms;
        let should_update = current_timestamp_ms.saturating_sub(config.last_partial_update_ms)
            >= (config.partial_update_interval_seconds * 1000.0) as u64;

        if !should_update {
            return Ok(AsrResult {
                partial: None,
                final_transcript: None,
            });
        }

        // 4. 更新上次更新时间
        {
            let mut config_guard = self.streaming_config.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock streaming config: {}", e)))?;
            config_guard.last_partial_update_ms = current_timestamp_ms;
        }

        // 5. 执行推理（这里可以调用 infer_on_boundary 的逻辑，但返回部分结果）
        // 注意：流式推理的部分结果输出需要更复杂的实现，这里简化处理
        Ok(AsrResult {
            partial: None,
            final_transcript: None,
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        // 清空缓冲区
        self.clear_buffer()?;
        Ok(())
    }
}

#[async_trait]
impl AsrStreamingExt for FasterWhisperAsrStreaming {
    fn accumulate_frame(&self, frame: AudioFrame) -> EngineResult<()> {
        FasterWhisperAsrStreaming::accumulate_frame(self, frame)
    }

    fn get_accumulated_frames(&self) -> EngineResult<Vec<AudioFrame>> {
        FasterWhisperAsrStreaming::get_accumulated_frames(self)
    }

    fn clear_buffer(&self) -> EngineResult<()> {
        FasterWhisperAsrStreaming::clear_buffer(self)
    }

    fn set_language(&self, language: Option<String>) -> EngineResult<()> {
        FasterWhisperAsrStreaming::set_language(self, language)
    }

    fn get_language(&self) -> EngineResult<Option<String>> {
        FasterWhisperAsrStreaming::get_language(self)
    }

    async fn infer_on_boundary(&self) -> EngineResult<AsrResult> {
        FasterWhisperAsrStreaming::infer_on_boundary(self).await
    }
}


