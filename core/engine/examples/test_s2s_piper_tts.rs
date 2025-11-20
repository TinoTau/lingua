//! 完整 S2S 流集成测试：测试文本→翻译→TTS 流程（使用 Piper HTTP TTS）
//! 
//! 使用方法：
//!   cargo run --example test_s2s_piper_tts
//! 
//! 前提条件：
//!   1. WSL2 中已启动 Piper HTTP 服务（http://127.0.0.1:5005/tts）
//!   2. 服务正在运行
//!   3. NMT 模型已加载（如果使用真实 NMT）
//! 
//! 注意：此测试模拟完整的 S2S 流程，但使用模拟的 ASR 结果（直接提供文本）

use core_engine::bootstrap::CoreEngineBuilder;
use core_engine::tts_streaming::TtsRequest;
use core_engine::nmt_incremental::{TranslationRequest, TranslationResponse};
use core_engine::event_bus::{EventBus, CoreEvent, EventTopic};
use core_engine::vad::VoiceActivityDetector;
use core_engine::asr_streaming::{AsrStreaming, AsrRequest, AsrResult};
use core_engine::nmt_incremental::NmtIncremental;
use core_engine::emotion_adapter::{EmotionAdapter, EmotionRequest, EmotionResponse};
use core_engine::persona_adapter::{PersonaAdapter, PersonaContext};
use core_engine::config_manager::{ConfigManager, EngineConfig};
use core_engine::cache_manager::CacheManager;
use core_engine::telemetry::TelemetrySink;
use core_engine::types::{AudioFrame, StableTranscript};
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;
use async_trait::async_trait;

// 使用现有的 stub 实现
use core_engine::emotion_adapter::EmotionStub;
use core_engine::persona_adapter::PersonaStub;
use core_engine::nmt_incremental::MarianNmtStub;

// 简化的 stub 实现
struct StubEventBus;
#[async_trait]
impl EventBus for StubEventBus {
    async fn start(&self) -> core_engine::error::EngineResult<()> { Ok(()) }
    async fn stop(&self) -> core_engine::error::EngineResult<()> { Ok(()) }
    async fn publish(&self, _event: CoreEvent) -> core_engine::error::EngineResult<()> { Ok(()) }
    async fn subscribe(&self, _topic: EventTopic) -> core_engine::error::EngineResult<core_engine::event_bus::EventSubscription> {
        Ok(core_engine::event_bus::EventSubscription {
            topic: EventTopic("test".to_string()),
        })
    }
}

struct StubVad;
#[async_trait]
impl VoiceActivityDetector for StubVad {
    async fn detect(&self, _frame: &AudioFrame) -> core_engine::error::EngineResult<core_engine::vad::DetectionOutcome> {
        Ok(core_engine::vad::DetectionOutcome {
            is_speech: false,
            confidence: 0.0,
        })
    }
}

struct StubAsr;
#[async_trait]
impl AsrStreaming for StubAsr {
    async fn initialize(&self) -> core_engine::error::EngineResult<()> { Ok(()) }
    async fn infer(&self, _request: AsrRequest) -> core_engine::error::EngineResult<AsrResult> {
        Ok(AsrResult {
            partial: None,
            final_transcript: None,
        })
    }
    async fn finalize(&self) -> core_engine::error::EngineResult<()> { Ok(()) }
}

struct StubConfig;
#[async_trait]
impl ConfigManager for StubConfig {
    async fn load(&self) -> core_engine::error::EngineResult<EngineConfig> {
        Ok(EngineConfig {
            mode: "test".to_string(),
            source_language: "zh-CN".to_string(),
            target_language: "en-US".to_string(),
        })
    }
    async fn current(&self) -> core_engine::error::EngineResult<EngineConfig> {
        Ok(EngineConfig {
            mode: "test".to_string(),
            source_language: "zh-CN".to_string(),
            target_language: "en-US".to_string(),
        })
    }
}

struct StubCache;
#[async_trait]
impl CacheManager for StubCache {
    async fn warm_up(&self) -> core_engine::error::EngineResult<()> { Ok(()) }
    async fn purge(&self) -> core_engine::error::EngineResult<()> { Ok(()) }
}

struct StubTelemetry;
#[async_trait]
impl TelemetrySink for StubTelemetry {
    async fn record(&self, _datum: core_engine::telemetry::TelemetryDatum) -> core_engine::error::EngineResult<()> { Ok(()) }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 完整 S2S 流集成测试（Piper TTS） ===\n");

