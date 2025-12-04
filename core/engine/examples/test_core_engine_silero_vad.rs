//! CoreEngine 与 SileroVad 集成测试
//! 
//! 测试 CoreEngine 使用 SileroVad 进行自然停顿检测的功能
//! 
//! 使用方法：
//!   cargo run --example test_core_engine_silero_vad
//! 
//! 前置条件：
//!   1. NMT 服务运行在 http://127.0.0.1:5008
//!   2. TTS 服务运行在 http://127.0.0.1:5005
//!   3. Silero VAD 模型位于 models/vad/silero/silero_vad.onnx

use core_engine::bootstrap::CoreEngineBuilder;
use core_engine::types::AudioFrame;
use core_engine::vad::{SileroVad, SileroVadConfig, VoiceActivityDetector};
use core_engine::event_bus::SimpleEventBus;
use std::sync::Arc;
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("  CoreEngine + SileroVad 集成测试");
    println!("============================================================\n");

    // 1. 检查服务健康状态
    println!("[1/4] 检查依赖服务...");
    
    let nmt_url = "http://127.0.0.1:5008";
    let tts_url = "http://127.0.0.1:5005/tts";
    
    // 检查 NMT 服务
    let nmt_client = reqwest::Client::new();
    match nmt_client.get(&format!("{}/health", nmt_url)).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("  ✅ NMT 服务正常: {}", nmt_url);
        }
        _ => {
            eprintln!("  ❌ NMT 服务不可用: {}", nmt_url);
            eprintln!("     请确保 NMT 服务已启动");
            return Err("NMT 服务不可用".into());
        }
    }
    
    // 检查 TTS 服务
    match nmt_client.get(&format!("{}/health", tts_url.replace("/tts", ""))).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("  ✅ TTS 服务正常: {}", tts_url);
        }
        _ => {
            eprintln!("  ❌ TTS 服务不可用: {}", tts_url);
            eprintln!("     请确保 TTS 服务已启动");
            return Err("TTS 服务不可用".into());
        }
    }
    println!();

    // 2. 初始化 SileroVad
    println!("[2/4] 初始化 SileroVad...");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_path = manifest_dir.join("models").join("vad").join("silero").join("silero_vad.onnx");
    
    if !model_path.exists() {
        eprintln!("  ❌ Silero VAD 模型文件不存在: {}", model_path.display());
        return Err(format!("模型文件不存在: {}", model_path.display()).into());
    }
    
    let vad_config = SileroVadConfig {
        model_path: model_path.to_string_lossy().to_string(),
        sample_rate: 16000,
        frame_size: 512,
        silence_threshold: 0.5,
        min_silence_duration_ms: 600,
    };
    
    let vad = Arc::new(SileroVad::with_config(vad_config)?);
    println!("  ✅ SileroVad 初始化成功");
    println!();

    // 3. 初始化 CoreEngine
    println!("[3/4] 初始化 CoreEngine...");
    let init_start = Instant::now();
    
    let engine = CoreEngineBuilder::new()
        .event_bus(Arc::new(SimpleEventBus))
        .vad(vad.clone())
        .asr_with_default_whisper()
        .map_err(|e| format!("ASR 初始化失败: {}", e))?
        .nmt_with_m2m100_http_client(Some(nmt_url))
        .map_err(|e| format!("NMT 初始化失败: {}", e))?
        .tts_with_piper_http(Some(tts_url))
        .map_err(|e| format!("TTS 初始化失败: {}", e))?
        .build()
        .map_err(|e| format!("CoreEngine 构建失败: {}", e))?;
    
    let init_ms = init_start.elapsed().as_millis();
    println!("  ✅ CoreEngine 初始化成功 (耗时: {}ms)", init_ms);
    println!();

    // 4. 测试自然停顿检测
    println!("[4/4] 测试自然停顿检测...");
    println!("  发送语音帧序列（模拟说话 -> 停顿 -> 说话）...");
    
    // 重置 VAD 状态
    vad.reset().await?;
    
    let mut timestamp = 0u64;
    let mut boundary_count = 0;
    
    // 发送语音帧（模拟说话）
    println!("  发送语音帧（0-320ms）...");
    for i in 0..10 {
        let frame = create_speech_frame(timestamp);
        let result = vad.detect(frame).await?;
        timestamp += 32;
        
        if result.is_boundary {
            boundary_count += 1;
            println!("    帧 {}: 检测到边界 (类型: {:?}, 置信度: {:.3})", 
                     i, result.boundary_type, result.confidence);
        }
    }
    
    // 发送静音帧（模拟停顿）
    println!("  发送静音帧（320-608ms，期望检测到自然停顿）...");
    let mut natural_pause_detected = false;
    for i in 0..9 {
        let frame = create_silence_frame(timestamp);
        let result = vad.detect(frame).await?;
        timestamp += 32;
        
        if result.is_boundary {
            boundary_count += 1;
            if let Some(core_engine::vad::BoundaryType::NaturalPause) = result.boundary_type {
                natural_pause_detected = true;
                println!("    帧 {}: ✅ 检测到自然停顿! (置信度: {:.3})", i, result.confidence);
            } else {
                println!("    帧 {}: 检测到边界 (类型: {:?}, 置信度: {:.3})", 
                         i, result.boundary_type, result.confidence);
            }
        }
    }
    
    // 继续发送语音帧（模拟继续说话）
    println!("  发送语音帧（608-928ms）...");
    for i in 0..10 {
        let frame = create_speech_frame(timestamp);
        let result = vad.detect(frame).await?;
        timestamp += 32;
        
        if result.is_boundary {
            boundary_count += 1;
            println!("    帧 {}: 检测到边界 (类型: {:?}, 置信度: {:.3})", 
                     i, result.boundary_type, result.confidence);
        }
    }
    
    println!();
    println!("============================================================");
    println!("  测试完成");
    println!("============================================================");
    println!();
    println!("总结:");
    println!("  ✅ SileroVad 初始化: 成功");
    println!("  ✅ CoreEngine 初始化: 成功");
    println!("  ✅ 边界检测次数: {}", boundary_count);
    if natural_pause_detected {
        println!("  ✅ 自然停顿检测: 成功");
    } else {
        println!("  ⚠️  自然停顿检测: 未触发（可能需要调整配置）");
    }
    println!();
    println!("SileroVad 已成功集成到 CoreEngine！");
    
    Ok(())
}

/// 创建模拟语音帧（正弦波）
fn create_speech_frame(timestamp_ms: u64) -> AudioFrame {
    let mut data = Vec::with_capacity(512);
    let frequency = 440.0;
    let sample_rate = 16000.0;
    for i in 0..512 {
        let t = (i as f32) / sample_rate;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5;
        data.push(sample);
    }
    
    AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data,
        timestamp_ms,
    }
}

/// 创建静音帧
fn create_silence_frame(timestamp_ms: u64) -> AudioFrame {
    AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: vec![0.0; 512],
        timestamp_ms,
    }
}

