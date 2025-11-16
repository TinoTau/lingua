use async_trait::async_trait;
use crate::error::EngineResult;
use super::types::{TranslationRequest, TranslationResponse};

#[async_trait]
pub trait NmtIncremental: Send + Sync {
    async fn initialize(&self) -> EngineResult<()>;
    async fn translate(&self, request: TranslationRequest) -> EngineResult<TranslationResponse>;
    async fn finalize(&self) -> EngineResult<()>;
}

