use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryDatum {
    pub name: String,
    pub value: f64,
    pub unit: String,
}

#[async_trait]
pub trait TelemetrySink: Send + Sync {
    async fn record(&self, datum: TelemetryDatum) -> EngineResult<()>;
}
