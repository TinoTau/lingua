// core/engine/src/asr_streaming/ext.rs
// Extension trait for ASR streaming implementations

use async_trait::async_trait;
use crate::error::EngineResult;
use crate::types::AudioFrame;
use crate::asr_streaming::AsrResult;

/// Extension trait for ASR streaming implementations that support
/// frame accumulation and boundary-based inference
#[async_trait]
pub trait AsrStreamingExt: Send + Sync {
    /// Accumulate an audio frame to the buffer
    fn accumulate_frame(&self, frame: AudioFrame) -> EngineResult<()>;
    
    /// Get accumulated frames (for speaker identification, etc.)
    fn get_accumulated_frames(&self) -> EngineResult<Vec<AudioFrame>>;
    
    /// Clear the audio buffer
    fn clear_buffer(&self) -> EngineResult<()>;
    
    /// Set the language for ASR
    fn set_language(&self, language: Option<String>) -> EngineResult<()>;
    
    /// Get the current language setting
    fn get_language(&self) -> EngineResult<Option<String>>;
    
    /// Infer on boundary (when VAD detects a speech boundary)
    async fn infer_on_boundary(&self) -> EngineResult<AsrResult>;
}

