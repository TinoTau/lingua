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
    pub text: String,
    pub lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionResponse {
    pub primary: String,      // 主要情绪: "neutral" | "joy" | "sadness" | "anger" | "fear" | "surprise"
    pub intensity: f32,       // 情绪强度: 0.0 - 1.0
    pub confidence: f32,      // 置信度: 0.0 - 1.0
}

#[async_trait]
pub trait EmotionAdapter: Send + Sync {
    async fn analyze(&self, request: EmotionRequest) -> EngineResult<EmotionResponse>;
}
