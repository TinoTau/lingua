use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;
use crate::types::AudioFrame;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionOutcome {
    pub is_boundary: bool,
    pub confidence: f32,
    pub frame: AudioFrame,
}

#[async_trait]
pub trait VoiceActivityDetector: Send + Sync {
    async fn detect(&self, frame: AudioFrame) -> EngineResult<DetectionOutcome>;
}
