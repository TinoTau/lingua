use async_trait::async_trait;

use crate::error::EngineResult;

#[async_trait]
pub trait CacheManager: Send + Sync {
    async fn warm_up(&self) -> EngineResult<()>;
    async fn purge(&self) -> EngineResult<()>;
}
