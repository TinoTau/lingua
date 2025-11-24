// tests/business_flow_step_by_step_test.rs
// 分步骤测试业务流程，逐步验证每个组件

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

/// 步骤 1: 测试 VAD 是否正常工作
#[tokio::test]
async fn test_step1_vad() {
    println!("\n========== 步骤 1: 测试 VAD ==========");
    
    let vad = Arc::new(TestVad::new());
    
    for i in 0..5 {
        let frame = AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.0; 1600],
            timestamp_ms: (i * 100) as u64,
        };
        
        let result = vad.detect(frame).await.expect("VAD 检测失败");
        println!("帧 {}: is_boundary = {}", i + 1, result.is_boundary);
    }
    
    println!("✓ VAD 测试通过");
}

/// 步骤 2: 测试 ASR 初始化
#[tokio::test]
async fn test_step2_asr_init() {
    println!("\n========== 步骤 2: 测试 ASR 初始化 ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let asr_model_dir = crate_root.join("models/asr/whisper-base");
    
    if !asr_model_dir.exists() {
        println!("⚠ 跳过测试: Whisper ASR 模型目录不存在: {}", asr_model_dir.display());
        return;
    }
    
    let asr = core_engine::asr_whisper::WhisperAsrStreaming::new_from_dir(&asr_model_dir)
        .expect("Failed to create WhisperAsrStreaming");
    
    asr.initialize().await.expect("Failed to initialize ASR");
    
    println!("✓ ASR 初始化成功");
}

/// 步骤 3: 测试 ASR 累积帧（不推理）
#[tokio::test]
async fn test_step3_asr_accumulate() {
    println!("\n========== 步骤 3: 测试 ASR 累积帧 ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let asr_model_dir = crate_root.join("models/asr/whisper-base");
    
    if !asr_model_dir.exists() {
        println!("⚠ 跳过测试: Whisper ASR 模型目录不存在: {}", asr_model_dir.display());
        return;
    }
    
    let asr = core_engine::asr_whisper::WhisperAsrStreaming::new_from_dir(&asr_model_dir)
        .expect("Failed to create WhisperAsrStreaming");
    
    asr.initialize().await.expect("Failed to initialize ASR");
    
    // 累积 5 帧
    for i in 0..5 {
        let frame = AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.0; 1600],
            timestamp_ms: (i * 100) as u64,
        };
        
        let count = asr.accumulate_frame(frame).expect("Failed to accumulate frame");
        println!("帧 {}: 累积帧数 = {}", i + 1, count);
    }
    
    println!("✓ ASR 累积帧测试通过");
}

/// 步骤 4: 测试 NMT 初始化
/// ⚠️ 已废弃：此测试使用 ONNX decoder，已不再使用。
#[tokio::test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
async fn test_step4_nmt_init() {
    println!("\n========== 步骤 4: 测试 NMT 初始化 ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let nmt_model_dir = crate_root.join("models/nmt/marian-en-zh");
    
    if !nmt_model_dir.exists() {
        println!("⚠ 跳过测试: Marian NMT 模型目录不存在: {}", nmt_model_dir.display());
        return;
    }
    
    let nmt = core_engine::nmt_incremental::MarianNmtOnnx::new_from_dir(&nmt_model_dir)
        .expect("Failed to create MarianNmtOnnx");
    
    nmt.initialize().await.expect("Failed to initialize NMT");
    
    println!("✓ NMT 初始化成功");
}

/// 步骤 5: 测试 CoreEngine 构建
#[tokio::test]
async fn test_step5_core_engine_build() {
    println!("\n========== 步骤 5: 测试 CoreEngine 构建 ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let asr_model_dir = crate_root.join("models/asr/whisper-base");
    let nmt_model_dir = crate_root.join("models/nmt/marian-en-zh");
    
    if !asr_model_dir.exists() || !nmt_model_dir.exists() {
        println!("⚠ 跳过测试: 模型目录不存在");
        return;
    }
    
    let event_bus = Arc::new(TestEventBus::new());
    
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
    
    println!("✓ CoreEngine 构建成功");
    
    // 测试 boot
    engine.boot().await.expect("Failed to boot");
    println!("✓ CoreEngine boot 成功");
    
    // 测试 shutdown
    engine.shutdown().await.expect("Failed to shutdown");
    println!("✓ CoreEngine shutdown 成功");
}

/// 步骤 6: 测试处理单个音频帧（不触发推理）
#[tokio::test]
async fn test_step6_process_single_frame() {
    println!("\n========== 步骤 6: 测试处理单个音频帧 ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let asr_model_dir = crate_root.join("models/asr/whisper-base");
    let nmt_model_dir = crate_root.join("models/nmt/marian-en-zh");
    
    if !asr_model_dir.exists() || !nmt_model_dir.exists() {
        println!("⚠ 跳过测试: 模型目录不存在");
        return;
    }
    
    let event_bus = Arc::new(TestEventBus::new());
    
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
    
    engine.boot().await.expect("Failed to boot");
    
    // 处理 1 个音频帧（不会触发边界，所以不会推理）
    let frame = AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: vec![0.0; 1600],
        timestamp_ms: 0,
    };
    
    use tokio::time::{timeout, Duration};
    let result = timeout(
        Duration::from_secs(5),
        engine.process_audio_frame(frame, Some("en".to_string()))
    ).await;
    
    match result {
        Ok(Ok(None)) => {
            println!("✓ 处理单个音频帧成功（无结果，正常）");
        }
        Ok(Ok(Some(_))) => {
            println!("✓ 处理单个音频帧成功（有结果）");
        }
        Ok(Err(e)) => {
            println!("✗ 处理音频帧时出错: {}", e);
            panic!("处理音频帧失败");
        }
        Err(_) => {
            println!("✗ 处理音频帧超时");
            panic!("处理音频帧超时");
        }
    }
    
    engine.shutdown().await.expect("Failed to shutdown");
}

