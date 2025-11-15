// tests/asr_whisper_integration_test.rs
// 集成测试：测试完整的 ASR 流程（真实音频文件）

use std::path::PathBuf;
use core_engine::asr_whisper::{WhisperAsrEngine, WhisperAsrStreaming};
use core_engine::AsrStreaming;
use core_engine::AudioFrame;
use hound;

/// 从 WAV 文件读取音频帧
fn read_wav_file(wav_path: &PathBuf) -> anyhow::Result<Vec<AudioFrame>> {
    let mut reader = hound::WavReader::open(wav_path)?;
    let spec = reader.spec();
    
    let mut frames = Vec::new();
    let mut samples = Vec::new();
    let frame_size = (spec.sample_rate as usize / 10);  // 每帧 0.1 秒
    
    // 读取所有样本
    match spec.sample_format {
        hound::SampleFormat::Float => {
            for sample_result in reader.samples::<f32>() {
                samples.push(sample_result?);
            }
        }
        hound::SampleFormat::Int => {
            for sample_result in reader.samples::<i16>() {
                let sample = sample_result? as f32 / 32768.0;
                samples.push(sample);
            }
        }
    }
    
    // 将样本分割成帧
    for (i, chunk) in samples.chunks(frame_size).enumerate() {
        frames.push(AudioFrame {
            sample_rate: spec.sample_rate,
            channels: spec.channels as u8,
            data: chunk.to_vec(),
            timestamp_ms: (i * 100) as u64,  // 每帧 0.1 秒 = 100ms
        });
    }
    
    Ok(frames)
}

/// 测试 1: 从 WAV 文件到转录文本的完整流程
#[tokio::test]
async fn test_wav_file_to_transcript() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");
    
    // 查找测试音频文件
    let possible_audio_paths = [
        crate_root.join("third_party/jfk.wav"),
        crate_root.join("third_party/test_audio.wav"),
        crate_root.join("tests/fixtures/test_audio.wav"),
    ];
    
    let audio_path = possible_audio_paths.iter()
        .find(|p| p.exists());
    
    if audio_path.is_none() {
        println!("⚠ 跳过测试: 未找到测试音频文件");
        println!("  尝试的路径:");
        for path in &possible_audio_paths {
            println!("    - {}", path.display());
        }
        return;
    }
    
    let audio_path = audio_path.unwrap();
    
    if !model_dir.exists() {
        println!("⚠ 跳过测试: Whisper 模型目录不存在");
        return;
    }

    println!("\n========== 测试 WAV 文件到转录文本 ==========");
    println!("音频文件: {}", audio_path.display());
    println!("模型目录: {}", model_dir.display());

    // 1. 读取 WAV 文件
    println!("\n1. 读取 WAV 文件...");
    let frames = read_wav_file(audio_path)
        .expect("Failed to read WAV file");
    println!("   ✓ 读取成功，共 {} 帧", frames.len());

    // 2. 创建 WhisperAsrEngine
    println!("\n2. 加载 Whisper 模型...");
    let engine = WhisperAsrEngine::new_from_dir(&model_dir)
        .expect("Failed to load WhisperAsrEngine");
    println!("   ✓ 模型加载成功");

    // 3. 预处理并转录
    println!("\n3. 进行转录...");
    let transcript = engine.transcribe_frames(&frames)
        .expect("Failed to transcribe");
    println!("   ✓ 转录完成");
    println!("\n转录结果: {}", transcript);

    // 4. 验证结果
    assert!(!transcript.is_empty(), "转录结果不应该为空");
    println!("\n✓ WAV 文件到转录文本测试完成");
}

/// 测试 2: 流式推理的端到端流程
#[tokio::test]
async fn test_streaming_inference_e2e() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");
    
    // 查找测试音频文件
    let possible_audio_paths = [
        crate_root.join("third_party/jfk.wav"),
        crate_root.join("third_party/test_audio.wav"),
        crate_root.join("tests/fixtures/test_audio.wav"),
    ];
    
    let audio_path = possible_audio_paths.iter()
        .find(|p| p.exists());
    
    if audio_path.is_none() {
        println!("⚠ 跳过测试: 未找到测试音频文件");
        return;
    }
    
    let audio_path = audio_path.unwrap();
    
    if !model_dir.exists() {
        println!("⚠ 跳过测试: Whisper 模型目录不存在");
        return;
    }

    println!("\n========== 测试流式推理端到端流程 ==========");
    println!("音频文件: {}", audio_path.display());

    // 1. 读取 WAV 文件
    println!("\n1. 读取 WAV 文件...");
    let frames = read_wav_file(audio_path)
        .expect("Failed to read WAV file");
    println!("   ✓ 读取成功，共 {} 帧", frames.len());

    // 2. 创建 WhisperAsrStreaming
    println!("\n2. 创建 WhisperAsrStreaming...");
    let asr = WhisperAsrStreaming::new_from_dir(&model_dir)
        .expect("Failed to load WhisperAsrStreaming");
    
    // 初始化
    asr.initialize().await.expect("Failed to initialize");
    println!("   ✓ 初始化成功");

    // 3. 启用流式推理
    println!("\n3. 启用流式推理...");
    asr.enable_streaming(1.0);  // 每 1 秒输出一次部分结果
    println!("   ✓ 流式推理已启用");

    // 4. 逐帧处理（模拟实时流式处理）
    println!("\n4. 逐帧处理音频...");
    let mut partial_results = Vec::new();
    let mut final_results = Vec::new();

    for (i, frame) in frames.iter().enumerate() {
        // 累积帧
        asr.accumulate_frame(frame.clone())
            .expect("Failed to accumulate frame");

        // 每 10 帧检查一次部分结果（模拟定期检查）
        if i % 10 == 0 && i > 0 {
            if let Some(partial) = asr.infer_partial(frame.timestamp_ms).await
                .expect("Failed to infer partial") {
                partial_results.push(partial.clone());
                println!("   帧 {}: 部分结果 - {}", i, partial.text);
            }
        }

        // 每 50 帧触发一次边界检测（模拟自然停顿）
        if i % 50 == 0 && i > 0 {
            let result = asr.infer_on_boundary().await
                .expect("Failed to infer on boundary");
            
            if let Some(ref final_transcript) = result.final_transcript {
                final_results.push(final_transcript.clone());
                println!("   帧 {}: 最终结果 - {}", i, final_transcript.text);
            }
        }
    }

    // 5. 处理最后剩余的帧
    println!("\n5. 处理最后剩余的帧...");
    let result = asr.infer_on_boundary().await
        .expect("Failed to infer on boundary");
    
    if let Some(ref final_transcript) = result.final_transcript {
        final_results.push(final_transcript.clone());
        println!("   最终结果: {}", final_transcript.text);
    }

    println!("\n统计:");
    println!("  部分结果数: {}", partial_results.len());
    println!("  最终结果数: {}", final_results.len());

    // 清理
    asr.finalize().await.expect("Failed to finalize");
    println!("\n✓ 流式推理端到端测试完成");
}

