mod xlmr_emotion;
mod stub;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;
use crate::types::StableTranscript;

pub use xlmr_emotion::XlmREmotionEngine;
pub use stub::EmotionStub;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionRequest {
    pub transcript: StableTranscript,
    pub acoustic_features: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionResponse {
    pub label: String,
    pub confidence: f32,
}

#[async_trait]
pub trait EmotionAdapter: Send + Sync {
    async fn analyze(&self, request: EmotionRequest) -> EngineResult<EmotionResponse>;
}
