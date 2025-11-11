use std::sync::Arc;

use async_trait::async_trait;
use core_engine::*;

struct DummyEventBus;

#[async_trait]
impl EventBus for DummyEventBus {
    async fn start(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn stop(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn publish(&self, _event: CoreEvent) -> EngineResult<()> {
        Ok(())
    }

    async fn subscribe(&self, topic: EventTopic) -> EngineResult<EventSubscription> {
        Ok(EventSubscription { topic })
    }
}

struct DummyVad;

#[async_trait]
impl VoiceActivityDetector for DummyVad {
    async fn detect(&self, frame: AudioFrame) -> EngineResult<DetectionOutcome> {
        Ok(DetectionOutcome {
            is_boundary: true,
            confidence: 1.0,
            frame,
        })
    }
}

struct DummyAsr;

#[async_trait]
impl AsrStreaming for DummyAsr {
    async fn initialize(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn infer(&self, _request: AsrRequest) -> EngineResult<AsrResult> {
        Ok(AsrResult {
            partial: None,
            final_transcript: None,
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        Ok(())
    }
}

struct DummyNmt;

#[async_trait]
impl NmtIncremental for DummyNmt {
    async fn initialize(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn translate(&self, _request: TranslationRequest) -> EngineResult<TranslationResponse> {
        Ok(TranslationResponse {
            translated_text: String::new(),
            is_stable: true,
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        Ok(())
    }
}

struct DummyEmotion;

#[async_trait]
impl EmotionAdapter for DummyEmotion {
    async fn analyze(&self, _request: EmotionRequest) -> EngineResult<EmotionResponse> {
        Ok(EmotionResponse {
            label: "neutral".to_string(),
            confidence: 1.0,
        })
    }
}

struct DummyPersona;

#[async_trait]
impl PersonaAdapter for DummyPersona {
    async fn personalize(
        &self,
        transcript: StableTranscript,
        _context: PersonaContext,
    ) -> EngineResult<StableTranscript> {
        Ok(transcript)
    }
}

struct DummyTts;

#[async_trait]
impl TtsStreaming for DummyTts {
    async fn synthesize(&self, _request: TtsRequest) -> EngineResult<TtsStreamChunk> {
        Ok(TtsStreamChunk {
            audio: vec![],
            timestamp_ms: 0,
            is_last: true,
        })
    }

    async fn close(&self) -> EngineResult<()> {
        Ok(())
    }
}

struct DummyConfig;

#[async_trait]
impl ConfigManager for DummyConfig {
    async fn load(&self) -> EngineResult<EngineConfig> {
        Ok(EngineConfig {
            mode: "fast".to_string(),
            source_language: "en".to_string(),
            target_language: "zh".to_string(),
        })
    }

    async fn current(&self) -> EngineResult<EngineConfig> {
        self.load().await
    }
}

struct DummyCache;

#[async_trait]
impl CacheManager for DummyCache {
    async fn warm_up(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn purge(&self) -> EngineResult<()> {
        Ok(())
    }
}

struct DummyTelemetry;

#[async_trait]
impl TelemetrySink for DummyTelemetry {
    async fn record(&self, _datum: TelemetryDatum) -> EngineResult<()> {
        Ok(())
    }
}

#[tokio::test]
async fn core_engine_boot_and_shutdown() {
    let engine = CoreEngineBuilder::new()
        .event_bus(Arc::new(DummyEventBus))
        .vad(Arc::new(DummyVad))
        .asr(Arc::new(DummyAsr))
        .nmt(Arc::new(DummyNmt))
        .emotion(Arc::new(DummyEmotion))
        .persona(Arc::new(DummyPersona))
        .tts(Arc::new(DummyTts))
        .config(Arc::new(DummyConfig))
        .cache(Arc::new(DummyCache))
        .telemetry(Arc::new(DummyTelemetry))
        .build()
        .expect("builder should succeed");

    engine.boot().await.expect("boot should succeed");
    engine
        .shutdown()
        .await
        .expect("shutdown should succeed");
}
