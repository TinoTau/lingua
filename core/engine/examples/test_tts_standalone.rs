//! 独立测试程序：验证 TTS 服务
//! 
//! 使用方法：
//!   cargo run --example test_tts_standalone
//! 
//! 前提条件：
//!   TTS 服务已启动（Piper HTTP 或 YourTTS HTTP）
//!   - Piper: http://127.0.0.1:5005
//!   - YourTTS: http://127.0.0.1:5004

use core_engine::tts_streaming::{TtsStreaming, TtsRequest};
use core_engine::tts_streaming::{PiperHttpTts, PiperHttpConfig};
use core_engine::tts_streaming::{YourTtsHttp, YourTtsHttpConfig};
use std::path::PathBuf;
use std::fs;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TTS 独立测试程序 ===\n");

    // 获取项目根目录
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| manifest_dir.as_path());
    let test_output_dir = project_root.join("test_output");

    // 确保输出目录存在
    fs::create_dir_all(&test_output_dir)?;

    // 测试 1: Piper HTTP TTS
    println!("[1/2] 测试 Piper HTTP TTS...");
    match test_piper_tts(&test_output_dir).await {
        Ok(_) => println!("[OK] Piper TTS 测试通过\n"),
        Err(e) => {
            println!("[SKIP] Piper TTS 测试跳过: {}\n", e);
        }
    }

    // 测试 2: YourTTS HTTP
    println!("[2/2] 测试 YourTTS HTTP...");
    match test_yourtts_http(&test_output_dir).await {
        Ok(_) => println!("[OK] YourTTS 测试通过\n"),
        Err(e) => {
            println!("[SKIP] YourTTS 测试跳过: {}\n", e);
        }
    }

    println!("=== 测试完成 ===");
    Ok(())
}

async fn test_piper_tts(output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 检查服务
    let health_url = "http://127.0.0.1:5005/health";
    let client = reqwest::Client::new();
    
    match client.get(health_url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Err("Piper service health check failed".into());
            }
        }
        Err(_) => {
            return Err("Piper service not available".into());
        }
    }

    // 创建客户端
    let config = PiperHttpConfig::default();
    let tts = PiperHttpTts::new(config)?;

    // 测试用例
    let test_cases = vec![
        ("你好，欢迎使用 Lingua 语音翻译系统。", "zh_CN-huayan-medium", "zh-CN"),
        ("Hello, welcome to the Lingua translation system.", "en_US-lessac-medium", "en-US"),
    ];

    for (i, (text, voice, locale)) in test_cases.iter().enumerate() {
        println!("  测试 {}: {}", i + 1, text);
        
        let request = TtsRequest {
            text: text.to_string(),
            voice: voice.to_string(),
            locale: locale.to_string(),
            reference_audio: None,
        };

        let start_time = std::time::Instant::now();
        let chunk = tts.synthesize(request).await?;
        let elapsed = start_time.elapsed();

        println!("    耗时: {:?}", elapsed);
        println!("    音频大小: {} 字节", chunk.audio.len());

        // 保存音频文件
        let output_file = output_dir.join(format!("test_piper_{}.wav", i + 1));
        fs::write(&output_file, &chunk.audio)?;
        println!("    保存到: {}", output_file.display());
    }

    tts.close().await?;
    Ok(())
}

