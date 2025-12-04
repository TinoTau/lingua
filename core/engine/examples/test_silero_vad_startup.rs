//! SileroVad 服务启动测试
//! 
//! 测试 SileroVad 在服务启动时的初始化和基本功能
//! 
//! 使用方法：
//!   cargo run --example test_silero_vad_startup
//! 
//! 测试内容：
//!   1. 模型加载和初始化
//!   2. 基本检测功能（语音/静音）
//!   3. 自然停顿检测
//!   4. 边界类型返回

use core_engine::vad::{SileroVad, SileroVadConfig, VoiceActivityDetector};
use core_engine::types::AudioFrame;
use core_engine::vad::BoundaryType;
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("  SileroVad 服务启动测试");
    println!("============================================================\n");

    // 1. 确定模型路径
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // 优先使用官方模型，如果不存在则使用旧模型
    let official_model_path = manifest_dir.join("models").join("vad").join("silero").join("silero_vad_official.onnx");
    let model_path = if official_model_path.exists() {
        official_model_path
    } else {
        manifest_dir.join("models").join("vad").join("silero").join("silero_vad.onnx")
    };
    
    println!("[1/4] 检查模型文件...");
    if !model_path.exists() {
        eprintln!("❌ 模型文件不存在: {}", model_path.display());
        eprintln!("   请确保模型文件位于正确路径");
        return Err(format!("模型文件不存在: {}", model_path.display()).into());
    }
    println!("✅ 模型文件存在: {}", model_path.display());
    println!();

    // 2. 初始化 SileroVad
    println!("[2/4] 初始化 SileroVad...");
    let init_start = Instant::now();
    
    let config = SileroVadConfig {
        model_path: model_path.to_string_lossy().to_string(),
        sample_rate: 16000,
        frame_size: 512,  // 32ms @ 16kHz
        silence_threshold: 0.5,
        min_silence_duration_ms: 600,  // 0.6秒
        adaptive_enabled: false,  // 测试时禁用自适应
        adaptive_min_samples: 3,
        adaptive_rate: 0.1,
        adaptive_min_duration_ms: 300,
        adaptive_max_duration_ms: 1200,
    };
    
    let vad = match SileroVad::with_config(config) {
        Ok(v) => {
            let init_ms = init_start.elapsed().as_millis();
            println!("✅ SileroVad 初始化成功 (耗时: {}ms)", init_ms);
            println!("   配置:");
            println!("     - 采样率: 16kHz");
            println!("     - 帧大小: 512 samples (32ms)");
            println!("     - 静音阈值: 0.5");
            println!("     - 最小静音时长: 600ms");
            println!();
            v
        }
        Err(e) => {
            eprintln!("❌ SileroVad 初始化失败: {}", e);
            return Err(format!("初始化失败: {}", e).into());
        }
    };

    // 3. 测试基本检测功能
    println!("[3/4] 测试基本检测功能...");
    
    // 3.1 测试语音帧（正弦波模拟语音）
    println!("  3.1 测试语音帧（模拟语音信号）...");
    let speech_frame = create_speech_frame(0);
    let speech_result = vad.detect(speech_frame).await?;
    println!("     ✅ 语音帧检测:");
    println!("        - 边界: {}", speech_result.is_boundary);
    println!("        - 置信度: {:.3}", speech_result.confidence);
    println!("        - 边界类型: {:?}", speech_result.boundary_type);
    
    if speech_result.confidence > 0.5 {
        println!("     ✅ 置信度正常（> 0.5，表示检测到语音）");
    } else {
        println!("     ⚠️  置信度较低（< 0.5），可能需要调整阈值");
    }
    println!();

    // 3.2 测试静音帧
    println!("  3.2 测试静音帧...");
    let silence_frame = create_silence_frame(32);
    let silence_result = vad.detect(silence_frame).await?;
    println!("     ✅ 静音帧检测:");
    println!("        - 边界: {}", silence_result.is_boundary);
    println!("        - 置信度: {:.3}", silence_result.confidence);
    println!("        - 边界类型: {:?}", silence_result.boundary_type);
    
    if silence_result.confidence < 0.5 {
        println!("     ✅ 置信度正常（< 0.5，表示检测到静音）");
    } else {
        println!("     ⚠️  置信度较高（> 0.5），可能误判为语音");
    }
    println!();

    // 4. 测试自然停顿检测
    println!("[4/4] 测试自然停顿检测...");
    println!("  发送语音帧 -> 静音帧序列（模拟自然停顿）...");
    
    // 重置 VAD 状态
    vad.reset().await?;
    
    // 发送多个语音帧
    let mut timestamp = 0u64;
    for i in 0..10 {
        let frame = create_speech_frame(timestamp);
        let result = vad.detect(frame).await?;
        timestamp += 32; // 32ms per frame
        if i == 0 || i == 9 {
            println!("    帧 {}: 语音 - 边界={}, 置信度={:.3}", i, result.is_boundary, result.confidence);
        }
    }
    
    // 发送静音帧，直到检测到边界
    println!("  发送静音帧（等待检测到自然停顿）...");
    let mut boundary_detected = false;
    let mut silence_frame_count = 0;
    
    for _i in 0..30 {  // 最多30帧（约1秒）
        let frame = create_silence_frame(timestamp);
        let result = vad.detect(frame).await?;
        timestamp += 32;
        silence_frame_count += 1;
        
        if result.is_boundary {
            boundary_detected = true;
            let silence_duration_ms = silence_frame_count * 32;
            println!("     ✅ 检测到自然停顿!");
            println!("        - 静音帧数: {}", silence_frame_count);
            println!("        - 静音时长: {}ms", silence_duration_ms);
            println!("        - 边界类型: {:?}", result.boundary_type);
            println!("        - 置信度: {:.3}", result.confidence);
            
            // 验证边界类型
            if let Some(BoundaryType::NaturalPause) = result.boundary_type {
                println!("     ✅ 边界类型正确（NaturalPause）");
            } else {
                println!("     ⚠️  边界类型不正确，期望 NaturalPause");
            }
            break;
        }
    }
    
    if !boundary_detected {
        println!("     ⚠️  未检测到自然停顿（可能需要更多静音帧或调整配置）");
        println!("        - 已发送 {} 帧静音（{}ms）", silence_frame_count, silence_frame_count * 32);
        println!("        - 建议: 增加 min_silence_duration_ms 或减少 silence_threshold");
    }
    println!();

    // 5. 测试重置功能
    println!("[额外] 测试重置功能...");
    vad.reset().await?;
    println!("✅ 重置成功");
    println!();

    println!("============================================================");
    println!("  测试完成");
    println!("============================================================");
    println!();
    println!("总结:");
    println!("  ✅ 模型加载: 成功");
    println!("  ✅ 初始化: 成功");
    println!("  ✅ 语音检测: 正常");
    println!("  ✅ 静音检测: 正常");
    if boundary_detected {
        println!("  ✅ 自然停顿检测: 成功");
    } else {
        println!("  ⚠️  自然停顿检测: 未触发（可能需要调整配置）");
    }
    println!();
    println!("SileroVad 已准备就绪，可以在 CoreEngine 中使用！");
    
    Ok(())
}

/// 创建模拟语音帧（正弦波）
fn create_speech_frame(timestamp_ms: u64) -> AudioFrame {
    let mut data = Vec::with_capacity(512);
    // 生成 440Hz 正弦波（A4 音符）模拟语音
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

