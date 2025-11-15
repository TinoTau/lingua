// tests/asr_whisper_simple_test.rs
// 简化的 Whisper 转录测试，用于探索 API

use std::path::PathBuf;
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

/// 测试：加载模型并尝试转录音频
#[test]
fn test_whisper_simple_transcribe() {
    println!("\n========== Whisper 简单转录测试 ==========");
    
    // 1. 找到音频文件
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");
    
    let wav_path = project_root.join("third_party/whisper.cpp/samples/jfk.wav");
    
    if !wav_path.exists() {
        println!("⚠ 跳过测试: JFK 音频文件不存在");
        return;
    }
    
    // 2. 加载模型
    let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");
    
    if !model_path.exists() {
        println!("⚠ 跳过测试: 模型文件不存在");
        return;
    }
    
    println!("加载模型: {}", model_path.display());
    let ctx = WhisperContext::new_with_params(
        model_path.to_str().unwrap(),
        WhisperContextParameters::default(),
    ).expect("Failed to load model");
    
    println!("✓ 模型加载成功");
    
    // 3. 加载音频（简化版：直接读取 WAV 文件）
    println!("加载音频: {}", wav_path.display());
    
    // 使用 hound 读取 WAV 文件
    let mut reader = hound::WavReader::open(&wav_path)
        .expect("Failed to open WAV file");
    let spec = reader.spec();
    
    println!("  采样率: {} Hz", spec.sample_rate);
    println!("  声道: {}", spec.channels);
    
    // 读取样本并转换为 f32
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
    
    // 转换为单声道（如果需要）
    let audio_data: Vec<f32> = if spec.channels == 2 {
        samples.chunks(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    } else {
        samples
    };
    
    // 简单重采样到 16kHz（如果需要）
    let audio_16k = if spec.sample_rate != 16000 {
        let ratio = 16000.0 / spec.sample_rate as f64;
        let new_len = (audio_data.len() as f64 * ratio) as usize;
        (0..new_len)
            .map(|i| {
                let src_idx = (i as f64 / ratio) as usize;
                audio_data.get(src_idx).copied().unwrap_or(0.0)
            })
            .collect()
    } else {
        audio_data
    };
    
    println!("✓ 音频加载成功 ({} 样本, {:.2} 秒)", 
        audio_16k.len(), 
        audio_16k.len() as f32 / 16000.0);
    
    // 4. 配置参数
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_n_threads(4);
    params.set_translate(false);
    
    // 5. 运行推理
    println!("\n运行推理...");
    let mut state = ctx.create_state()
        .expect("Failed to create state");
    
    state.full(params, &audio_16k)
        .expect("Failed to run inference");
    
    println!("✓ 推理完成");
    
    // 6. 获取结果（尝试不同的 API 方法）
    let num_segments = state.full_n_segments();
    println!("\n找到 {} 个片段", num_segments);
    
    // 使用 get_segment 方法获取结果
    println!("\n========== 转录结果 ==========");
    let mut full_text = String::new();
    
    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            // 使用 Debug 格式输出，然后解析（临时方案）
            // 或者尝试使用可能的方法
            let segment_debug = format!("{:?}", segment);
            
            // 从 Debug 输出中提取文本（临时方案）
            // 实际应该使用正确的 API 方法
            if segment_debug.contains("text: Ok(") {
                // 简单提取文本（临时方案）
                if let Some(start_idx) = segment_debug.find("text: Ok(\"") {
                    let text_start = start_idx + 10;
                    if let Some(end_idx) = segment_debug[text_start..].find("\")") {
                        let text = &segment_debug[text_start..text_start + end_idx];
                        let text_trimmed = text.trim();
                        if !text_trimmed.is_empty() {
                            println!("片段 {}: {}", i, text_trimmed);
                            full_text.push_str(text_trimmed);
                            full_text.push(' ');
                        }
                    }
                }
            } else {
                // 如果无法提取，至少打印 Debug 信息
                println!("片段 {}: {:?}", i, segment);
            }
        }
    }
    
    println!("\n========== 完整转录 ==========");
    println!("{}", full_text.trim());
    
    // 验证结果（JFK 演讲的经典台词）
    let full_text_lower = full_text.to_lowercase();
    let expected_phrases = [
        "ask not what your country can do for you",
        "what you can do for your country",
    ];
    
    println!("\n========== 验证结果 ==========");
    let mut all_found = true;
    for phrase in &expected_phrases {
        let found = full_text_lower.contains(phrase);
        if found {
            println!("✓ 找到: '{}'", phrase);
        } else {
            println!("✗ 未找到: '{}'", phrase);
            all_found = false;
        }
    }
    
    if all_found {
        println!("\n✓ 所有预期短语都找到了！");
    }
    
    // 至少应该有一些输出
    assert!(!full_text.trim().is_empty(), "转录结果不应为空");
}

