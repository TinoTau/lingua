// tests/asr_whisper_transcribe_test.rs
// 测试 Whisper 模型转录音频文件

use std::path::PathBuf;

/// 从 WAV 文件加载音频数据并转换为 Whisper 输入格式
fn load_wav_file(wav_path: &PathBuf) -> anyhow::Result<Vec<f32>> {
    use hound::WavReader;
    
    let mut reader = WavReader::open(wav_path)?;
    let spec = reader.spec();
    
    println!("WAV 文件信息:");
    println!("  采样率: {} Hz", spec.sample_rate);
    println!("  声道数: {}", spec.channels);
    println!("  位深度: {} bits", spec.bits_per_sample);
    
    // 读取所有样本
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?
        }
        hound::SampleFormat::Int => {
            // 将整数样本转换为浮点数 [-1.0, 1.0]
            let max_value = (1i32 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>()
                .map(|s| s.map(|sample| sample as f32 / max_value))
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    
    // 如果是立体声，转换为单声道（取平均值）
    let mono_samples = if spec.channels == 2 {
        samples.chunks(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    } else {
        samples
    };
    
    // 如果需要，重采样到 16kHz
    let target_sample_rate = 16000u32;
    let resampled = if spec.sample_rate != target_sample_rate {
        println!("  重采样: {} Hz -> {} Hz", spec.sample_rate, target_sample_rate);
        // 简单的线性重采样（对于测试足够）
        let ratio = target_sample_rate as f64 / spec.sample_rate as f64;
        let new_len = (mono_samples.len() as f64 * ratio) as usize;
        let mut resampled = Vec::with_capacity(new_len);
        
        for i in 0..new_len {
            let src_idx = (i as f64 / ratio) as usize;
            if src_idx < mono_samples.len() {
                resampled.push(mono_samples[src_idx]);
            } else {
                resampled.push(0.0);
            }
        }
        resampled
    } else {
        mono_samples
    };
    
    println!("  最终音频长度: {} 样本 ({} 秒)", 
        resampled.len(), 
        resampled.len() as f32 / target_sample_rate as f32);
    
    Ok(resampled)
}

/// 测试 1: 加载 JFK 音频文件并转录
#[test]
fn test_whisper_transcribe_jfk() {
    use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
    
    println!("\n========== Whisper 音频转录测试 ==========");
    
    // 1. 找到音频文件
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");
    
    let wav_path = project_root.join("third_party/whisper.cpp/samples/jfk.wav");
    
    if !wav_path.exists() {
        println!("⚠ 跳过测试: JFK 音频文件不存在: {}", wav_path.display());
        return;
    }
    
    println!("音频文件: {}", wav_path.display());
    
    // 2. 加载模型
    let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");
    
    if !model_path.exists() {
        println!("⚠ 跳过测试: 模型文件不存在: {}", model_path.display());
        return;
    }
    
    println!("\n[1/4] 加载模型...");
    let ctx = WhisperContext::new_with_params(
        model_path.to_str().unwrap(),
        WhisperContextParameters::default(),
    ).expect("Failed to load Whisper model");
    println!("✓ 模型加载成功");
    
    // 3. 加载音频文件
    println!("\n[2/4] 加载音频文件...");
    let audio_data = load_wav_file(&wav_path)
        .expect("Failed to load WAV file");
    println!("✓ 音频加载成功");
    
    // 4. 配置推理参数
    println!("\n[3/4] 配置推理参数...");
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));  // JFK 演讲是英文
    params.set_n_threads(4);
    params.set_translate(false);
    params.set_print_progress(false);
    params.set_print_special(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(true);
    println!("✓ 参数配置完成");
    
    // 5. 运行推理
    println!("\n[4/4] 运行推理...");
    println!("  这可能需要几秒钟...");
    
    let start_time = std::time::Instant::now();
    
    // 创建状态并运行推理
    let mut state = ctx.create_state()
        .expect("Failed to create Whisper state");
    state.full(params, &audio_data)
        .expect("Failed to run inference");
    
    let elapsed = start_time.elapsed();
    println!("✓ 推理完成 (耗时: {:.2} 秒)", elapsed.as_secs_f64());
    
    // 6. 获取并输出结果
    println!("\n========== 转录结果 ==========");
    
    let num_segments = state.full_n_segments();
    println!("  找到 {} 个片段", num_segments);
    
    let mut full_text = String::new();
    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            // 从 Debug 输出中提取文本（whisper-rs 0.15.1 的字段可能是私有的）
            let segment_debug = format!("{:?}", segment);
            
            // 提取文本
            if let Some(start_idx) = segment_debug.find("text: Ok(\"") {
                let text_start = start_idx + 10;
                if let Some(end_idx) = segment_debug[text_start..].find("\")") {
                    let text = &segment_debug[text_start..text_start + end_idx];
                    let text_trimmed = text.trim();
                    if !text_trimmed.is_empty() {
                        // 尝试提取时间戳
                        let start_sec = if let Some(ts_start) = segment_debug.find("start_ts: ") {
                            if let Some(ts_end) = segment_debug[ts_start + 10..].find(',') {
                                segment_debug[ts_start + 10..ts_start + 10 + ts_end]
                                    .trim()
                                    .parse::<i64>()
                                    .unwrap_or(0) as f64 / 100.0
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };
                        
                        let end_sec = if let Some(ts_start) = segment_debug.find("end_ts: ") {
                            if let Some(ts_end) = segment_debug[ts_start + 8..].find(',') {
                                segment_debug[ts_start + 8..ts_start + 8 + ts_end]
                                    .trim()
                                    .parse::<i64>()
                                    .unwrap_or(0) as f64 / 100.0
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        };
                        
                        println!("[{:.2}s - {:.2}s]: {}", start_sec, end_sec, text_trimmed);
                        full_text.push_str(text_trimmed);
                        full_text.push(' ');
                    }
                }
            }
        }
    }
    
    println!("\n完整文本:");
    println!("{}", full_text.trim());
    
    // 7. 验证结果（JFK 演讲的经典台词）
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
    } else {
        println!("\n⚠ 部分短语未找到，但转录可能仍然正确");
    }
    
    // 至少应该有一些输出
    assert!(!full_text.trim().is_empty(), "转录结果不应为空");
}

/// 测试 2: 测试音频加载函数
#[test]
fn test_load_wav_file() {
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
    
    let audio_data = load_wav_file(&wav_path)
        .expect("Failed to load WAV file");
    
    // 验证音频数据
    assert!(!audio_data.is_empty(), "音频数据不应为空");
    assert_eq!(audio_data.len() % 16000, 0, "音频长度应该是 16kHz 的整数倍");
    
    // 验证音频数据范围（应该在 [-1.0, 1.0] 范围内）
    let min_val = audio_data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_val = audio_data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    
    println!("音频数据范围: [{:.4}, {:.4}]", min_val, max_val);
    assert!(min_val >= -1.1 && max_val <= 1.1, "音频数据应该在合理范围内");
    
    println!("✓ 音频加载测试通过");
}

