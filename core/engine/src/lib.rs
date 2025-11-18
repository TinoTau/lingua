pub mod bootstrap;
pub mod asr_whisper;
pub mod cache_manager;
pub mod config_manager;
pub mod emotion_adapter;
pub mod error;
pub mod event_bus;
pub mod nmt_incremental;
pub mod persona_adapter;
pub mod telemetry;
pub mod tts_streaming;
pub mod types;
pub mod vad;
pub mod asr_streaming;
pub mod onnx_utils;

pub use bootstrap::{CoreEngine, CoreEngineBuilder, ProcessResult};
pub use cache_manager::CacheManager;
pub use config_manager::{ConfigManager, EngineConfig};
pub use emotion_adapter::{EmotionAdapter, EmotionRequest, EmotionResponse, XlmREmotionEngine, EmotionStub};
pub use error::{EngineError, EngineResult};
pub use event_bus::{CoreEvent, EventBus, EventSubscription, EventTopic};
pub use nmt_incremental::{
    LanguageCode, LanguagePair, MarianNmtOnnx, MarianTokenizer, NmtIncremental, TranslationRequest, TranslationResponse,
};
pub use persona_adapter::{PersonaAdapter, PersonaContext, RuleBasedPersonaAdapter, PersonaStub};
pub use telemetry::{TelemetryDatum, TelemetrySink};
pub use tts_streaming::{TtsRequest, TtsStreamChunk, TtsStreaming, FastSpeech2TtsEngine, TtsStub};
pub use types::{AudioFrame, PartialTranscript, StableTranscript};
pub use vad::{DetectionOutcome, VoiceActivityDetector};
pub use asr_streaming::{AsrRequest, AsrResult, AsrStreaming};
