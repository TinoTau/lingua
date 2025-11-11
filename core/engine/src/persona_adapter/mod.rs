use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;
use crate::types::StableTranscript;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaContext {
    pub user_id: String,
    pub tone: String,
    pub culture: String,
}

#[async_trait]
pub trait PersonaAdapter: Send + Sync {
    async fn personalize(
        &self,
        transcript: StableTranscript,
        context: PersonaContext,
    ) -> EngineResult<StableTranscript>;
}