    // 检查服务是否运行
    println!("[1/6] 检查 Piper HTTP 服务状态...");
    let health_url = "http://127.0.0.1:5005/health";
    let client = reqwest::Client::new();
    match client.get(health_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("[OK] Piper HTTP 服务正在运行");
            } else {
                eprintln!("[ERROR] 服务返回错误状态: {}", resp.status());
                return Err("Service health check failed".into());
            }
        }
        Err(e) => {
            eprintln!("[ERROR] 无法连接到服务: {}", e);
            eprintln!("[INFO] 请确保 WSL2 中的 Piper HTTP 服务正在运行");
            return Err("Service not available".into());
        }
    }

    // 构建 CoreEngine
    println!("\n[2/6] 构建 CoreEngine（使用 Piper HTTP TTS）...");
    let engine = CoreEngineBuilder::new()
        .event_bus(Arc::new(StubEventBus))
        .vad(Arc::new(StubVad))
        .asr(Arc::new(StubAsr))
        .nmt(Arc::new(MarianNmtStub::new()))
        .emotion(Arc::new(EmotionStub::new()))
        .persona(Arc::new(PersonaStub::new()))
        .config(Arc::new(StubConfig))
        .cache(Arc::new(StubCache))
        .telemetry(Arc::new(StubTelemetry))
        .tts_with_default_piper_http()
        .map_err(|e| format!("Failed to build CoreEngine: {}", e))?;
    
    println!("[OK] CoreEngine 构建成功");

    // 初始化引擎
    println!("\n[3/6] 初始化 CoreEngine...");
    let engine = engine.build()?;
    engine.boot().await
        .map_err(|e| format!("Failed to boot CoreEngine: {}", e))?;
    println!("[OK] CoreEngine 初始化成功");

    // 模拟 ASR 结果（中文文本）
    println!("\n[4/6] 模拟 ASR 结果（中文文本）...");
    let source_text = "你好，欢迎使用 Lingua 语音翻译系统。";
    println!("  源文本（中文）: {}", source_text);

    // 步骤 5: NMT 翻译（使用 stub，返回模拟翻译）
    println!("\n[5/6] 执行 NMT 翻译...");
    let translation_request = TranslationRequest {
        text: source_text.to_string(),
        source_language: "zh-CN".to_string(),
        target_language: "en-US".to_string(),
    };
    
    let translation_response = engine.nmt().translate(translation_request).await
        .map_err(|e| format!("Translation failed: {}", e))?;
    
    let target_text = translation_response.translated_text;
    println!("  目标文本（英文）: {}", target_text);
    println!("  翻译稳定: {}", translation_response.is_stable);

    // 步骤 6: TTS 合成（使用 Piper HTTP TTS 合成中文语音）
    println!("\n[6/6] 执行 TTS 合成（Piper HTTP）...");
    println!("  注意: 这里合成的是中文语音（源语言），用于回放");
    
    let tts_request = TtsRequest {
        text: source_text.to_string(), // 使用源文本（中文）进行 TTS
        voice: "zh_CN-huayan-medium".to_string(),
        locale: "zh-CN".to_string(),
    };
    
    let start_time = std::time::Instant::now();
    let chunk = engine.tts().synthesize(tts_request).await
        .map_err(|e| format!("TTS synthesis failed: {}", e))?;
    let elapsed = start_time.elapsed();
    
    println!("[OK] TTS 合成成功");
    println!("  耗时: {:?}", elapsed);
    println!("  音频大小: {} 字节", chunk.audio.len());
    println!("  是否最后一块: {}", chunk.is_last);

    // 验证 WAV 格式
    if chunk.audio.len() >= 4 {
        let header = String::from_utf8_lossy(&chunk.audio[0..4]);
        if header == "RIFF" {
            println!("  格式: WAV (RIFF)");
        }
    }

    // 保存到文件（使用项目根目录）
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| manifest_dir.as_path());
    let output_file = project_root.join("test_output").join("s2s_piper_test.wav");
    
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&output_file, &chunk.audio)?;
    println!("\n[OK] 音频文件已保存");
    println!("  文件路径: {}", output_file.display());
    println!("  文件大小: {} 字节", fs::metadata(&output_file)?.len());

    // 验证文件大小
    if chunk.audio.len() > 1024 {
        println!("[OK] 音频文件大小 > 1024 字节，符合预期");
    }

    // 关闭引擎
    println!("\n关闭 CoreEngine...");
    engine.shutdown().await
        .map_err(|e| format!("Failed to shutdown CoreEngine: {}", e))?;
    println!("[OK] CoreEngine 已关闭");

    println!("\n=== 测试完成 ===");
    println!("\n测试流程总结：");
    println!("  1. ✅ ASR 模拟: 中文文本 \"{}\"", source_text);
    println!("  2. ✅ NMT 翻译: 英文文本 \"{}\"", target_text);
    println!("  3. ✅ TTS 合成: 中文语音（Piper HTTP）");
    println!("\n下一步：");
    println!("  1. 播放音频文件验证语音质量: {}", output_file.display());
    println!("  2. 如果正常，完整的 S2S 流程已验证通过");

    Ok(())
}

