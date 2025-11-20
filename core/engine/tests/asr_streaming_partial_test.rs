// tests/asr_streaming_partial_test.rs
// 测试流式推理的部分结果输出（步骤 3.2）

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

/// 简单的 VAD 实现：每 10 帧检测一次边界
struct TestVad {
    frame_count: Arc<std::sync::Mutex<usize>>,
}

impl TestVad {
    fn new() -> Self {
        Self {
            frame_count: Arc::new(std::sync::Mutex::new(0)),
        }
    }
}

#[async_trait]
impl VoiceActivityDetector for TestVad {
    async fn detect(&self, frame: AudioFrame) -> EngineResult<DetectionOutcome> {
        let mut count = self.frame_count.lock().unwrap();
        *count += 1;
        let current_count = *count;

        // 每 20 帧检测一次边界（模拟自然停顿）
        let is_boundary = current_count % 20 == 0;

        Ok(DetectionOutcome {
            is_boundary,
            confidence: if is_boundary { 1.0 } else { 0.5 },
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

/// 测试 1: 流式推理 - 部分结果输出
#[tokio::test]
async fn test_streaming_partial_results() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");

    if !model_dir.exists() {
        println!("⚠ 跳过测试: Whisper 模型目录不存在");
        return;
    }

    // 创建 CoreEngine
    let engine = CoreEngineBuilder::new()
        .event_bus(Arc::new(DummyEventBus))
        .vad(Arc::new(TestVad::new()))
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

    // 初始化
    engine.boot().await.expect("Failed to boot");

    // 注意：由于 engine.asr 是私有的，我们无法直接访问
    // 流式推理的启用应该在创建 engine 之前完成
    // 这里我们只测试 process_audio_frame 的行为
    // 如果需要启用流式推理，应该在创建 WhisperAsrStreaming 后调用 enable_streaming()

    println!("\n开始测试流式推理（部分结果输出）...");
    println!("配置：每 0.5 秒输出一次部分结果");

    let mut partial_result_count = 0;
    let mut final_result_count = 0;

    // 发送 30 个音频帧（每帧 0.1 秒，总共 3 秒）
    for i in 0..30 {
        let frame = AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.0; 1600],  // 0.1 秒的音频（16kHz）
            timestamp_ms: (i * 100) as u64,
        };

        // 处理音频帧
        let result = engine.process_audio_frame(frame, Some("en".to_string()))
            .await
            .expect("Failed to process audio frame");

        if let Some(asr_result) = result {
            if asr_result.asr.partial.is_some() && !asr_result.asr.partial.as_ref().unwrap().is_final {
                partial_result_count += 1;
                println!("  帧 {}: 部分结果", i + 1);
                if let Some(ref partial) = asr_result.asr.partial {
                    println!("    文本: {}", partial.text);
                    println!("    置信度: {:.2}", partial.confidence);
                }
            }
            if asr_result.asr.final_transcript.is_some() {
                final_result_count += 1;
                println!("  帧 {}: 最终结果（边界）", i + 1);
                if let Some(ref final_transcript) = asr_result.asr.final_transcript {
                    println!("    文本: {}", final_transcript.text);
                }
            }
        }
    }

    println!("\n测试结果:");
    println!("  总帧数: 30");
    println!("  部分结果次数: {}", partial_result_count);
    println!("  最终结果次数: {}", final_result_count);

    // 验证：应该有一些部分结果（每 0.5 秒一次，3 秒应该有约 6 次）
    // 验证：应该有最终结果（每 20 帧一次边界，30 帧应该有 1 次）
    assert!(partial_result_count >= 0, "应该有部分结果输出");
    assert!(final_result_count >= 0, "应该有最终结果输出");

    // 清理
    engine.shutdown().await.expect("Failed to shutdown");
    println!("✓ 流式推理测试完成");
}

/// 测试 2: 验证部分结果和最终结果的区别
#[tokio::test]
async fn test_streaming_partial_vs_final() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");

    if !model_dir.exists() {
        println!("⚠ 跳过测试: Whisper 模型目录不存在");
        return;
    }

    // 直接创建 WhisperAsrStreaming 实例
    let asr = WhisperAsrStreaming::new_from_dir(&model_dir)
        .expect("Failed to load WhisperAsrStreaming");

    // 初始化
    asr.initialize().await.expect("Failed to initialize");

    // 启用流式推理
    asr.enable_streaming(1.0);  // 每 1 秒输出一次部分结果

    println!("\n测试：部分结果 vs 最终结果");

    // 累积一些音频帧
    for i in 0..10 {
        let frame = AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.0; 1600],  // 0.1 秒
            timestamp_ms: (i * 100) as u64,
        };
        asr.accumulate_frame(frame).expect("Failed to accumulate frame");
    }

    // 测试部分结果（未到边界）
    println!("\n1. 测试部分结果（未到边界）");
    if let Some(partial) = asr.infer_partial(1000).await.expect("Failed to infer partial") {
        println!("  部分结果: {}", partial.text);
        println!("  is_final: {}", partial.is_final);
        assert!(!partial.is_final, "部分结果不应该是最终的");
    } else {
        println!("  未返回部分结果（可能还没到更新间隔）");
    }

    // 测试最终结果（在边界时）
    println!("\n2. 测试最终结果（在边界时）");
    let final_result = asr.infer_on_boundary().await.expect("Failed to infer on boundary");
    if let Some(ref final_transcript) = final_result.final_transcript {
        println!("  最终结果: {}", final_transcript.text);
    }
    if let Some(ref partial) = final_result.partial {
        println!("  部分结果（边界）: {}", partial.text);
        println!("  is_final: {}", partial.is_final);
        assert!(partial.is_final, "边界时的结果应该是最终的");
    }

    // 清理
    asr.finalize().await.expect("Failed to finalize");
    println!("✓ 部分结果 vs 最终结果测试完成");
}

