use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFrame {
    pub sample_rate: u32,
    pub channels: u8,
    pub data: Vec<f32>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialTranscript {
    pub text: String,
    pub confidence: f32,
    pub is_final: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableTranscript {
    pub text: String,
    pub speaker_id: Option<String>,
    pub language: String,
}
