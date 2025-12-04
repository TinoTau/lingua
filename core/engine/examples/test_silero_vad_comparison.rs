//! Silero VAD Python vs Rust 对比测试
//! 
//! 使用相同的模型和测试用例，对比 Python 和 Rust 实现的输出差异
//! 
//! 使用方法：
//!   cargo run --example test_silero_vad_comparison

use core_engine::vad::{SileroVad, SileroVadConfig, VoiceActivityDetector};
use core_engine::types::AudioFrame;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("  Silero VAD Python vs Rust 对比测试");
    println!("============================================================\n");

    // 1. 确定模型路径
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let official_model_path = manifest_dir.join("models").join("vad").join("silero").join("silero_vad_official.onnx");
    let model_path = if official_model_path.exists() {
        official_model_path
    } else {
        manifest_dir.join("models").join("vad").join("silero").join("silero_vad.onnx")
    };
    
    println!("[1/3] 检查模型文件...");
    if !model_path.exists() {
        eprintln!("❌ 模型文件不存在: {}", model_path.display());
        return Err(format!("模型文件不存在: {}", model_path.display()).into());
    }
    println!("✅ 模型文件存在: {}", model_path.display());
    println!();

    // 2. 初始化 SileroVad
    println!("[2/3] 初始化 SileroVad...");
    let config = SileroVadConfig {
        model_path: model_path.to_string_lossy().to_string(),
        sample_rate: 16000,
        frame_size: 512,  // 32ms @ 16kHz
        silence_threshold: 0.5,
        min_silence_duration_ms: 600,
        adaptive_enabled: false,  // 对比测试时禁用自适应
        adaptive_min_samples: 3,
        adaptive_rate: 0.1,
        adaptive_min_duration_ms: 300,
        adaptive_max_duration_ms: 1200,
    };
    
    let vad = SileroVad::with_config(config)?;
    println!("✅ SileroVad 初始化成功");
    println!();

    // 3. 运行对比测试
    println!("[3/3] 运行对比测试...");
    println!();
    
    // 测试用例
    let test_cases = vec![
        ("语音帧（440Hz正弦波）", create_speech_frame(0, 440.0)),
        ("静音帧（全零）", create_silence_frame(32)),
        ("语音帧2（880Hz正弦波）", create_speech_frame(64, 880.0)),
        ("静音帧2", create_silence_frame(96)),
    ];
    
    println!("Rust 实现结果:");
    println!("{}", "=".repeat(60));
    
    for (i, (name, frame)) in test_cases.iter().enumerate() {
        println!("\n[{}] {}", i + 1, name);
        
        // 计算音频统计信息
        let audio_max = frame.data.iter().cloned().fold(0.0f32, f32::max);
        let audio_min = frame.data.iter().cloned().fold(0.0f32, f32::min);
        let audio_mean = frame.data.iter().sum::<f32>() / frame.data.len() as f32;
        let audio_rms = (frame.data.iter().map(|x| x * x).sum::<f32>() / frame.data.len() as f32).sqrt();
        
        println!("  输入音频: {} samples, min={:.4}, max={:.4}, mean={:.4}, rms={:.4}",
                 frame.data.len(), audio_min, audio_max, audio_mean, audio_rms);
        
        // 检测
        let result = vad.detect(frame.clone()).await?;
        
        println!("  检测结果:");
        println!("    speech_prob (置信度) = {:.6}", result.confidence);
        println!("    is_boundary = {}", result.is_boundary);
        println!("    boundary_type = {:?}", result.boundary_type);
        println!("    判断: {}", if result.confidence > 0.5 { "语音" } else { "静音" });
    }
    
    println!("\n{}", "=".repeat(60));
    println!("对比说明");
    println!("{}", "=".repeat(60));
    println!();
    println!("请运行 Python 对比脚本获取 Python 实现的输出：");
    println!("  python core/engine/scripts/test_silero_vad_python_vs_rust.py");
    println!();
    println!("然后对比两者的输出，检查：");
    println!("  1. 模型输出的原始值是否一致");
    println!("  2. speech_prob 的计算方式是否正确");
    println!("  3. 阈值判断（> 0.5 为语音）是否合理");
    println!();
    
    Ok(())
}

/// 创建模拟语音帧（正弦波）
fn create_speech_frame(timestamp_ms: u64, frequency: f32) -> AudioFrame {
    let mut data = Vec::with_capacity(512);
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

