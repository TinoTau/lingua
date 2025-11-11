use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;
use crate::types::{AudioFrame, PartialTranscript, StableTranscript};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrRequest {
    pub frame: AudioFrame,
    pub language_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrResult {
    pub partial: Option<PartialTranscript>,
    pub final_transcript: Option<StableTranscript>,
}

#[async_trait]
pub trait AsrStreaming: Send + Sync {
    async fn initialize(&self) -> EngineResult<()>;
    async fn infer(&self, request: AsrRequest) -> EngineResult<AsrResult>;
    async fn finalize(&self) -> EngineResult<()>;
}
