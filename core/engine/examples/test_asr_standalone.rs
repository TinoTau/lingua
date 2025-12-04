//! 独立测试程序：验证 ASR 服务（Whisper）
//! 
//! 使用方法：
//!   cargo run --example test_asr_standalone
//! 
//! 前提条件：
//!   1. Whisper 模型已下载
//!   2. test_output 目录中有测试音频文件（chinese.wav, english.wav）

use core_engine::asr_streaming::{AsrStreaming, AsrRequest};
use core_engine::asr_whisper::streaming::WhisperAsrStreaming;
use core_engine::types::AudioFrame;
use std::path::PathBuf;
use std::fs;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ASR 独立测试程序 ===\n");

    // 获取项目根目录
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| manifest_dir.as_path());
    let test_output_dir = project_root.join("test_output");

    // 检查测试音频文件
    let chinese_wav = test_output_dir.join("chinese.wav");
    let english_wav = test_output_dir.join("english.wav");

    if !chinese_wav.exists() && !english_wav.exists() {
        eprintln!("[ERROR] 未找到测试音频文件");
        eprintln!("  请确保以下文件之一存在：");
        eprintln!("    - {}", chinese_wav.display());
        eprintln!("    - {}", english_wav.display());
        return Err("测试音频文件不存在".into());
    }

    // 创建 Whisper ASR 实例
    println!("[1/3] 初始化 Whisper ASR...");
    
    // 尝试从默认模型目录加载
    let model_dir = project_root.join("core").join("engine").join("models").join("asr").join("whisper");
    let asr = if model_dir.exists() {
        WhisperAsrStreaming::new_from_dir(&model_dir)?
    } else {
        eprintln!("[ERROR] 未找到 Whisper 模型目录");
        eprintln!("  请确保模型位于: {}", model_dir.display());
        return Err("Whisper model not found".into());
    };
    
    asr.initialize().await?;
    println!("[OK] ASR 初始化成功\n");

    // 测试中文音频
    if chinese_wav.exists() {
        println!("[2/3] 测试中文音频识别...");
        test_asr_with_file(&asr, &chinese_wav, "zh").await?;
        println!("[OK] 中文识别测试完成\n");
    }

    // 测试英文音频
    if english_wav.exists() {
        println!("[3/3] 测试英文音频识别...");
        test_asr_with_file(&asr, &english_wav, "en").await?;
        println!("[OK] 英文识别测试完成\n");
    }

    // 清理
    asr.finalize().await?;

    println!("=== 测试完成 ===");
    Ok(())
}

async fn test_asr_with_file(
    asr: &WhisperAsrStreaming,
    wav_path: &PathBuf,
    language_hint: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  音频文件: {}", wav_path.display());
    println!("  语言提示: {}", language_hint);

    // 读取 WAV 文件
    let mut wav_file = fs::File::open(wav_path)?;
    let mut wav_data = Vec::new();
    wav_file.read_to_end(&mut wav_data)?;

    // 解析 WAV 文件（简单实现，假设是 16kHz, 16-bit, mono）
    // 注意：这里使用简化实现，实际应该使用专门的 WAV 解析库
    let sample_rate = 16000u32;
    let channels = 1u8;
    
    // 跳过 WAV 头（44 字节），读取 PCM 数据
    let pcm_data = if wav_data.len() > 44 {
        &wav_data[44..]
    } else {
        &wav_data
    };

    // 转换为 f32 音频数据（假设是 16-bit PCM）
    let audio_samples: Vec<f32> = pcm_data
        .chunks_exact(2)
        .map(|chunk| {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            sample as f32 / 32768.0
        })
        .collect();

    println!("  音频长度: {:.2} 秒", audio_samples.len() as f32 / sample_rate as f32);
    println!("  采样点数: {}", audio_samples.len());

    // 将音频分割成帧（每帧 512 samples，约 32ms @ 16kHz）
    let frame_size = 512;
    let mut timestamp_ms = 0u64;
    let mut has_result = false;

    for chunk in audio_samples.chunks(frame_size) {
        let frame = AudioFrame {
            sample_rate,
            channels,
            data: chunk.to_vec(),
            timestamp_ms,
        };

        let request = AsrRequest {
            frame,
            language_hint: Some(language_hint.to_string()),
        };

        let result = asr.infer(request).await?;

        // 显示部分识别结果
        if let Some(ref partial) = result.partial {
            if !has_result {
                println!("  部分识别: {}", partial.text);
                has_result = true;
            }
        }

        // 显示最终识别结果
        if let Some(ref final_transcript) = result.final_transcript {
            println!("  最终识别: {}", final_transcript.text);
            if let Some(ref speaker_id) = final_transcript.speaker_id {
                println!("  说话者ID: {}", speaker_id);
            }
            println!("  语言: {}", final_transcript.language);
        }

        timestamp_ms += (chunk.len() as f32 / sample_rate as f32 * 1000.0) as u64;
    }

    if !has_result {
        println!("  ⚠️  未获得识别结果（可能需要更多音频数据）");
    }

    Ok(())
}