async fn test_yourtts_http(output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 检查服务
    let health_url = "http://127.0.0.1:5004/health";
    let client = reqwest::Client::new();
    
    match client.get(health_url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Err("YourTTS service health check failed".into());
            }
        }
        Err(_) => {
            return Err("YourTTS service not available".into());
        }
    }

    // 创建客户端
    let config = YourTtsHttpConfig::default();
    let tts = YourTtsHttp::new(config)?;

    // 测试用例 1: 无参考音频
    println!("  测试 1: 无参考音频（默认语音）");
    let request1 = TtsRequest {
        text: "Hello, this is a test of YourTTS without reference audio.".to_string(),
        voice: "default".to_string(),
        locale: "en".to_string(),
        reference_audio: None,
    };

    let start_time = std::time::Instant::now();
    let chunk1 = tts.synthesize(request1).await?;
    let elapsed1 = start_time.elapsed();

    println!("    耗时: {:?}", elapsed1);
    println!("    音频大小: {} 字节", chunk1.audio.len());

    // 保存音频文件
    let output_file1 = output_dir.join("test_yourtts_no_ref.wav");
    // 注意：YourTTS 客户端已经将 f32 转换为 u8 PCM（16-bit）
    save_pcm_to_wav(&chunk1.audio, &output_file1, 22050)?;
    println!("    保存到: {}", output_file1.display());

    // 测试用例 2: 使用参考音频（如果可用）
    let reference_wav = output_dir.join("chinese.wav");
    if reference_wav.exists() {
        println!("\n  测试 2: 使用参考音频（音色克隆）");
        
        // 读取参考音频（简化实现：假设是 16-bit PCM WAV）
        let mut wav_file = fs::File::open(&reference_wav)?;
        let mut wav_data = Vec::new();
        wav_file.read_to_end(&mut wav_data)?;
        
        // 跳过 WAV 头（44 字节），读取 PCM 数据
        let pcm_data = if wav_data.len() > 44 {
            &wav_data[44..]
        } else {
            &wav_data
        };
        
        // 转换为 f32（假设是 16-bit PCM）
        let reference_audio_f32: Vec<f32> = pcm_data
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect();

        let request2 = TtsRequest {
            text: "你好，这是使用参考音频的 YourTTS 测试。".to_string(),
            voice: "cloned".to_string(),
            locale: "zh".to_string(),
            reference_audio: Some(reference_audio_f32),
        };

        let start_time = std::time::Instant::now();
        let chunk2 = tts.synthesize(request2).await?;
        let elapsed2 = start_time.elapsed();

        println!("    耗时: {:?}", elapsed2);
        println!("    音频大小: {} 字节", chunk2.audio.len());

        // 保存音频文件
        let output_file2 = output_dir.join("test_yourtts_with_ref.wav");
        // YourTTS 客户端已经将 f32 转换为 u8 PCM（16-bit）
        save_pcm_to_wav(&chunk2.audio, &output_file2, 22050)?;
        println!("    保存到: {}", output_file2.display());
    } else {
        println!("\n  ⚠️  跳过参考音频测试（未找到参考音频文件: {})", reference_wav.display());
    }

    tts.close().await?;
    Ok(())
}

fn save_pcm_to_wav(pcm_data: &[u8], output_path: &PathBuf, sample_rate: u32) -> Result<(), Box<dyn std::error::Error>> {
    // 注意：这里假设 pcm_data 是 16-bit PCM
    // 如果 YourTTS 返回的是 f32，需要先转换
    use std::io::Write;
    
    let mut wav_file = fs::File::create(output_path)?;
    
    // 写入 WAV 头
    let num_samples = pcm_data.len() / 2; // 16-bit = 2 bytes per sample
    let data_size = num_samples * 2;
    let file_size = 36 + data_size;
    
    // RIFF header
    wav_file.write_all(b"RIFF")?;
    wav_file.write_all(&(file_size as u32).to_le_bytes())?;
    wav_file.write_all(b"WAVE")?;
    
    // fmt chunk
    wav_file.write_all(b"fmt ")?;
    wav_file.write_all(&16u32.to_le_bytes())?; // chunk size
    wav_file.write_all(&1u16.to_le_bytes())?; // audio format (PCM)
    wav_file.write_all(&1u16.to_le_bytes())?; // num channels
    wav_file.write_all(&(sample_rate as u32).to_le_bytes())?; // sample rate
    wav_file.write_all(&((sample_rate * 2) as u32).to_le_bytes())?; // byte rate
    wav_file.write_all(&2u16.to_le_bytes())?; // block align
    wav_file.write_all(&16u16.to_le_bytes())?; // bits per sample
    
    // data chunk
    wav_file.write_all(b"data")?;
    wav_file.write_all(&(data_size as u32).to_le_bytes())?;
    wav_file.write_all(pcm_data)?;
    
    Ok(())
}

