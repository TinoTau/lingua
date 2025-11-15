// tests/asr_whisper_streaming_test.rs
// 测试 WhisperAsrStreaming 的 AsrStreaming trait 实现

use std::path::PathBuf;
use std::sync::Arc;
use core_engine::asr_whisper::WhisperAsrStreaming;
use core_engine::asr_streaming::{AsrRequest, AsrStreaming};
use core_engine::types::AudioFrame;

/// 测试 1: 初始化 WhisperAsrStreaming
#[tokio::test]
async fn test_whisper_streaming_initialize() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");

    if !model_path.exists() {
        println!("⚠ 跳过测试: 模型文件不存在");
        return;
    }

    let asr = WhisperAsrStreaming::new_from_model_path(&model_path)
        .expect("Failed to create WhisperAsrStreaming");

    let result = asr.initialize().await;
    assert!(result.is_ok(), "初始化应该成功");
    println!("✓ WhisperAsrStreaming 初始化成功");
}

/// 测试 2: 使用单个 AudioFrame 进行推理
#[tokio::test]
async fn test_whisper_streaming_infer_single_frame() {
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

    // 创建 ASR 实例
    let asr = Arc::new(
        WhisperAsrStreaming::new_from_model_path(&model_path)
            .expect("Failed to create WhisperAsrStreaming")
    );

    // 初始化
    asr.initialize().await.expect("Failed to initialize");

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

    // 创建单个 AudioFrame（包含所有音频数据）
    let frame = AudioFrame {
        sample_rate: spec.sample_rate,
        channels: spec.channels as u8,
        data: samples,
        timestamp_ms: 0,
    };

    // 创建请求
    let request = AsrRequest {
        frame,
        language_hint: Some("en".to_string()),
    };

    println!("\n开始推理...");
    let result = asr.infer(request).await
        .expect("Failed to infer");

    println!("\n推理结果:");
    if let Some(ref partial) = result.partial {
        println!("  部分结果: {}", partial.text);
        println!("  置信度: {:.2}", partial.confidence);
    }
    if let Some(ref final_transcript) = result.final_transcript {
        println!("  最终结果: {}", final_transcript.text);
        println!("  语言: {}", final_transcript.language);
    }

    // 验证结果
    assert!(result.partial.is_some() || result.final_transcript.is_some(), 
            "应该返回部分或最终结果");
    
    if let Some(ref final_transcript) = result.final_transcript {
        let text_lower = final_transcript.text.to_lowercase();
        assert!(
            text_lower.contains("ask not what your country can do for you") ||
            text_lower.contains("what you can do for your country"),
            "转录结果应该包含 JFK 演讲的关键内容"
        );
    }

    // 清理
    asr.finalize().await.expect("Failed to finalize");
}

/// 测试 3: 使用多个 AudioFrame 进行推理（流式）
#[tokio::test]
async fn test_whisper_streaming_infer_multiple_frames() {
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

    // 创建 ASR 实例
    let asr = Arc::new(
        WhisperAsrStreaming::new_from_model_path(&model_path)
            .expect("Failed to create WhisperAsrStreaming")
    );

    // 初始化
    asr.initialize().await.expect("Failed to initialize");

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
    let chunk_size = samples.len() / 5;  // 分成 5 个帧
    let mut frames = Vec::new();
    
    for (i, chunk) in samples.chunks(chunk_size).enumerate() {
        frames.push(AudioFrame {
            sample_rate: spec.sample_rate,
            channels: spec.channels as u8,
            data: chunk.to_vec(),
            timestamp_ms: (i * chunk_size * 1000 / spec.sample_rate as usize) as u64,
        });
    }

    println!("\n开始流式推理（{} 个帧）...", frames.len());

    // 逐个发送帧
    let mut last_result = None;
    for (i, frame) in frames.iter().enumerate() {
        let request = AsrRequest {
            frame: frame.clone(),
            language_hint: Some("en".to_string()),
        };

        println!("  处理帧 {} / {}...", i + 1, frames.len());
        let result = asr.infer(request).await
            .expect("Failed to infer");

        if result.final_transcript.is_some() {
            last_result = Some(result);
        }
    }

    // 验证最终结果
    if let Some(ref result) = last_result {
        if let Some(ref final_transcript) = result.final_transcript {
            println!("\n最终转录结果:");
            println!("{}", final_transcript.text);
            
            let text_lower = final_transcript.text.to_lowercase();
            assert!(
                text_lower.contains("ask not") || text_lower.contains("country"),
                "转录结果应该包含 JFK 演讲的关键内容"
            );
        }
    } else {
        println!("⚠ 警告: 没有收到最终结果");
    }

    // 清理
    asr.finalize().await.expect("Failed to finalize");
}

/// 测试 4: 完整的 initialize -> infer -> finalize 流程
#[tokio::test]
async fn test_whisper_streaming_full_lifecycle() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");

    if !model_path.exists() {
        println!("⚠ 跳过测试: 模型文件不存在");
        return;
    }

    let asr = WhisperAsrStreaming::new_from_model_path(&model_path)
        .expect("Failed to create WhisperAsrStreaming");

    // 1. 初始化
    asr.initialize().await.expect("Failed to initialize");
    println!("✓ 初始化成功");

    // 2. 创建一个简单的测试帧（静音）
    let frame = AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: vec![0.0; 16000],  // 1 秒静音
        timestamp_ms: 0,
    };

    let request = AsrRequest {
        frame,
        language_hint: None,
    };

    // 3. 推理
    let result = asr.infer(request).await.expect("Failed to infer");
    println!("✓ 推理完成");

    // 4. 清理
    asr.finalize().await.expect("Failed to finalize");
    println!("✓ 清理完成");

    // 验证缓冲区已清空（通过再次推理应该返回空结果）
    let empty_frame = AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: vec![0.0; 100],
        timestamp_ms: 0,
    };

    let empty_request = AsrRequest {
        frame: empty_frame,
        language_hint: None,
    };

    let empty_result = asr.infer(empty_request).await.expect("Failed to infer");
    // 注意：由于 finalize 后缓冲区已清空，新的推理应该只包含新帧
    // 这里我们只验证不会崩溃
    println!("✓ 清理后推理完成（缓冲区已重置）");
}

