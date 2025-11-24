// core/engine/src/asr_whisper/streaming.rs
// Whisper ASR 的 AsrStreaming trait 实现

use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use anyhow::anyhow;

use crate::asr_streaming::{AsrRequest, AsrResult, AsrStreaming};
use crate::error::{EngineError, EngineResult};
use crate::types::{AudioFrame, PartialTranscript, StableTranscript};

use super::engine::WhisperAsrEngine;
use super::audio_preprocessing::preprocess_audio_frame;

/// 流式推理配置（基于自然停顿）
#[derive(Debug, Clone)]
struct StreamingConfig {
    /// 部分结果更新间隔（秒）- 在用户说话过程中，每隔多久输出一次部分结果
    partial_update_interval_seconds: f64,
    /// 上次部分结果更新的时间戳（毫秒）
    last_partial_update_ms: u64,
    /// 是否启用流式推理（部分结果输出）
    enabled: bool,
}

/// Whisper ASR 的流式实现
/// 
/// 支持三种模式：
/// 1. 基础模式：每次 `infer()` 调用时进行完整推理（当前默认）
/// 2. VAD 集成模式：使用 `accumulate_frame()` 累积帧，在 `infer_on_boundary()` 时推理
/// 3. 流式模式：使用滑动窗口定期推理，返回部分结果（步骤 3.2）
pub struct WhisperAsrStreaming {
    engine: Arc<Mutex<WhisperAsrEngine>>,  // 使用 Mutex 以支持内部可变性（语言设置）
    /// 音频帧缓冲区（累积所有收到的帧）
    audio_buffer: Arc<Mutex<Vec<AudioFrame>>>,
    /// 是否已初始化
    initialized: Arc<Mutex<bool>>,
    /// 流式推理配置
    streaming_config: Arc<Mutex<StreamingConfig>>,
}

