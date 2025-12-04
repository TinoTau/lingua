mod fastspeech2_tts;
mod vits_tts;
mod vits_zh_aishell3_tokenizer;
mod text_processor;
mod audio_utils;
mod stub;
mod piper_http;
pub mod yourtts_http;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;

pub use fastspeech2_tts::FastSpeech2TtsEngine;
pub use vits_tts::VitsTtsEngine;
pub use stub::TtsStub;
pub use text_processor::TextProcessor;
pub use audio_utils::{save_pcm_to_wav, validate_pcm_audio};
pub use piper_http::{PiperHttpTts, PiperHttpConfig};
pub use yourtts_http::{YourTtsHttp, YourTtsHttpConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub voice: String,
    pub locale: String,
    pub speaker_id: Option<String>,
    pub reference_audio: Option<Vec<f32>>,
    pub voice_embedding: Option<Vec<f32>>,
    pub speaker: Option<String>,
    pub speech_rate: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsStreamChunk {
    pub audio: Vec<u8>,
    pub timestamp_ms: u64,
    pub is_last: bool,
}

#[async_trait]
pub trait TtsStreaming: Send + Sync {
    async fn synthesize(&self, request: TtsRequest) -> EngineResult<TtsStreamChunk>;
    async fn close(&self) -> EngineResult<()>;
}
