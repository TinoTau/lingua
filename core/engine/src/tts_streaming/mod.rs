use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub voice: String,
    pub locale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsStreamChunk {
    pub audio: Vec<u8>,
    pub timestamp_ms: u64,
    pub is_last: bool,
}

#[async_trait]
pub trait TtsStreaming: Send + Sync {
    async fn synthesize(&self, request: TtsRequest) -> EngineResult<TtsStreamChunk>;
    async fn close(&self) -> EngineResult<()>;
}
