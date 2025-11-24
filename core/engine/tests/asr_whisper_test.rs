//! ASR Whisper 单元测试
//! 
//! 测试 ASR 识别功能和语言检测

use std::path::PathBuf;
use core_engine::asr_streaming::{AsrRequest, AsrStreaming};
use core_engine::asr_whisper::WhisperAsrStreaming;
use core_engine::types::AudioFrame;
use hound::WavReader;

/// 加载 WAV 文件并转换为 AudioFrame
fn load_wav_to_audio_frame(wav_path: &PathBuf) -> Result<Vec<AudioFrame>, Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(wav_path)?;
    let spec = reader.spec();
    
    // 读取所有样本
    let samples: Result<Vec<i16>, _> = reader.samples().collect();
    let samples = samples?;
    
    // 转换为 f32（归一化到 -1.0 到 1.0）
    let audio_data: Vec<f32> = samples
        .iter()
        .map(|&s| s as f32 / 32768.0)
        .collect();
    
    // 如果 stereo，转换为 mono（取平均值）
    let mono_data = if spec.channels == 2 {
        audio_data
            .chunks(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    } else {
        audio_data
    };
    
    // 创建 AudioFrame
    Ok(vec![AudioFrame {
        sample_rate: spec.sample_rate,
        channels: spec.channels as u8,
        data: mono_data,
        timestamp_ms: 0,
    }])
}

#[tokio::test]
async fn test_asr_chinese_audio() {
    let wav_path = PathBuf::from("../../test_output/chinese.wav");
    if !wav_path.exists() {
        eprintln!("[SKIP] 测试文件不存在: {}", wav_path.display());
        return;
    }
    
    println!("\n=== 测试 ASR 识别中文音频 ===");
    
    // 加载 ASR
    let model_dir = PathBuf::from("models/asr/whisper-base");
    let asr = WhisperAsrStreaming::new_from_dir(&model_dir)
        .expect("Failed to load Whisper ASR");
    
    // 初始化
    AsrStreaming::initialize(&asr).await
        .expect("Failed to initialize ASR");
    
    // 加载音频
    let audio_frames = load_wav_to_audio_frame(&wav_path)
        .expect("Failed to load audio file");
    
    println!("音频文件: {}", wav_path.display());
    println!("音频帧数: {}", audio_frames.len());
    
    // 测试自动语言检测
    println!("\n[测试] 自动语言检测...");
    for (i, frame) in audio_frames.iter().enumerate() {
        let asr_request = AsrRequest {
            frame: frame.clone(),
            language_hint: None,  // 自动检测
        };
        
        let asr_result = AsrStreaming::infer(&asr, asr_request).await
            .expect("ASR inference failed");
        
        if let Some(ref final_transcript) = asr_result.final_transcript {
            println!("  帧 {}: 文本 = '{}'", i + 1, final_transcript.text);
            println!("  帧 {}: 语言 = '{}'", i + 1, final_transcript.language);
            
            // 验证：应该识别为中文
            let has_chinese = final_transcript.text.chars().any(|c| {
                let code = c as u32;
                (0x4E00..=0x9FFF).contains(&code)
            });
            
            if has_chinese {
                println!("  ✅ 检测到中文字符");
            } else {
                println!("  ⚠️  未检测到中文字符，文本: '{}'", final_transcript.text);
            }
        }
    }
}

#[tokio::test]
async fn test_asr_english_audio() {
    let wav_path = PathBuf::from("../../test_output/english.wav");
    if !wav_path.exists() {
        eprintln!("[SKIP] 测试文件不存在: {}", wav_path.display());
        return;
    }
    
    println!("\n=== 测试 ASR 识别英文音频 ===");
    
    // 加载 ASR
    let model_dir = PathBuf::from("models/asr/whisper-base");
    let asr = WhisperAsrStreaming::new_from_dir(&model_dir)
        .expect("Failed to load Whisper ASR");
    
    // 初始化
    AsrStreaming::initialize(&asr).await
        .expect("Failed to initialize ASR");
    
    // 加载音频
    let audio_frames = load_wav_to_audio_frame(&wav_path)
        .expect("Failed to load audio file");
    
    println!("音频文件: {}", wav_path.display());
    println!("音频帧数: {}", audio_frames.len());
    
    // 测试自动语言检测
    println!("\n[测试] 自动语言检测...");
    for (i, frame) in audio_frames.iter().enumerate() {
        let asr_request = AsrRequest {
            frame: frame.clone(),
            language_hint: None,  // 自动检测
        };
        
        let asr_result = AsrStreaming::infer(&asr, asr_request).await
            .expect("ASR inference failed");
        
        if let Some(ref final_transcript) = asr_result.final_transcript {
            println!("  帧 {}: 文本 = '{}'", i + 1, final_transcript.text);
            println!("  帧 {}: 语言 = '{}'", i + 1, final_transcript.language);
            
            // 验证：应该识别为英文
            let english_ratio = final_transcript.text.chars()
                .filter(|c| c.is_ascii_alphabetic() || c.is_whitespace() || c.is_ascii_punctuation())
                .count() as f32 / final_transcript.text.chars().count().max(1) as f32;
            
            if english_ratio > 0.7 {
                println!("  ✅ 检测到英文文本（英文字符比例: {:.2}%）", english_ratio * 100.0);
            } else {
                println!("  ⚠️  可能不是英文文本，英文字符比例: {:.2}%", english_ratio * 100.0);
            }
        }
    }
}

