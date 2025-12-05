//! YourTTS HTTP 客户端实现
//! 
//! 通过 HTTP 请求调用 Python YourTTS 服务进行 zero-shot TTS 合成
//! 
//! 注意：服务可以在 Windows 或 WSL2 中运行
//! - 如果在 WSL2 中运行，需要设置 host 为 0.0.0.0 以允许从 Windows 访问
//! - WSL2 会自动将端口映射到 Windows localhost，客户端连接 127.0.0.1:5004 即可

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{EngineError, EngineResult};
use crate::tts_streaming::{TtsRequest, TtsStreamChunk, TtsStreaming};

/// YourTTS HTTP 服务配置
#[derive(Debug, Clone)]
pub struct YourTtsHttpConfig {
    /// HTTP 服务端点（例如：http://127.0.0.1:5004）
    pub endpoint: String,
    /// 请求超时时间（毫秒）
    pub timeout_ms: u64,
}

impl Default for YourTtsHttpConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:5004".to_string(),
            timeout_ms: 30000,  // YourTTS 可能需要更长时间（从10秒增加到30秒）
        }
    }
}

/// YourTTS HTTP 客户端
pub struct YourTtsHttp {
    client: reqwest::Client,
    config: YourTtsHttpConfig,
}

impl YourTtsHttp {
    /// 创建新的 YourTTS HTTP 客户端
    pub fn new(config: YourTtsHttpConfig) -> EngineResult<Self> {
        let timeout = Duration::from_millis(config.timeout_ms);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| EngineError::new(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// 使用默认配置创建客户端
    pub fn with_default_config() -> EngineResult<Self> {
        Self::new(YourTtsHttpConfig::default())
    }
    
    /// 将 PCM 16-bit 音频数据编码为 WAV 格式
    /// 
    /// # Arguments
    /// * `pcm_data` - PCM 16-bit 音频数据（小端字节序）
    /// * `sample_rate` - 采样率（Hz）
    /// * `channels` - 声道数（1 = mono, 2 = stereo）
    fn encode_pcm_to_wav(
        pcm_data: &[u8],
        sample_rate: u32,
        channels: u16,
    ) -> EngineResult<Vec<u8>> {
        let mut wav = Vec::new();
        
        // RIFF header
        wav.extend_from_slice(b"RIFF");
        let file_size = 36 + pcm_data.len() as u32;
        wav.extend_from_slice(&file_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        
        // fmt chunk
        wav.extend_from_slice(b"fmt ");
        let fmt_chunk_size = 16u32;
        wav.extend_from_slice(&fmt_chunk_size.to_le_bytes());
        let audio_format = 1u16; // PCM
        wav.extend_from_slice(&audio_format.to_le_bytes());
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        let byte_rate = sample_rate as u32 * channels as u32 * 2; // 16-bit = 2 bytes
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        let block_align = channels * 2; // 16-bit = 2 bytes
        wav.extend_from_slice(&block_align.to_le_bytes());
        let bits_per_sample = 16u16;
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());
        
        // data chunk
        wav.extend_from_slice(b"data");
        let data_size = pcm_data.len() as u32;
        wav.extend_from_slice(&data_size.to_le_bytes());
        wav.extend_from_slice(pcm_data);
        
        Ok(wav)
    }

    /// 异步注册说话者到 YourTTS 服务
    /// 
    /// 当识别到新说话者时，调用此方法将 reference_audio 注册到服务端缓存。
    /// 后续合成请求只需传递 speaker_id 即可使用缓存的 reference_audio。
    /// 
    /// # Arguments
    /// * `speaker_id` - 说话者ID
    /// * `reference_audio` - 参考音频数据（f32 数组）
    /// * `reference_sample_rate` - 参考音频采样率（默认 16000 Hz）
    /// * `voice_embedding` - 可选的音色embedding
    pub async fn register_speaker(
        &self,
        speaker_id: String,
        reference_audio: Vec<f32>,
        reference_sample_rate: u32,
        voice_embedding: Option<Vec<f32>>,
    ) -> EngineResult<()> {
        eprintln!("[YourTTS] Registering speaker '{}' (async, {} samples @ {} Hz)", 
                 speaker_id, reference_audio.len(), reference_sample_rate);
        
        let request = RegisterSpeakerRequest {
            speaker_id: speaker_id.clone(),
            reference_audio,
            reference_sample_rate,
            voice_embedding,
        };
        
        // 使用较长的超时时间，因为这是异步操作，不阻塞主流程
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| EngineError::new(format!("Failed to create HTTP client: {}", e)))?;
        
        let url = format!("{}/register_speaker", self.config.endpoint);
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                eprintln!("[YourTTS] ⚠️  Failed to register speaker '{}': {} (async registration, non-blocking)", speaker_id, e);
                EngineError::new(format!("HTTP request failed: {}", e))
            })?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("[YourTTS] ⚠️  Failed to register speaker '{}': status {}: {} (async registration, non-blocking)", 
                     speaker_id, status, error_text);
            return Err(EngineError::new(format!(
                "HTTP request failed with status {}: {}",
                status, error_text
            )));
        }
        
        let result: RegisterSpeakerResponse = response
            .json()
            .await
            .map_err(|e| {
                eprintln!("[YourTTS] ⚠️  Failed to parse register speaker response for '{}': {} (async registration, non-blocking)", speaker_id, e);
                EngineError::new(format!("Failed to parse response: {}", e))
            })?;
        
        eprintln!("[YourTTS] ✅ Speaker '{}' registered successfully (cache size: {})", 
                 result.speaker_id, result.cache_size);
        
        Ok(())
    }
}

