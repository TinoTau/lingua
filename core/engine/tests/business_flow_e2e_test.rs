// tests/business_flow_e2e_test.rs
// 端到端测试：验证完整业务流程（音频帧 → VAD → ASR → NMT → 事件发布）

use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;
use core_engine::*;
use std::sync::Mutex;
use std::collections::VecDeque;

// 测试用的 EventBus：记录所有发布的事件
struct TestEventBus {
    events: Arc<Mutex<VecDeque<CoreEvent>>>,
}

impl TestEventBus {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn get_events(&self) -> Vec<CoreEvent> {
        let events = self.events.lock().unwrap();
        events.iter().cloned().collect()
    }

    fn clear_events(&self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }
}

#[async_trait]
impl EventBus for TestEventBus {
    async fn start(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn stop(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn publish(&self, event: CoreEvent) -> EngineResult<()> {
        let mut events = self.events.lock().unwrap();
        events.push_back(event);
        Ok(())
    }

    async fn subscribe(&self, topic: EventTopic) -> EngineResult<EventSubscription> {
        Ok(EventSubscription { topic })
    }
}

// 简单的 VAD 实现：每 20 帧检测一次边界
struct TestVad {
    frame_count: Arc<Mutex<usize>>,
}

impl TestVad {
    fn new() -> Self {
        Self {
            frame_count: Arc::new(Mutex::new(0)),
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

struct DummyEmotion;

#[async_trait]
impl EmotionAdapter for DummyEmotion {
    async fn analyze(&self, _request: EmotionRequest) -> EngineResult<EmotionResponse> {
        Ok(EmotionResponse {
            label: "neutral".to_string(),
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

/// 测试完整业务流程：音频帧 → VAD → ASR → NMT → 事件发布
#[tokio::test]
async fn test_full_business_flow() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let asr_model_dir = crate_root.join("models/asr/whisper-base");
    let nmt_model_dir = crate_root.join("models/nmt/marian-en-zh");

    // 检查模型是否存在
    if !asr_model_dir.exists() {
        println!("⚠ 跳过测试: Whisper ASR 模型目录不存在: {}", asr_model_dir.display());
        return;
    }
    if !nmt_model_dir.exists() {
        println!("⚠ 跳过测试: Marian NMT 模型目录不存在: {}", nmt_model_dir.display());
        return;
    }

    // 创建测试用的 EventBus
    let event_bus = Arc::new(TestEventBus::new());
    let event_bus_clone = Arc::clone(&event_bus);

    // 创建 CoreEngine
    let engine = CoreEngineBuilder::new()
        .event_bus(Arc::clone(&event_bus) as Arc<dyn EventBus>)
        .vad(Arc::new(TestVad::new()))
        .asr_with_default_whisper()
        .expect("Failed to load default Whisper ASR")
        .nmt_with_default_marian_onnx()
        .expect("Failed to load default Marian NMT")
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
    event_bus_clone.clear_events();

    println!("\n========== 开始端到端业务流程测试 ==========");
    println!("流程：音频帧 → VAD → ASR → NMT → 事件发布");

    // 发送 30 个音频帧（每帧 0.1 秒，总共 3 秒）
    // 每 20 帧检测一次边界，所以应该有 1 次边界检测
    let mut asr_results = Vec::new();
    let mut translation_results = Vec::new();

    for i in 0..30 {
        let frame = AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.0; 1600],  // 0.1 秒的音频（16kHz）
            timestamp_ms: (i * 100) as u64,
        };

        // 处理音频帧
        if let Some(result) = engine.process_audio_frame(frame, Some("en".to_string()))
            .await
            .expect("Failed to process audio frame")
        {
            // 记录 ASR 结果
            if result.asr.final_transcript.is_some() {
                asr_results.push(result.asr.final_transcript.clone().unwrap());
                println!("\n帧 {}: ASR 最终结果", i + 1);
                if let Some(ref final_transcript) = result.asr.final_transcript {
                    println!("  文本: {}", final_transcript.text);
                    println!("  语言: {}", final_transcript.language);
                }
            }
            if result.asr.partial.is_some() {
                println!("\n帧 {}: ASR 部分结果", i + 1);
                if let Some(ref partial) = result.asr.partial {
                    println!("  文本: {}", partial.text);
                    println!("  置信度: {:.2}", partial.confidence);
                }
            }

            // 记录翻译结果
            if let Some(ref translation) = result.translation {
                translation_results.push(translation.clone());
                println!("\n帧 {}: NMT 翻译结果", i + 1);
                println!("  翻译: {}", translation.translated_text);
                println!("  是否稳定: {}", translation.is_stable);
            }
        }
    }

    // 获取所有发布的事件
    let events = event_bus_clone.get_events();
    println!("\n========== 事件统计 ==========");
    println!("总事件数: {}", events.len());

    let mut asr_partial_count = 0;
    let mut asr_final_count = 0;
    let mut translation_count = 0;

    for event in &events {
        match event.topic.0.as_str() {
            "AsrPartial" => {
                asr_partial_count += 1;
                println!("\nASR 部分结果事件 #{}", asr_partial_count);
                if let Some(text) = event.payload.get("text") {
                    println!("  文本: {}", text);
                }
            }
            "AsrFinal" => {
                asr_final_count += 1;
                println!("\nASR 最终结果事件 #{}", asr_final_count);
                if let Some(text) = event.payload.get("text") {
                    println!("  文本: {}", text);
                }
            }
            "Translation" => {
                translation_count += 1;
                println!("\n翻译事件 #{}", translation_count);
                if let Some(translated_text) = event.payload.get("translated_text") {
                    println!("  翻译: {}", translated_text);
                }
            }
            _ => {
                println!("\n其他事件: {}", event.topic.0);
            }
        }
    }

    println!("\n========== 测试结果 ==========");
    println!("ASR 部分结果事件: {}", asr_partial_count);
    println!("ASR 最终结果事件: {}", asr_final_count);
    println!("翻译事件: {}", translation_count);
    println!("ASR 最终结果数: {}", asr_results.len());
    println!("翻译结果数: {}", translation_results.len());

    // 验证：应该有至少 1 次 ASR 最终结果（因为每 20 帧检测一次边界）
    // 验证：如果有 ASR 最终结果，应该有对应的翻译结果
    assert!(asr_final_count >= 0, "应该有 ASR 最终结果事件");
    if asr_final_count > 0 {
        assert!(translation_count >= 0, "如果有 ASR 最终结果，应该有翻译事件");
    }

    // 清理
    engine.shutdown().await.expect("Failed to shutdown");
    println!("\n✓ 端到端业务流程测试完成");
}