impl WhisperAsrStreaming {
    /// 从模型路径创建 WhisperAsrStreaming
    /// 
    /// # Arguments
    /// * `model_path` - GGML 模型文件路径
    pub fn new_from_model_path(model_path: &std::path::Path) -> anyhow::Result<Self> {
        let engine = WhisperAsrEngine::new_from_model_path(model_path)?;
        
        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            initialized: Arc::new(Mutex::new(false)),
            streaming_config: Arc::new(Mutex::new(StreamingConfig {
                partial_update_interval_seconds: 1.0,  // 每 1 秒更新部分结果
                last_partial_update_ms: 0,
                enabled: false,  // 默认禁用，需要显式启用
            })),
        })
    }

    /// 从模型目录创建 WhisperAsrStreaming
    /// 
    /// # Arguments
    /// * `model_dir` - 模型目录路径
    pub fn new_from_dir(model_dir: &std::path::Path) -> anyhow::Result<Self> {
        let engine = WhisperAsrEngine::new_from_dir(model_dir)?;
        
        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            initialized: Arc::new(Mutex::new(false)),
            streaming_config: Arc::new(Mutex::new(StreamingConfig {
                partial_update_interval_seconds: 1.0,  // 每 1 秒更新部分结果
                last_partial_update_ms: 0,
                enabled: false,  // 默认禁用，需要显式启用
            })),
        })
    }

    /// 设置语言
    /// 
    /// # Arguments
    /// * `language` - 语言代码（如 "en", "zh", "ja"），`None` 表示自动检测
    /// 
    /// # Examples
    /// ```
    /// asr.set_language(Some("en".to_string()));  // 设置为英语
    /// asr.set_language(Some("zh".to_string()));  // 设置为中文
    /// asr.set_language(None);                     // 自动检测
    /// ```
    pub fn set_language(&self, language: Option<String>) -> EngineResult<()> {
        let mut engine = self.engine.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock WhisperAsrEngine: {}", e)))?;
        engine.set_language(language);
        Ok(())
    }

    /// 启用流式推理模式（部分结果输出）
    /// 
    /// # Arguments
    /// * `partial_update_interval_seconds` - 部分结果更新间隔（秒），在用户说话过程中每隔多久输出一次部分结果
    pub fn enable_streaming(&self, partial_update_interval_seconds: f64) {
        if let Ok(mut config) = self.streaming_config.lock() {
            config.enabled = true;
            config.partial_update_interval_seconds = partial_update_interval_seconds;
        }
    }

    /// 禁用流式推理模式
    pub fn disable_streaming(&self) {
        if let Ok(mut config) = self.streaming_config.lock() {
            config.enabled = false;
        }
    }

    /// 检查是否启用流式推理
    pub fn is_streaming_enabled(&self) -> bool {
        if let Ok(config) = self.streaming_config.lock() {
            config.enabled
        } else {
            false
        }
    }

    /// 清空音频缓冲区
    pub fn clear_buffer(&self) {
        if let Ok(mut buffer) = self.audio_buffer.lock() {
            buffer.clear();
        }
    }

    /// 只累积音频帧，不进行推理
    /// 
    /// # Arguments
    /// * `frame` - 音频帧
    /// 
    /// # Returns
    /// 返回累积的帧数
    pub fn accumulate_frame(&self, frame: AudioFrame) -> EngineResult<usize> {
        let mut buffer = self.audio_buffer.lock()
            .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
        buffer.push(frame);
        Ok(buffer.len())
    }

    /// 流式推理：基于自然停顿，定期输出部分结果
    /// 
    /// 在用户说话过程中，每隔一定时间（partial_update_interval_seconds）输出一次部分结果
    /// 最终结果在 `infer_on_boundary()` 中返回（检测到自然停顿时）
    /// 
    /// # Arguments
    /// * `current_timestamp_ms` - 当前时间戳（毫秒）
    /// 
    /// # Returns
    /// 返回部分结果（如果到了更新间隔），否则返回 None
    pub async fn infer_partial(&self, current_timestamp_ms: u64) -> EngineResult<Option<PartialTranscript>> {
        let config = {
            let config_guard = self.streaming_config.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock streaming config: {}", e)))?;
            config_guard.clone()
        };

        if !config.enabled {
            // 流式推理未启用，返回 None
            return Ok(None);
        }

        // 1. 检查是否需要更新部分结果
        let should_update_partial = current_timestamp_ms >= config.last_partial_update_ms + 
            (config.partial_update_interval_seconds * 1000.0) as u64;

        if !should_update_partial {
            // 还没到更新间隔，返回 None
            return Ok(None);
        }

        // 2. 更新最后更新时间
        {
            let mut config_guard = self.streaming_config.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock streaming config: {}", e)))?;
            config_guard.last_partial_update_ms = current_timestamp_ms;
        }

        // 3. 获取当前缓冲区中的所有帧（累积的所有音频）
        let frames = {
            let buffer = self.audio_buffer.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
            buffer.clone()
        };

        if frames.is_empty() {
            return Ok(None);
        }

        // 4. 预处理所有累积的帧（不使用滑动窗口，使用所有累积的音频）
        let mut audio_buffer = Vec::new();
        for frame in &frames {
            let preprocessed = preprocess_audio_frame(frame)
                .map_err(|e| EngineError::new(format!("Failed to preprocess audio frame: {}", e)))?;
            audio_buffer.extend_from_slice(&preprocessed);
        }
        let audio_data = audio_buffer;

        // 5. 运行推理（使用 spawn_blocking 避免阻塞异步运行时）
        let engine_clone = Arc::clone(&self.engine);
        let audio_data_clone = audio_data.clone();
        let (transcript_text, _detected_lang) = tokio::task::spawn_blocking(move || {
            let engine = engine_clone.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock WhisperAsrEngine: {}", e))?;
            engine.transcribe_full(&audio_data_clone)
                .map_err(|e| anyhow::anyhow!("Failed to transcribe: {}", e))
        })
        .await
        .map_err(|e| EngineError::new(format!("Task join error: {}", e)))?
        .map_err(|e| EngineError::new(format!("Transcription error: {}", e)))?;

        // 6. 构造部分结果
        if transcript_text.is_empty() {
            Ok(None)
        } else {
            let confidence = 0.90;  // 部分结果的置信度稍低

            Ok(Some(PartialTranscript {
                text: transcript_text,
                confidence,
                is_final: false,  // 部分结果不是最终的
            }))
        }
    }

    /// 在检测到语音边界时触发推理
    /// 
    /// # Returns
    /// 返回推理结果，如果缓冲区为空则返回空结果
    pub async fn infer_on_boundary(&self) -> EngineResult<AsrResult> {
        // 1. 获取当前缓冲区中的所有帧
        let frames = {
            let buffer = self.audio_buffer.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
            buffer.clone()
        };

        // 2. 如果缓冲区为空，返回空结果
        if frames.is_empty() {
            return Ok(AsrResult {
                partial: None,
                final_transcript: None,
            });
        }

        // 3. 预处理所有累积的帧（不使用滑动窗口，使用所有累积的音频）
        let mut audio_buffer = Vec::new();
        for frame in &frames {
            let preprocessed = preprocess_audio_frame(frame)
                .map_err(|e| EngineError::new(format!("Failed to preprocess audio frame: {}", e)))?;
            audio_buffer.extend_from_slice(&preprocessed);
        }
        let audio_data = audio_buffer;

        // 6. 运行推理（使用 spawn_blocking 避免阻塞异步运行时）
        let engine_clone = Arc::clone(&self.engine);
        let audio_data_clone = audio_data.clone();
        let (transcript_text, _detected_lang) = tokio::task::spawn_blocking(move || {
            let engine = engine_clone.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock WhisperAsrEngine: {}", e))?;
            engine.transcribe_full(&audio_data_clone)
                .map_err(|e| anyhow::anyhow!("Failed to transcribe: {}", e))
        })
        .await
        .map_err(|e| EngineError::new(format!("Task join error: {}", e)))?
        .map_err(|e| EngineError::new(format!("Transcription error: {}", e)))?;

        // 7. 清空缓冲区（因为已经推理完成）
        self.clear_buffer();

        // 8. 重置流式推理配置（如果启用）
        {
            let config_guard = self.streaming_config.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock streaming config: {}", e)))?;
            if config_guard.enabled {
                drop(config_guard);
                if let Ok(mut config_guard) = self.streaming_config.lock() {
                    config_guard.last_partial_update_ms = 0;
                }
            }
        }

        // 9. 构造结果
        let result = if transcript_text.is_empty() {
            AsrResult {
                partial: None,
                final_transcript: None,
            }
        } else {
            let confidence = 0.95;

            AsrResult {
                partial: Some(PartialTranscript {
                    text: transcript_text.clone(),
                    confidence,
                    is_final: true,  // 在边界时，结果应该是最终的
                }),
                final_transcript: {
                    let engine = self.engine.lock()
                        .map_err(|e| EngineError::new(format!("Failed to lock WhisperAsrEngine: {}", e)))?;
                    Some(StableTranscript {
                        text: transcript_text,
                        speaker_id: None,
                        language: engine.language()
                            .unwrap_or("unknown")
                            .to_string(),
                    })
                },
            }
        };

        Ok(result)
    }
}

