//! 事件发布相关功能
//! 
//! 包含 ASR、NMT、TTS、Emotion 等事件发布方法

use serde_json::json;

use crate::error::EngineResult;
use crate::event_bus::{CoreEvent, EventTopic};
use crate::types::{PartialTranscript, StableTranscript};
use crate::tts_streaming::TtsStreamChunk;
use crate::emotion_adapter::EmotionResponse;
use crate::nmt_incremental::TranslationResponse;

use super::core::CoreEngine;

impl CoreEngine {
    /// 发布 ASR 部分结果事件
    pub(crate) async fn publish_asr_partial_event(
        &self,
        partial: &PartialTranscript,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("AsrPartial".to_string()),
            payload: json!({
                "text": partial.text,
                "confidence": partial.confidence,
                "is_final": partial.is_final,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }

    /// 发布 ASR 最终结果事件
    pub(crate) async fn publish_asr_final_event(
        &self,
        transcript: &StableTranscript,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("AsrFinal".to_string()),
            payload: json!({
                "text": transcript.text,
                "speaker_id": transcript.speaker_id,
                "language": transcript.language,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }

    /// 发布 TTS 事件
    pub(crate) async fn publish_tts_event(
        &self,
        tts_chunk: &TtsStreamChunk,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("Tts".to_string()),
            payload: json!({
                "audio_length": tts_chunk.audio.len(),
                "timestamp_ms": tts_chunk.timestamp_ms,
                "is_last": tts_chunk.is_last,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }

    /// 发布 Emotion 事件
    pub(crate) async fn publish_emotion_event(
        &self,
        emotion: &EmotionResponse,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("Emotion".to_string()),
            payload: json!({
                "primary": emotion.primary,
                "intensity": emotion.intensity,
                "confidence": emotion.confidence,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }

    /// 发布翻译事件
    pub(crate) async fn publish_translation_event(
        &self,
        translation: &TranslationResponse,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("Translation".to_string()),
            payload: json!({
                "translated_text": translation.translated_text,
                "is_stable": translation.is_stable,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }
}

