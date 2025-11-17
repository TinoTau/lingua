use async_trait::async_trait;

use crate::error::EngineResult;
use crate::types::StableTranscript;
use super::{PersonaAdapter, PersonaContext};

/// Persona 适配器的 stub 实现（用于测试和开发）
/// 
/// 直接返回原始 transcript，不做任何个性化处理
pub struct PersonaStub;

impl PersonaStub {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PersonaAdapter for PersonaStub {
    async fn personalize(
        &self,
        transcript: StableTranscript,
        _context: PersonaContext,
    ) -> EngineResult<StableTranscript> {
        // Stub 实现：直接返回原始 transcript
        Ok(transcript)
    }
}

