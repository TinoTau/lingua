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
        // 返回默认的 neutral 情感
        Ok(EmotionResponse {
            label: "neutral".to_string(),
            confidence: 0.5,
        })
    }
}

