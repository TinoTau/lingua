use serde::{Deserialize, Serialize};
use crate::types::PartialTranscript;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub transcript: PartialTranscript,
    pub target_language: String,
    pub wait_k: Option<u8>,
    pub speaker_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub translated_text: String,
    pub is_stable: bool,
    pub speaker_id: Option<String>,
    pub source_text: Option<String>,
    pub source_audio_duration_ms: Option<u64>,
    pub quality_metrics: Option<TranslationQualityMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationQualityMetrics {
    pub perplexity: Option<f32>,
    pub avg_probability: Option<f32>,
    pub min_probability: Option<f32>,
}

