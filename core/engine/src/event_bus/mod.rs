mod channel;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;

pub use channel::ChannelEventBus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreEvent {
    pub topic: EventTopic,
    pub payload: serde_json::Value,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EventTopic(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    pub topic: EventTopic,
}

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn start(&self) -> EngineResult<()>;
    async fn stop(&self) -> EngineResult<()>;
    async fn publish(&self, event: CoreEvent) -> EngineResult<()>;
    async fn subscribe(&self, topic: EventTopic) -> EngineResult<EventSubscription>;
}