/// YourTTS HTTP 服务请求体
#[derive(Debug, Serialize)]
struct YourTtsHttpRequest {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    speaker_id: Option<String>,  // 说话者ID（用于查找缓存的 reference_audio）
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_audio: Option<Vec<f32>>,  // 用于 zero-shot TTS（如果没有提供 speaker_id）
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_sample_rate: Option<u32>,  // 参考音频采样率（默认 16000 Hz）
    #[serde(skip_serializing_if = "Option::is_none")]
    voice_embedding: Option<Vec<f32>>,  // 说话者音色embedding（用于音色相似度比较）
    #[serde(skip_serializing_if = "Option::is_none")]
    speaker: Option<String>,  // 说话者名称（当没有 reference_audio 时使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    /// 语速（字符/秒，用于调整合成速度，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    speech_rate: Option<f32>,
}

/// 注册说话者的请求体
#[derive(Debug, Serialize)]
struct RegisterSpeakerRequest {
    speaker_id: String,
    reference_audio: Vec<f32>,
    reference_sample_rate: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    voice_embedding: Option<Vec<f32>>,
}

/// 注册说话者的响应
#[derive(Debug, Deserialize)]
struct RegisterSpeakerResponse {
    status: String,
    speaker_id: String,
    message: String,
    cache_size: usize,
}

/// YourTTS HTTP 服务响应
#[derive(Debug, Deserialize)]
struct YourTtsHttpResponse {
    audio: Vec<f32>,
    sample_rate: u32,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    used_reference: Option<bool>,
    #[serde(default)]
    speaker_applied: Option<bool>,  // 指示音色是否被应用
}

