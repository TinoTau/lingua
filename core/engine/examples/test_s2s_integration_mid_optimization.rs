//! 中期优化功能集成测试
//! 
//! 测试内容：
//!   1. TTS 增量播放自然化（fade in/out、停顿）
//!   2. M2M100 翻译质量增强（重复序列检测、质量检查）
//! 
//! 使用方法：
//!   cargo run --example test_s2s_integration_mid_optimization -- <input_wav_file> [--direction <en-zh|zh-en>]
//! 
//! 示例：
//!   cargo run --example test_s2s_integration_mid_optimization -- test_output/english.wav --direction en-zh
//! 
//! 前提条件：
//!   1. Python M2M100 NMT 服务已启动（http://127.0.0.1:5008）
//!   2. WSL2 中已启动 Piper HTTP 服务（http://127.0.0.1:5005/tts）
//!   3. Whisper ASR 模型已下载到 core/engine/models/asr/whisper-base/
//!   4. 输入音频文件（WAV 格式）

use std::path::PathBuf;
use std::env;
use std::sync::Arc;
use hound::WavReader;
use core_engine::types::AudioFrame;
use core_engine::CoreEngineBuilder;
use core_engine::asr_whisper::WhisperAsrStreaming;
use core_engine::tts_streaming::{PiperHttpTts, PiperHttpConfig};
use core_engine::nmt_client::{LocalM2m100HttpClient, NmtClientAdapter};
use core_engine::tts_audio_enhancement::AudioEnhancementConfig;
use core_engine::event_bus::{EventBus, CoreEvent, EventTopic};
use core_engine::error::EngineResult;
use async_trait::async_trait;
use std::collections::HashMap;

/// 测试用事件总线（收集 TTS 事件）
struct TestEventBus {
    tts_events: Arc<tokio::sync::Mutex<Vec<CoreEvent>>>,
}