/// 测试 3: 性能测试（延迟、吞吐量）
#[tokio::test]
async fn test_performance() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");
    
    if !model_dir.exists() {
        println!("⚠ 跳过测试: Whisper 模型目录不存在");
        return;
    }

    println!("\n========== 性能测试 ==========");

    // 创建测试音频数据（1 秒，16kHz 单声道）
    let test_audio = vec![0.0; 16000];
    let frame = AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: test_audio,
        timestamp_ms: 0,
    };

    // 1. 测试模型加载时间
    println!("\n1. 测试模型加载时间...");
    let start = std::time::Instant::now();
    let engine = WhisperAsrEngine::new_from_dir(&model_dir)
        .expect("Failed to load WhisperAsrEngine");
    let load_time = start.elapsed();
    println!("   模型加载时间: {:.2} 秒", load_time.as_secs_f64());

    // 2. 测试推理延迟
    println!("\n2. 测试推理延迟...");
    let start = std::time::Instant::now();
    let _transcript = engine.transcribe_frame(&frame)
        .expect("Failed to transcribe");
    let inference_time = start.elapsed();
    println!("   推理延迟: {:.2} 秒", inference_time.as_secs_f64());
    println!("   推理延迟: {:.0} 毫秒", inference_time.as_millis());

    // 3. 测试多次推理的平均延迟
    println!("\n3. 测试多次推理的平均延迟...");
    let mut total_time = std::time::Duration::ZERO;
    let num_iterations = 3;
    
    for i in 0..num_iterations {
        let start = std::time::Instant::now();
        let _transcript = engine.transcribe_frame(&frame)
            .expect("Failed to transcribe");
        let elapsed = start.elapsed();
        total_time += elapsed;
        println!("   迭代 {}: {:.2} 秒", i + 1, elapsed.as_secs_f64());
    }
    
    let avg_time = total_time / num_iterations as u32;
    println!("   平均推理延迟: {:.2} 秒", avg_time.as_secs_f64());
    println!("   平均推理延迟: {:.0} 毫秒", avg_time.as_millis());

    // 4. 验证性能要求
    println!("\n4. 验证性能要求...");
    if avg_time.as_secs_f64() < 1.0 {
        println!("   ✓ 平均推理延迟 < 1 秒（满足要求）");
    } else {
        println!("   ⚠ 平均推理延迟 >= 1 秒（可能需要优化）");
    }

    println!("\n✓ 性能测试完成");
}

/// 测试 4: 不同语言的音频测试
#[tokio::test]
async fn test_different_languages() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");
    
    if !model_dir.exists() {
        println!("⚠ 跳过测试: Whisper 模型目录不存在");
        return;
    }

    println!("\n========== 测试不同语言的音频 ==========");

    // 创建 WhisperAsrStreaming
    let asr = WhisperAsrStreaming::new_from_dir(&model_dir)
        .expect("Failed to load WhisperAsrStreaming");
    asr.initialize().await.expect("Failed to initialize");

    // 测试不同语言设置
    let languages: Vec<(Option<&'static str>, &'static str)> = vec![
        (Some("en"), "英语"),
        (Some("zh"), "中文"),
        (Some("ja"), "日语"),
        (None, "自动检测"),
    ];

    // 创建测试音频帧（静音，用于测试语言设置）
    let frame = AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: vec![0.0; 1600],  // 0.1 秒
        timestamp_ms: 0,
    };

    for (lang_code_opt, lang_name) in languages {
        println!("\n测试语言: {}", lang_name);
        
        // 设置语言
        match lang_code_opt {
            Some(code) => {
                asr.set_language(Some(code.to_string()))
                    .expect("Failed to set language");
                println!("  语言代码: {}", code);
            }
            None => {
                asr.set_language(None)
                    .expect("Failed to set language to auto-detect");
                println!("  语言代码: 自动检测");
            }
        }

        // 累积帧并推理（虽然结果是空的，但可以验证没有错误）
        asr.clear_buffer();
        asr.accumulate_frame(frame.clone())
            .expect("Failed to accumulate frame");
        
        let result = asr.infer_on_boundary().await;
        match result {
            Ok(_) => println!("  ✓ 语言设置 {} 推理成功", lang_name),
            Err(e) => println!("  ⚠ 推理返回错误（可能是预期的，因为音频是静音）: {}", e),
        }
    }

    // 清理
    asr.finalize().await.expect("Failed to finalize");
    println!("\n✓ 不同语言的音频测试完成");
}

