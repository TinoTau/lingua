use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub mode: String,
    pub source_language: String,
    pub target_language: String,
}

#[async_trait]
pub trait ConfigManager: Send + Sync {
    async fn load(&self) -> EngineResult<EngineConfig>;
    async fn current(&self) -> EngineResult<EngineConfig>;
}