#[async_trait]
impl AsrStreaming for WhisperAsrStreaming {
    async fn initialize(&self) -> EngineResult<()> {
        // 模型已在创建时加载，这里只需要标记为已初始化
        if let Ok(mut initialized) = self.initialized.lock() {
            *initialized = true;
        }
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

        // 3. 如果启用流式推理，检查是否需要输出部分结果
        if config.enabled {
            // 尝试获取部分结果
            if let Some(partial) = self.infer_partial(request.frame.timestamp_ms).await? {
                return Ok(AsrResult {
                    partial: Some(partial),
                    final_transcript: None,  // 部分结果不包含最终结果
                });
            }
            // 如果还没到更新间隔，返回空结果（继续累积）
            return Ok(AsrResult {
                partial: None,
                final_transcript: None,
            });
        }

        // 4. 否则，使用基础模式：累积所有帧并进行完整推理
        let frames = {
            let buffer = self.audio_buffer.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
            buffer.clone()
        };

        if frames.is_empty() {
            return Ok(AsrResult {
                partial: None,
                final_transcript: None,
            });
        }

        // 5. 预处理所有累积的帧
        let mut audio_buffer = Vec::new();
        for frame in &frames {
            let preprocessed = preprocess_audio_frame(frame)
                .map_err(|e| EngineError::new(format!("Failed to preprocess audio frame: {}", e)))?;
            audio_buffer.extend_from_slice(&preprocessed);
        }
        let audio_data = audio_buffer;

        // 6. 运行推理（使用 spawn_blocking 避免阻塞异步运行时）
        let engine_clone = Arc::clone(&self.engine);
        let audio_data_clone = audio_data.clone();
        let (transcript_text, detected_lang) = tokio::task::spawn_blocking(move || {
            let engine = engine_clone.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock WhisperAsrEngine: {}", e))?;
            engine.transcribe_full(&audio_data_clone)
                .map_err(|e| anyhow::anyhow!("Failed to transcribe: {}", e))
        })
        .await
        .map_err(|e| EngineError::new(format!("Task join error: {}", e)))?
        .map_err(|e| EngineError::new(format!("Transcription error: {}", e)))?;

        // 8. 构造结果
        let result = if transcript_text.is_empty() {
            AsrResult {
                partial: None,
                final_transcript: None,
            }
        } else {
            let confidence = 0.95;
            
            // 使用检测到的语言，如果没有则使用设置的语言，最后使用 "unknown"
            let final_language = detected_lang
                .or_else(|| {
                    let engine = self.engine.lock().ok()?;
                    engine.language().map(|s| s.to_string())
                })
                .unwrap_or_else(|| "unknown".to_string());

            AsrResult {
                partial: Some(PartialTranscript {
                    text: transcript_text.clone(),
                    confidence,
                    is_final: false,
                }),
                final_transcript: Some(StableTranscript {
                    text: transcript_text,
                    speaker_id: None,
                    language: final_language,
                }),
            }
        };

        Ok(result)
    }

    async fn finalize(&self) -> EngineResult<()> {
        // 清空缓冲区
        self.clear_buffer();
        
        // 标记为未初始化
        if let Ok(mut initialized) = self.initialized.lock() {
            *initialized = false;
        }

        Ok(())
    }
}

