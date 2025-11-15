// tests/asr_whisper_engine_test.rs
// 测试 WhisperAsrEngine 的完整功能

use std::path::PathBuf;
use core_engine::asr_whisper::WhisperAsrEngine;
use core_engine::types::AudioFrame;

/// 测试 1: 从模型路径加载引擎
#[test]
fn test_whisper_engine_load_from_path() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");

    if !model_path.exists() {
        println!("⚠ 跳过测试: 模型文件不存在");
        return;
    }

    let engine = WhisperAsrEngine::new_from_model_path(&model_path)
        .expect("Failed to load Whisper engine");

    println!("✓ WhisperAsrEngine 加载成功");
    println!("  模型路径: {}", engine.model_path().display());
    assert_eq!(engine.model_path(), model_path);
}

/// 测试 2: 从模型目录加载引擎
#[test]
fn test_whisper_engine_load_from_dir() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");

    if !model_dir.exists() {
        println!("⚠ 跳过测试: 模型目录不存在");
        return;
    }

    let engine = WhisperAsrEngine::new_from_dir(&model_dir)
        .expect("Failed to load Whisper engine from directory");

    println!("✓ WhisperAsrEngine 从目录加载成功");
    println!("  模型路径: {}", engine.model_path().display());
}

/// 测试 3: 使用 AudioFrame 进行转录
#[test]
fn test_whisper_engine_transcribe_frame() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");

    let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");
    let wav_path = project_root.join("third_party/whisper.cpp/samples/jfk.wav");

    if !model_path.exists() || !wav_path.exists() {
        println!("⚠ 跳过测试: 模型或音频文件不存在");
        return;
    }

    // 加载引擎
    let mut engine = WhisperAsrEngine::new_from_model_path(&model_path)
        .expect("Failed to load Whisper engine");
    engine.set_language(Some("en".to_string()));

    // 加载音频并转换为 AudioFrame
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
        channels: spec.channels as u8,  // hound 使用 u16，AudioFrame 使用 u8
        data: samples,
        timestamp_ms: 0,
    };

    println!("\n使用 AudioFrame 进行转录...");
    println!("  原始采样率: {} Hz", frame.sample_rate);
    println!("  声道数: {}", frame.channels);
    println!("  数据长度: {} 样本", frame.data.len());

    // 进行转录
    let result = engine.transcribe_frame(&frame)
        .expect("Failed to transcribe frame");

    println!("\n转录结果:");
    println!("{}", result);

    // 验证结果
    let result_lower = result.to_lowercase();
    assert!(
        result_lower.contains("ask not what your country can do for you") ||
        result_lower.contains("what you can do for your country"),
        "转录结果应该包含 JFK 演讲的关键内容"
    );
}

/// 测试 4: 使用多个 AudioFrame 进行转录
#[test]
fn test_whisper_engine_transcribe_frames() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");

    let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");
    let wav_path = project_root.join("third_party/whisper.cpp/samples/jfk.wav");

    if !model_path.exists() || !wav_path.exists() {
        println!("⚠ 跳过测试: 模型或音频文件不存在");
        return;
    }

    // 加载引擎
    let mut engine = WhisperAsrEngine::new_from_model_path(&model_path)
        .expect("Failed to load Whisper engine");
    engine.set_language(Some("en".to_string()));

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

    // 将音频分割成多个帧（模拟流式输入）
    let chunk_size = samples.len() / 3;  // 分成 3 个帧
    let mut frames = Vec::new();
    
    for (i, chunk) in samples.chunks(chunk_size).enumerate() {
        frames.push(AudioFrame {
            sample_rate: spec.sample_rate,
            channels: spec.channels as u8,  // hound 使用 u16，AudioFrame 使用 u8
            data: chunk.to_vec(),
            timestamp_ms: (i * chunk_size * 1000 / spec.sample_rate as usize) as u64,
        });
    }

    println!("\n使用多个 AudioFrame 进行转录...");
    println!("  帧数: {}", frames.len());
    println!("  每帧大小: ~{} 样本", chunk_size);

    // 进行转录
    let result = engine.transcribe_frames(&frames)
        .expect("Failed to transcribe frames");

    println!("\n转录结果:");
    println!("{}", result);

    // 验证结果
    assert!(!result.trim().is_empty(), "转录结果不应为空");
}

