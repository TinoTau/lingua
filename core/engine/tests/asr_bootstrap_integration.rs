// tests/asr_bootstrap_integration.rs
// 测试 Whisper ASR 与 CoreEngineBuilder 的集成

use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;
use core_engine::*;
use core_engine::asr_whisper::WhisperAsrStreaming;

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

struct DummyNmt;

#[async_trait]
impl NmtIncremental for DummyNmt {
    async fn initialize(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn translate(&self, _request: TranslationRequest) -> EngineResult<TranslationResponse> {
        Ok(TranslationResponse {
            translated_text: "dummy translation".to_string(),
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
            primary: "neutral".to_string(),
            intensity: 0.0,
            confidence: 0.5,
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
            mode: "test".to_string(),
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


/// 测试 1: 使用默认 Whisper ASR 创建 CoreEngine
#[tokio::test]
async fn test_core_engine_with_default_whisper() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");

    if !model_dir.exists() {
        println!("⚠ 跳过测试: Whisper 模型目录不存在");
        return;
    }

    // 创建 CoreEngine，使用默认 Whisper ASR
    let engine = CoreEngineBuilder::new()
        .event_bus(Arc::new(DummyEventBus))
        .vad(Arc::new(DummyVad))
        .asr_with_default_whisper()
        .expect("Failed to load default Whisper ASR")
        .nmt(Arc::new(DummyNmt))
        .emotion(Arc::new(DummyEmotion))
        .persona(Arc::new(DummyPersona))
        .tts(Arc::new(DummyTts))
        .config(Arc::new(DummyConfig))
        .cache(Arc::new(DummyCache))
        .telemetry(Arc::new(DummyTelemetry))
        .build()
        .expect("Failed to build CoreEngine");

    println!("✓ CoreEngine 创建成功（使用默认 Whisper ASR）");

    // 测试 boot 和 shutdown
    engine.boot().await.expect("Failed to boot");
    println!("✓ CoreEngine boot 成功");

    engine.shutdown().await.expect("Failed to shutdown");
    println!("✓ CoreEngine shutdown 成功");
}

/// 测试 2: 测试 Whisper ASR 的实际功能（直接测试 WhisperAsrStreaming）
#[tokio::test]
async fn test_whisper_asr_functionality() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");

    let model_dir = crate_root.join("models/asr/whisper-base");
    let wav_path = project_root.join("third_party/whisper.cpp/samples/jfk.wav");

    if !model_dir.exists() || !wav_path.exists() {
        println!("⚠ 跳过测试: 模型或音频文件不存在");
        return;
    }

    // 直接创建 WhisperAsrStreaming 实例（与 CoreEngineBuilder 使用相同的方式）
    let asr = WhisperAsrStreaming::new_from_dir(&model_dir)
        .expect("Failed to load WhisperAsrStreaming");

    // 初始化 ASR
    asr.initialize().await.expect("Failed to initialize ASR");

    // 加载音频
    let mut reader = hound::WavReader::open(&wav_path)
        .expect("Failed to open WAV file");
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().collect::<Result<Vec<_>, _>>()
                .expect("Failed to read samples")
        }
        hound::SampleFormat::Int => {
            let max_val = (1i32 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>()
                .map(|s| s.map(|sample| sample as f32 / max_val))
                .collect::<Result<Vec<_>, _>>()
                .expect("Failed to read samples")
        }
    };

    // 创建 AudioFrame
    let frame = AudioFrame {
        sample_rate: spec.sample_rate,
        channels: spec.channels as u8,
        data: samples,
        timestamp_ms: 0,
    };

    // 创建请求
    let request = AsrRequest {
        frame,
        language_hint: Some("en".to_string()),
    };

    // 运行推理
    println!("\n开始 ASR 推理...");
    let result = asr.infer(request).await
        .expect("Failed to infer");

    println!("\nASR 推理结果:");
    if let Some(ref partial) = result.partial {
        println!("  部分结果: {}", partial.text);
    }
    if let Some(ref final_transcript) = result.final_transcript {
        println!("  最终结果: {}", final_transcript.text);
    }

    // 验证结果
    assert!(result.partial.is_some() || result.final_transcript.is_some(),
            "应该返回部分或最终结果");

    // 清理
    asr.finalize().await.expect("Failed to finalize ASR");
    println!("✓ ASR 功能测试完成");
}

/// 测试 3: 测试模型目录不存在时的错误处理
#[tokio::test]
async fn test_asr_with_default_whisper_error_handling() {
    // 这个测试需要临时修改模型路径，或者使用一个不存在的路径
    // 为了简化，我们只测试错误消息的格式
    // 注意：如果模型存在，这个测试会通过 builder，所以我们需要检查实际情况
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");

    if !model_dir.exists() {
        // 模型不存在，测试错误处理
        let result = CoreEngineBuilder::new()
            .asr_with_default_whisper();

        assert!(result.is_err(), "模型不存在时应该返回错误");
        // 使用 match 来提取错误消息
        match result {
            Err(e) => {
                let error_msg = e.to_string();
                assert!(error_msg.contains("Whisper ASR model directory not found") ||
                        error_msg.contains("Failed to load WhisperAsrStreaming"),
                        "错误消息应该包含相关信息: {}", error_msg);
                println!("✓ 错误处理正常: {}", error_msg);
            }
            Ok(_) => {
                panic!("模型不存在时应该返回错误");
            }
        }
    } else {
        println!("⚠ 模型存在，跳过错误处理测试（这是正常的）");
    }
}

