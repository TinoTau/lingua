use crate::asr_streaming::AsrResult;
use crate::emotion_adapter::EmotionResponse;
use crate::nmt_incremental::TranslationResponse;
use crate::tts_streaming::TtsStreamChunk;

/// 处理结果（包含 ASR、Emotion、NMT 和 TTS 结果）
#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub asr: AsrResult,
    pub emotion: Option<EmotionResponse>,
    pub translation: Option<TranslationResponse>,
    pub tts: Option<TtsStreamChunk>,
}