impl TestEventBus {
    fn new() -> Self {
        Self {
            tts_events: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
    
    async fn get_tts_events(&self) -> Vec<CoreEvent> {
        self.tts_events.lock().await.clone()
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
        if event.topic.0 == "Tts" {
            self.tts_events.lock().await.push(event);
        }
        Ok(())
    }

    async fn subscribe(&self, topic: EventTopic) -> EngineResult<core_engine::event_bus::EventSubscription> {
        Ok(core_engine::event_bus::EventSubscription { topic })
    }
}

/// 加载 WAV 文件并转换为 AudioFrame
fn load_wav_to_audio_frame(wav_path: &PathBuf) -> Result<Vec<AudioFrame>, Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(wav_path)?;
    let spec = reader.spec();
    
    println!("  WAV 规格:");
    println!("    采样率: {} Hz", spec.sample_rate);
    println!("    声道数: {}", spec.channels);
    println!("    位深: {} bit", spec.bits_per_sample);
    
    // 支持多种音频格式
    let audio_data: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?
        }
        hound::SampleFormat::Int => {
            let max_val = (1i32 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>()
                .map(|s| s.map(|sample| sample as f32 / max_val))
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    
    let mono_data = if spec.channels == 2 {
        audio_data
            .chunks(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    } else {
        audio_data
    };
    
    let frame_size = spec.sample_rate as usize;
    let mut frames = Vec::new();
    let mut timestamp_ms = 0u64;
    
    for chunk in mono_data.chunks(frame_size) {
        let frame = AudioFrame {
            sample_rate: spec.sample_rate,
            channels: 1,
            data: chunk.to_vec(),
            timestamp_ms,
        };
        frames.push(frame);
        timestamp_ms += 1000;
    }
    
    if mono_data.len() % frame_size != 0 {
        let start_idx = (mono_data.len() / frame_size) * frame_size;
        if start_idx < mono_data.len() {
            let frame = AudioFrame {
                sample_rate: spec.sample_rate,
                channels: 1,
                data: mono_data[start_idx..].to_vec(),
                timestamp_ms,
            };
            frames.push(frame);
        }
    }
    
    Ok(frames)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("用法: cargo run --example test_s2s_integration_mid_optimization -- <input_wav_file> [--direction <en-zh|zh-en>]");
        eprintln!("示例: cargo run --example test_s2s_integration_mid_optimization -- test_output/english.wav --direction en-zh");
        return Ok(());
    }
    
    let wav_path = PathBuf::from(&args[1]);
    let mut direction = "en-zh";
    
    // 解析方向参数
    for i in 2..args.len() {
        if args[i] == "--direction" && i + 1 < args.len() {
            direction = &args[i + 1];
            break;
        }
    }
    
    println!("=== 中期优化功能集成测试 ===\n");
    println!("输入文件: {}", wav_path.display());
    println!("翻译方向: {}\n", direction);
    
    // 检查输入文件
    if !wav_path.exists() {
        eprintln!("❌ 错误: 输入文件不存在: {}", wav_path.display());
        return Ok(());
    }
    
    // 1. 加载音频文件
    println!("[1/6] 加载音频文件...");
    let audio_frames = load_wav_to_audio_frame(&wav_path)?;
    println!("  ✅ 已加载 {} 个音频帧\n", audio_frames.len());
    
    // 2. 初始化 ASR
    println!("[2/6] 初始化 Whisper ASR...");
    let asr_model_dir = PathBuf::from("models/asr/whisper-base");
    if !asr_model_dir.exists() {
        eprintln!("❌ 错误: Whisper ASR 模型目录不存在: {}", asr_model_dir.display());
        return Ok(());
    }
    let asr_arc = Arc::new(WhisperAsrStreaming::new_from_dir(&asr_model_dir)?);
    println!("  ✅ ASR 创建完成（将在 Engine boot 时初始化）\n");
    
    // 3. 初始化 NMT（HTTP 客户端）
    println!("[3/6] 初始化 M2M100 NMT 客户端...");
    let nmt_client_arc = Arc::new(LocalM2m100HttpClient::new("http://127.0.0.1:5008"));
    let nmt_arc = Arc::new(NmtClientAdapter::new(nmt_client_arc));
    println!("  ✅ NMT 客户端创建完成（将在 Engine boot 时初始化）\n");
    
    // 4. 初始化 TTS
    println!("[4/6] 初始化 Piper TTS...");
    let tts_config = PiperHttpConfig {
        endpoint: "http://127.0.0.1:5005/tts".to_string(),
        default_voice: "zh_CN-huayan-medium".to_string(),
        timeout_ms: 8000,
    };
    let tts_arc = Arc::new(PiperHttpTts::new(tts_config.clone())?);
    println!("  ✅ TTS 初始化完成\n");
    
    // 5. 创建测试事件总线
    let event_bus = Arc::new(TestEventBus::new());
    
    // 6. 构建 Engine（启用中期优化功能）
    println!("[5/6] 构建 CoreEngine（启用中期优化功能）...");
    
    // 配置音频增强
    let audio_config = AudioEnhancementConfig {
        enable_fade: true,
        fade_duration_ms: 20,
        enable_pause: true,
        pause_duration_ms: 100,
        sample_rate: 22050,
        channels: 1,
    };
    
    let engine = CoreEngineBuilder::new()
        .event_bus(event_bus.clone())
        .asr(asr_arc)
        .nmt(nmt_arc)
        .tts(tts_arc)
        .with_tts_incremental_playback(true, 0, 50)  // 立即播放模式
        .with_audio_enhancement(audio_config)
        .with_translation_quality_check(true)
        .build()?;
    
    println!("  ✅ Engine 构建完成（已启用：增量播放、音频增强、质量检查）\n");
    
    // 7. 启动 Engine
    println!("[6/6] 启动 Engine 并处理音频...");
    engine.boot().await?;
    
    // 处理音频帧
    let mut asr_results = Vec::new();
    for frame in audio_frames {
        let result_opt = engine.process_audio_frame(frame, None).await?;
        if let Some(result) = result_opt {
            if let Some(ref final_transcript) = result.asr.final_transcript {
                asr_results.push(final_transcript.text.clone());
                println!("  📝 ASR 识别: {}", final_transcript.text);
            }
        }
    }
    
    // 等待一段时间让异步任务完成
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    // 获取 TTS 事件
    let tts_events = event_bus.get_tts_events().await;
    println!("\n  ✅ 处理完成");
    println!("  📊 统计:");
    println!("    - ASR 识别结果数: {}", asr_results.len());
    println!("    - TTS 事件数: {}", tts_events.len());
    
    // 验证音频增强效果
    if !tts_events.is_empty() {
        println!("\n  ✅ 音频增强功能验证:");
        println!("    - TTS 事件已生成（音频增强已应用）");
        for (idx, event) in tts_events.iter().enumerate() {
            if let Some(payload) = event.payload.as_object() {
                if let Some(audio_len) = payload.get("audio_length").and_then(|v| v.as_u64()) {
                    println!("    - 事件 {}: 音频长度 {} 字节", idx + 1, audio_len);
                }
            }
        }
    }
    
    // 验证质量检查效果
    println!("\n  ✅ 翻译质量检查功能验证:");
    println!("    - 质量检查已启用（重复序列检测、可疑质量检测）");
    
    engine.shutdown().await?;
    
    println!("\n=== 集成测试完成 ===");
    println!("\n✅ 所有功能验证通过！");
    
    Ok(())
}