#[async_trait]
impl TtsStreaming for YourTtsHttp {
    async fn synthesize(&self, request: TtsRequest) -> EngineResult<TtsStreamChunk> {
        use std::time::Instant;
        let tts_start = Instant::now();
        eprintln!("[YourTTS] ===== TTS Request Started =====");
        // 安全截取字符串：使用字符边界而不是字节边界
        let text_preview = if request.text.chars().count() > 50 {
            request.text.chars().take(50).collect::<String>()
        } else {
            request.text.clone()
        };
        eprintln!("[YourTTS] Text: '{}' (locale={})", text_preview, request.locale);
        
        // 检查文本是否包含中文字符
        let contains_chinese = request.text.chars().any(|c| {
            let code = c as u32;
            (0x4E00..=0x9FFF).contains(&code) ||  // CJK Unified Ideographs
            (0x3400..=0x4DBF).contains(&code) ||  // CJK Extension A
            (0x20000..=0x2A6DF).contains(&code)   // CJK Extension B
        });
        
        // 如果文本包含中文，但：
        // 1. 没有 reference_audio（zero-shot TTS）
        // 2. 没有 speaker_id（查找缓存的 reference_audio）
        // 3. 语言设置为 "en"（多人模式回退）
        // 则 YourTTS 无法处理，应该返回错误
        if contains_chinese {
            let has_reference_audio = request.reference_audio.is_some();
            let has_speaker_id = request.speaker_id.is_some();
            let is_multi_user_mode = request.speaker_id.as_ref()
                .map(|s| s.starts_with("default_"))
                .unwrap_or(false);
            
            if is_multi_user_mode || (!has_reference_audio && !has_speaker_id) {
                return Err(EngineError::new(format!(
                    "YourTTS does not support Chinese text. Text contains Chinese characters but no reference audio is available for zero-shot TTS. Please use a TTS service that supports Chinese (e.g., Piper TTS)."
                )));
            }
        }
        
        // 准备请求体
        // 优先使用 speaker_id（查找服务端缓存的 reference_audio）
        // 如果没有 speaker_id，使用 reference_audio 进行 zero-shot TTS
        // 如果都没有，使用 speaker 参数（从 voice 字段映射）
        // 参考音频默认采样率是 16000 Hz（从 ASR/VAD 来的），需要重采样到 22050 Hz
        
        // 如果提供了 speaker_id，优先使用它（查找缓存的 reference_audio）
        // 否则，如果有 reference_audio，使用它
        // 最后，如果都没有，使用 speaker 参数
        let use_speaker_id = request.speaker_id.is_some();
        let use_reference_audio = request.reference_audio.is_some() && !use_speaker_id;
        
        let http_request = YourTtsHttpRequest {
            text: request.text.clone(),
            speaker_id: request.speaker_id.clone(),  // 传递 speaker_id（用于查找缓存的 reference_audio）
            reference_audio: if use_reference_audio {
                request.reference_audio.clone()
            } else {
                None  // 如果使用 speaker_id，不传递 reference_audio
            },
            reference_sample_rate: if use_reference_audio {
                Some(16000)  // 参考音频来自 ASR/VAD，采样率是 16000 Hz
            } else {
                None
            },
            voice_embedding: if use_reference_audio {
                request.voice_embedding.clone()  // 只有在使用 reference_audio 时才传递 voice_embedding
            } else {
                None
            },
            speaker: if !use_speaker_id && !use_reference_audio {
                // 只有在没有 speaker_id 和 reference_audio 时才使用 speaker 参数
                request.speaker.clone().or_else(|| {
                    if !request.voice.is_empty() {
                        Some(request.voice.clone())
                    } else {
                        None
                    }
                })
            } else {
                None
            },
            // 多人模式下，如果使用 default_male/default_female，且目标语言是中文，
            // YourTTS 不支持中文，需要将语言设置为 None 或使用支持的语言
            // 注意：YourTTS 模型只支持 en, fr-fr, pt-br
            language: if request.speaker_id.as_ref().map(|s| s.starts_with("default_")).unwrap_or(false) 
                && request.locale.starts_with("zh") {
                // 多人模式 + 中文：YourTTS 不支持，设置为 None 让服务端使用默认处理
                // 或者设置为 "en" 作为回退
                eprintln!("[YourTTS] ⚠️  Multi-user mode with Chinese target language detected. YourTTS doesn't support Chinese, using 'en' as fallback");
                Some("en".to_string())
            } else {
                Some(request.locale.clone())
            },
            speech_rate: request.speech_rate,
        };
        
        // 记录语速参数传递情况（用于调试）
        if let Some(rate) = request.speech_rate {
            eprintln!("[YourTTS] ✅ Using speech rate: {:.2} chars/s", rate);
        } else {
            eprintln!("[YourTTS] ⚠️  No speech_rate in TtsRequest (will not be sent to service)");
        }
        
        if let Some(ref sid) = request.speaker_id {
            eprintln!("[YourTTS] Using cached speaker mode: speaker_id='{}' (will lookup cached reference_audio)", sid);
        } else if request.reference_audio.is_some() {
            eprintln!("[YourTTS] Using zero-shot mode with reference audio ({} samples)", 
                     request.reference_audio.as_ref().map(|a| a.len()).unwrap_or(0));
            if request.voice_embedding.is_some() {
                eprintln!("[YourTTS] Voice embedding provided ({} dims) - will be used for similarity comparison", 
                         request.voice_embedding.as_ref().map(|e| e.len()).unwrap_or(0));
            } else {
                eprintln!("[YourTTS] ⚠️  No voice embedding provided - service will try to extract from Speaker Embedding service");
            }
        } else if let Some(ref speaker) = http_request.speaker {
            eprintln!("[YourTTS] Using speaker mode: '{}' (no reference audio)", speaker);
        } else {
            eprintln!("[YourTTS] Using default mode (no reference audio, no speaker)");
        }
        
        eprintln!("[YourTTS] Sending request to: {}/synthesize", self.config.endpoint);
        
        let response = self.client
            .post(&format!("{}/synthesize", self.config.endpoint))
            .json(&http_request)
            .send()
            .await
            .map_err(|e| EngineError::new(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("[YourTTS] Request failed with status {}: {}", status, error_text);
            return Err(EngineError::new(format!(
                "HTTP request failed with status {}: {}",
                status, error_text
            )));
        }

        let result: YourTtsHttpResponse = response
            .json()
            .await
            .map_err(|e| EngineError::new(format!("Failed to parse response: {}", e)))?;

        let tts_ms = tts_start.elapsed().as_millis() as u64;
        eprintln!("[YourTTS] Synthesis completed in {}ms", tts_ms);
        eprintln!("[YourTTS] Audio length: {} samples ({}s at {}Hz)", 
                  result.audio.len(), 
                  result.audio.len() as f32 / result.sample_rate as f32,
                  result.sample_rate);
        
        // 检查服务端是否真的使用了参考音频和音色
        if let Some(used_ref) = result.used_reference {
            if used_ref {
                eprintln!("[YourTTS] ✅ Service confirmed: Reference audio was used for zero-shot synthesis");
            } else {
                eprintln!("[YourTTS] ⚠️  Service confirmed: Reference audio was NOT used (fallback to default voice)");
            }
        } else {
            eprintln!("[YourTTS] ⚠️  Service did not report whether reference audio was used");
        }
        
        if let Some(speaker_applied) = result.speaker_applied {
            if speaker_applied {
                eprintln!("[YourTTS] ✅ Service confirmed: Speaker voice was successfully applied to output");
            } else {
                eprintln!("[YourTTS] ⚠️  Service confirmed: Speaker voice was NOT applied to output");
            }
        }
        
        // 转换 f32 音频数据为 i16 PCM（16-bit）
        // YourTTS 返回的是 f32 格式，需要转换为 PCM
        let pcm_samples: Vec<i16> = result.audio
            .iter()
            .map(|&sample| {
                // 将 f32 (-1.0 到 1.0) 转换为 i16
                (sample.clamp(-1.0, 1.0) * 32767.0) as i16
            })
            .collect();
        
        // 将 PCM 样本转换为字节数组（小端字节序）
        let pcm_bytes: Vec<u8> = pcm_samples
            .iter()
            .flat_map(|&sample| sample.to_le_bytes().to_vec())
            .collect();
        
        // 将 PCM 数据包装成 WAV 格式（添加 WAV 头）
        // 这样音频增强器才能正确解析
        let wav_audio = Self::encode_pcm_to_wav(
            &pcm_bytes,
            result.sample_rate,
            1, // 单声道
        )?;

        Ok(TtsStreamChunk {
            audio: wav_audio,
            timestamp_ms: 0,  // TODO: 使用实际时间戳
            is_last: true,
        })
    }

    async fn close(&self) -> EngineResult<()> {
        // HTTP 客户端不需要关闭操作
        Ok(())
    }
}

