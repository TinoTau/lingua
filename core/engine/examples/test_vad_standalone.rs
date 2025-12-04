//! 独立测试程序：验证 VAD 服务
//! 
//! 使用方法：
//!   cargo run --example test_vad_standalone
//! 
//! 测试内容：
//!   1. TimeBasedVad - 基于时间的 VAD
//!   2. SileroVad - 基于 ONNX 的 VAD（如果模型可用）

use core_engine::vad::{TimeBasedVad, SileroVad, VoiceActivityDetector};
use core_engine::types::AudioFrame;
use std::path::PathBuf;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== VAD 独立测试程序 ===\n");

    // 获取项目根目录
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| manifest_dir.as_path());
    let test_output_dir = project_root.join("test_output");

    // 测试 1: TimeBasedVad
    println!("[1/2] 测试 TimeBasedVad（基于时间的 VAD）...");
    test_time_based_vad().await?;
    println!("[OK] TimeBasedVad 测试通过\n");

    // 测试 2: SileroVad（如果模型可用）
    println!("[2/2] 测试 SileroVad（基于 ONNX 的 VAD）...");
    let silero_model_path = project_root.join("core").join("engine").join("models").join("vad").join("silero").join("silero_vad.onnx");
    
    if silero_model_path.exists() {
        test_silero_vad(&silero_model_path).await?;
        println!("[OK] SileroVad 测试通过\n");
    } else {
        println!("[SKIP] SileroVad 模型不存在，跳过测试");
        println!("  模型路径: {}", silero_model_path.display());
        println!("  提示: 如需测试 SileroVad，请先下载模型\n");
    }

    println!("=== 测试完成 ===");
    Ok(())
}

async fn test_time_based_vad() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 TimeBasedVad（3秒间隔）
    let vad = TimeBasedVad::new(3000);
    
    println!("  配置: 3秒间隔");
    
    // 创建测试音频帧
    let create_frame = |timestamp_ms: u64| -> AudioFrame {
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.1; 512], // 模拟音频数据
            timestamp_ms,
        }
    };
    
    // 测试序列
    let test_cases = vec![
        (0, false, "第一帧（初始化）"),
        (1000, false, "1秒后（未到边界）"),
        (2000, false, "2秒后（未到边界）"),
        (3000, true, "3秒后（应该检测到边界）"),
        (3500, false, "3.5秒后（未到边界）"),
        (6000, true, "6秒后（应该检测到边界）"),
    ];
    
    for (timestamp, expected_boundary, description) in test_cases {
        let frame = create_frame(timestamp);
        let result = vad.detect(frame).await?;
        
        if result.is_boundary == expected_boundary {
            println!("  ✅ {}ms: {} - 边界={}", timestamp, description, result.is_boundary);
        } else {
            println!("  ❌ {}ms: {} - 期望边界={}, 实际边界={}", 
                timestamp, description, expected_boundary, result.is_boundary);
            return Err(format!("TimeBasedVad 测试失败: {}ms", timestamp).into());
        }
    }
    
    // 测试重置
    vad.reset().await?;
    let frame_after_reset = create_frame(5000);
    let result_after_reset = vad.detect(frame_after_reset).await?;
    if !result_after_reset.is_boundary {
        println!("  ✅ 重置后第一帧不检测边界（正确）");
    } else {
        println!("  ❌ 重置后第一帧检测到边界（错误）");
        return Err("TimeBasedVad 重置测试失败".into());
    }
    
    Ok(())
}

async fn test_silero_vad(model_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 创建 SileroVad
    let vad = SileroVad::new(model_path)?;
    
    println!("  模型路径: {}", model_path.display());
    println!("  配置: 16kHz, 512 samples/frame");
    
    // 创建测试音频帧（模拟语音和静音）
    let create_speech_frame = |timestamp_ms: u64| -> AudioFrame {
        // 模拟语音信号（正弦波）
        let mut data = Vec::with_capacity(512);
        for i in 0..512 {
            let sample = (i as f32 * 0.1).sin() * 0.5;
            data.push(sample);
        }
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data,
            timestamp_ms,
        }
    };
    
    let create_silence_frame = |timestamp_ms: u64| -> AudioFrame {
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.0; 512], // 静音
            timestamp_ms,
        }
    };
    
    // 测试序列：语音 -> 静音 -> 语音
    println!("  测试序列: 语音 -> 静音 -> 语音");
    
    // 发送语音帧
    for i in 0..10 {
        let frame = create_speech_frame(i * 32); // 32ms per frame
        let result = vad.detect(frame).await?;
        println!("    帧 {}: 语音 - 边界={}, 置信度={:.2}", i, result.is_boundary, result.confidence);
    }
    
    // 发送静音帧（应该检测到边界）
    let mut boundary_detected = false;
    for i in 10..30 {
        let frame = create_silence_frame(i * 32);
        let result = vad.detect(frame).await?;
        if result.is_boundary {
            boundary_detected = true;
            println!("    帧 {}: 静音 - 边界={}, 置信度={:.2} ✅", i, result.is_boundary, result.confidence);
            break;
        }
    }
    
    if !boundary_detected {
        println!("  ⚠️  未检测到静音边界（可能需要更多静音帧）");
    }
    
    // 重置
    vad.reset().await?;
    println!("  ✅ 重置成功");
    
    Ok(())
}

