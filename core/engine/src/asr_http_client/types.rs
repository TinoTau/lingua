// core/engine/src/asr_http_client/types.rs
// Types for ASR HTTP client

use serde::{Deserialize, Serialize};

/// Request to ASR service
#[derive(Debug, Clone, Serialize)]
pub struct AsrHttpRequest {
    /// Base64 encoded audio (WAV format, 16kHz mono)
    pub audio_b64: String,
    /// Context prompt (previous sentences)
    pub prompt: String,
    /// Language code (e.g., "zh", "en"), None for auto-detect
    pub language: Option<String>,
    /// Task type: "transcribe" or "translate"
    pub task: String,
    /// Beam size for decoding
    pub beam_size: i32,
    /// Enable VAD filtering
    pub vad_filter: bool,
    /// Use context for better accuracy
    pub condition_on_previous_text: bool,
}

impl Default for AsrHttpRequest {
    fn default() -> Self {
        Self {
            audio_b64: String::new(),
            prompt: String::new(),
            language: None,
            task: "transcribe".to_string(),
            beam_size: 5,
            vad_filter: false,  // VAD is handled by Silero VAD, disable here to avoid double filtering
            condition_on_previous_text: true,
        }
    }
}

/// Response from ASR service
#[derive(Debug, Clone, Deserialize)]
pub struct AsrHttpResponse {
    /// Full transcribed text
    pub text: String,
    /// List of segment texts
    pub segments: Vec<String>,
    /// Detected language
    pub language: Option<String>,
    /// Audio duration in seconds
    pub duration: f32,
}

