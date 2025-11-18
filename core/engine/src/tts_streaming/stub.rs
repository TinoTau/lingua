use async_trait::async_trait;

use crate::error::EngineResult;
use super::{TtsRequest, TtsStreamChunk, TtsStreaming};

/// TTS 适配器的 stub 实现（用于测试和开发）
pub struct TtsStub;

impl TtsStub {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TtsStreaming for TtsStub {
    async fn synthesize(&self, _request: TtsRequest) -> EngineResult<TtsStreamChunk> {
        // Stub 实现：返回空音频
        Ok(TtsStreamChunk {
            audio: vec![],
            timestamp_ms: 0,
            is_last: true,
        })
    }

    async fn close(&self) -> EngineResult<()> {
        Ok(())
    }
}

