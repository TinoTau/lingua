// core/engine/src/asr_whisper/streaming.rs
// Whisper ASR 的 AsrStreaming trait 实现

use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::asr_streaming::{AsrRequest, AsrResult, AsrStreaming};
use crate::error::{EngineError, EngineResult};
use crate::types::{AudioFrame, PartialTranscript, StableTranscript};

use super::engine::WhisperAsrEngine;
use super::audio_preprocessing::preprocess_audio_frame;

/// Whisper ASR 的流式实现
/// 
/// 当前实现为基础版本：累积所有音频帧，在每次 `infer()` 调用时进行完整推理
pub struct WhisperAsrStreaming {
    engine: Arc<WhisperAsrEngine>,
    /// 音频帧缓冲区（累积所有收到的帧）
    audio_buffer: Arc<Mutex<Vec<AudioFrame>>>,
    /// 是否已初始化
    initialized: Arc<Mutex<bool>>,
}

impl WhisperAsrStreaming {
    /// 从模型路径创建 WhisperAsrStreaming
    /// 
    /// # Arguments
    /// * `model_path` - GGML 模型文件路径
    pub fn new_from_model_path(model_path: &std::path::Path) -> anyhow::Result<Self> {
        let engine = WhisperAsrEngine::new_from_model_path(model_path)?;
        
        Ok(Self {
            engine: Arc::new(engine),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            initialized: Arc::new(Mutex::new(false)),
        })
    }

    /// 从模型目录创建 WhisperAsrStreaming
    /// 
    /// # Arguments
    /// * `model_dir` - 模型目录路径
    pub fn new_from_dir(model_dir: &std::path::Path) -> anyhow::Result<Self> {
        let engine = WhisperAsrEngine::new_from_dir(model_dir)?;
        
        Ok(Self {
            engine: Arc::new(engine),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            initialized: Arc::new(Mutex::new(false)),
        })
    }

    /// 设置语言
    /// 
    /// 注意：由于 engine 是 Arc，我们需要通过内部可变性来修改
    /// 但 WhisperAsrEngine 的 set_language 需要 &mut self
    /// 这里我们暂时不实现，或者需要修改 WhisperAsrEngine 的设计
    /// 为了简化，这里先留空，后续可以改进
    #[allow(unused_variables)]
    pub fn set_language(&self, _language: Option<String>) {
        // TODO: 实现语言设置
    }

    /// 清空音频缓冲区
    pub fn clear_buffer(&self) {
        if let Ok(mut buffer) = self.audio_buffer.lock() {
            buffer.clear();
        }
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
            buffer.push(request.frame);
        }

        // 2. 获取当前缓冲区中的所有帧
        let frames = {
            let buffer = self.audio_buffer.lock()
                .map_err(|e| EngineError::new(format!("Failed to lock audio buffer: {}", e)))?;
            buffer.clone()
        };

        // 3. 如果缓冲区为空，返回空结果
        if frames.is_empty() {
            return Ok(AsrResult {
                partial: None,
                final_transcript: None,
            });
        }

        // 4. 累积所有帧并进行推理
        // 注意：当前实现是基础版本，每次 infer 都进行完整推理
        // 后续可以优化为只在检测到完整句子时才推理
        let mut audio_buffer = Vec::new();
        for frame in &frames {
            let preprocessed = preprocess_audio_frame(frame)
                .map_err(|e| EngineError::new(format!("Failed to preprocess audio frame: {}", e)))?;
            audio_buffer.extend_from_slice(&preprocessed);
        }
        let audio_data = audio_buffer;

        // 5. 运行推理
        let transcript_text = self.engine.transcribe_full(&audio_data)
            .map_err(|e| EngineError::new(format!("Failed to transcribe: {}", e)))?;

        // 6. 构造结果
        // 当前实现：每次都返回最终结果
        // 后续可以改进：返回部分结果和最终结果
        let result = if transcript_text.is_empty() {
            AsrResult {
                partial: None,
                final_transcript: None,
            }
        } else {
            // 计算置信度（简单实现：固定值，后续可以改进）
            let confidence = 0.95;

            AsrResult {
                partial: Some(PartialTranscript {
                    text: transcript_text.clone(),
                    confidence,
                    is_final: false,
                }),
                final_transcript: Some(StableTranscript {
                    text: transcript_text,
                    speaker_id: None,
                    language: self.engine.language()
                        .unwrap_or("unknown")
                        .to_string(),
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

