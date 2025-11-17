use async_trait::async_trait;

use crate::error::EngineResult;
use super::{EmotionAdapter, EmotionRequest, EmotionResponse};

/// Emotion 适配器的 stub 实现（用于测试和开发）
pub struct EmotionStub;

impl EmotionStub {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EmotionAdapter for EmotionStub {
    async fn analyze(&self, _request: EmotionRequest) -> EngineResult<EmotionResponse> {
        // 返回默认的 neutral 情感（根据 Emotion_Adapter_Spec.md）
        Ok(EmotionResponse {
            primary: "neutral".to_string(),
            intensity: 0.0,
            confidence: 0.5,
        })
    }
}

