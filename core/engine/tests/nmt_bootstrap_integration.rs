// tests/nmt_bootstrap_integration.rs
// 测试 bootstrap.rs 中使用真实的 MarianNmtOnnx 的业务功能
// 
// ⚠️ 已废弃：此测试使用 ONNX decoder，已不再使用。
// 当前系统已切换为 Python NMT 服务（HTTP 调用）。

use std::sync::Arc;
use async_trait::async_trait;
use core_engine::*;

// 使用测试用的 Dummy 实现
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

struct DummyEmotion;

#[async_trait]
impl EmotionAdapter for DummyEmotion {
    async fn analyze(&self, _request: EmotionRequest) -> EngineResult<EmotionResponse> {
        Ok(EmotionResponse {
            primary: "neutral".to_string(),
            intensity: 0.0,
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

/// 测试 1: CoreEngine 使用真实的 MarianNmtOnnx 能否正常启动和关闭
#[tokio::test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
async fn test_core_engine_with_real_nmt_boot_shutdown() {
    println!("\n========== Test 1: CoreEngine Boot/Shutdown with Real NMT ==========");
    
    let engine = CoreEngineBuilder::new()
        .event_bus(Arc::new(DummyEventBus))
        .vad(Arc::new(DummyVad))
        .asr(Arc::new(DummyAsr))
        .nmt_with_default_marian_onnx()
        .expect("Failed to load real MarianNmtOnnx")
        .emotion(Arc::new(DummyEmotion))
        .persona(Arc::new(DummyPersona))
        .tts(Arc::new(DummyTts))
        .config(Arc::new(DummyConfig))
        .cache(Arc::new(DummyCache))
        .telemetry(Arc::new(DummyTelemetry))
        .build()
        .expect("Failed to build CoreEngine");

    println!("✓ CoreEngine built successfully with real MarianNmtOnnx");

    // 测试启动
    engine.boot().await.expect("Failed to boot CoreEngine");
    println!("✓ CoreEngine booted successfully");

    // 测试关闭
    engine.shutdown().await.expect("Failed to shutdown CoreEngine");
    println!("✓ CoreEngine shutdown successfully");
}

/// 测试 2: 直接测试 MarianNmtOnnx 的 NmtIncremental trait 实现
#[tokio::test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
async fn test_marian_nmt_onnx_trait_implementation() {
    println!("\n========== Test 2: MarianNmtOnnx NmtIncremental Trait ==========");
    
    use std::path::PathBuf;
    use core_engine::nmt_incremental::MarianNmtOnnx;
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    // 加载模型
    let nmt: Arc<dyn NmtIncremental> = Arc::new(
        MarianNmtOnnx::new_from_dir(&model_dir)
            .expect("Failed to load real MarianNmtOnnx")
    );

    // 初始化 NMT
    nmt.initialize().await.expect("Failed to initialize NMT");
    println!("✓ NMT initialized");

    // 测试翻译
    let test_cases = vec![
        "Hello",
        "Hello world",
        "How are you",
    ];

    for source_text in test_cases {
        println!("\n--- Translating: '{}' ---", source_text);
        
        let request = TranslationRequest {
            transcript: PartialTranscript {
                text: source_text.to_string(),
                confidence: 1.0,
                is_final: true,
            },
            target_language: "zh".to_string(),
            wait_k: None,
        };

        match nmt.translate(request).await {
            Ok(response) => {
                println!("✓ Translation successful: '{}' -> '{}'", source_text, response.translated_text);
                assert!(!response.translated_text.is_empty(), "Translation should not be empty");
            }
            Err(e) => {
                println!("✗ Translation failed: {}", e);
                panic!("Translation failed for '{}': {}", source_text, e);
            }
        }
    }

    // 清理
    nmt.finalize().await.expect("Failed to finalize NMT");
    println!("\n✓ All translations completed successfully");
}

/// 测试 3: 测试完整的业务流程（模拟 ASR → NMT → Emotion → Persona → TTS 链）
#[tokio::test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
async fn test_nmt_integration_full_pipeline() {
    println!("\n========== Test 3: NMT Integration Full Pipeline ==========");
    
    use std::path::PathBuf;
    use core_engine::nmt_incremental::MarianNmtOnnx;
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    // 加载真实的 NMT 模型
    let nmt: Arc<dyn NmtIncremental> = Arc::new(
        MarianNmtOnnx::new_from_dir(&model_dir)
            .expect("Failed to load real MarianNmtOnnx")
    );

    // 初始化 NMT
    nmt.initialize().await.expect("Failed to initialize NMT");
    println!("✓ NMT initialized");

    // 模拟 ASR 输出（这里我们直接构造一个转录结果）
    let asr_transcript = PartialTranscript {
        text: "Hello world".to_string(),
        confidence: 0.95,
        is_final: true,
    };

    println!("\n--- Step 1: ASR Output ---");
    println!("  Text: '{}'", asr_transcript.text);
    println!("  Confidence: {:.2}", asr_transcript.confidence);

    // 通过 NMT 翻译
    println!("\n--- Step 2: NMT Translation ---");
    let translation_request = TranslationRequest {
        transcript: asr_transcript.clone(),
        target_language: "zh".to_string(),
        wait_k: None,
    };

    let translation_response = nmt.translate(translation_request).await
        .expect("Failed to translate");
    
    println!("  Source: '{}'", asr_transcript.text);
    println!("  Target: '{}'", translation_response.translated_text);
    println!("  Is Stable: {}", translation_response.is_stable);
    assert!(!translation_response.translated_text.is_empty(), "Translation should not be empty");

    // 通过 Emotion 分析（使用 Dummy 实现）
    println!("\n--- Step 3: Emotion Analysis ---");
    let emotion = Arc::new(DummyEmotion);
    let emotion_request = EmotionRequest {
        text: translation_response.translated_text.clone(),
        lang: "zh".to_string(),
    };

    let emotion_response = emotion.analyze(emotion_request).await
        .expect("Failed to analyze emotion");
    
    println!("  Primary: '{}'", emotion_response.primary);
    println!("  Confidence: {:.2}", emotion_response.confidence);

    // 通过 Persona 个性化（使用 Dummy 实现）
    println!("\n--- Step 4: Persona Personalization ---");
    let persona = Arc::new(DummyPersona);
    let persona_context = PersonaContext {
        user_id: "test_user".to_string(),
        tone: "formal".to_string(),
        culture: "zh-CN".to_string(),
    };

    let persona_transcript = persona.personalize(
        StableTranscript {
            text: translation_response.translated_text.clone(),
            speaker_id: None,
            language: "zh".to_string(),
        },
        persona_context,
    ).await.expect("Failed to personalize");

    println!("  Personalized Text: '{}'", persona_transcript.text);

    // 通过 TTS 合成（使用 Dummy 实现）
    println!("\n--- Step 5: TTS Synthesis ---");
    let tts = Arc::new(DummyTts);
    let tts_request = TtsRequest {
        text: persona_transcript.text,
        voice: "default".to_string(),
        locale: "zh-CN".to_string(),
    };

    let _tts_chunk = tts.synthesize(tts_request).await
        .expect("Failed to synthesize TTS");
    
    println!("  TTS Synthesis completed");

    // 清理
    nmt.finalize().await.expect("Failed to finalize NMT");
    
    println!("\n✓ Full pipeline test completed successfully");
    println!("  Summary: ASR → NMT (real) → Emotion (dummy) → Persona (dummy) → TTS (dummy)");
}

