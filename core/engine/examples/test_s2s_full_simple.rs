//! 完整 S2S 流集成测试（简化版）：使用真实的 ASR 和 NMT
//! 
//! 使用方法：
//!   cargo run --example test_s2s_full_simple -- <input_wav_file>
//! 
//! 示例：
//!   cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav
//! 
//! 前提条件：
//!   1. WSL2 中已启动 Piper HTTP 服务（http://127.0.0.1:5005/tts）
//!   2. Whisper ASR 模型已下载到 core/engine/models/asr/whisper-base/
//!   3. Marian NMT 模型已下载到 core/engine/models/nmt/marian-zh-en/
//!   4. 输入音频文件（WAV 格式）

use std::fs;
use std::path::PathBuf;
use std::env;
use hound::WavReader;
use core_engine::types::AudioFrame;
use core_engine::asr_streaming::{AsrRequest, AsrStreaming};
use core_engine::nmt_incremental::{TranslationRequest, NmtIncremental};
use core_engine::tts_streaming::{TtsRequest, TtsStreaming};

/// 加载 WAV 文件并转换为 AudioFrame
fn load_wav_to_audio_frame(wav_path: &PathBuf) -> Result<Vec<AudioFrame>, Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(wav_path)?;
    let spec = reader.spec();
    
    println!("  WAV 规格:");
    println!("    采样率: {} Hz", spec.sample_rate);
    println!("    声道数: {}", spec.channels);
    println!("    位深: {} bit", spec.bits_per_sample);
    
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
    
    // 将音频数据分割成帧（每帧 16000 样本，约 1 秒 @ 16kHz）
    let frame_size = spec.sample_rate as usize; // 1 秒的样本数
    let mut frames = Vec::new();
    let mut timestamp_ms = 0u64;
    
    for chunk in mono_data.chunks(frame_size) {
        let frame = AudioFrame {
            sample_rate: spec.sample_rate,
            channels: 1,
            data: chunk.to_vec(),
            timestamp_ms,
        };
        frames.push(frame);
        timestamp_ms += 1000; // 每帧 1 秒
    }
    
    // 如果最后一块不足一帧，也添加
    if mono_data.len() % frame_size != 0 {
        let start_idx = (mono_data.len() / frame_size) * frame_size;
        if start_idx < mono_data.len() {
            let frame = AudioFrame {
                sample_rate: spec.sample_rate,
                channels: 1,
                data: mono_data[start_idx..].to_vec(),
                timestamp_ms,
            };
            frames.push(frame);
        }
    }
    
    Ok(frames)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 完整 S2S 流集成测试（简化版 - 真实 ASR + NMT + Piper TTS） ===\n");

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: cargo run --example test_s2s_full_simple -- <input_wav_file>");
        eprintln!("示例: cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav");
        return Err("缺少输入音频文件路径".into());
    }
    
    let input_wav = PathBuf::from(&args[1]);
    if !input_wav.exists() {
        return Err(format!("输入文件不存在: {}", input_wav.display()).into());
    }

    // 检查服务是否运行
    println!("[1/7] 检查 Piper HTTP 服务状态...");
    let health_url = "http://127.0.0.1:5005/health";
    let client = reqwest::Client::new();
    match client.get(health_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("[OK] Piper HTTP 服务正在运行");
            } else {
                eprintln!("[ERROR] 服务返回错误状态: {}", resp.status());
                return Err("Service health check failed".into());
            }
        }
        Err(e) => {
            eprintln!("[ERROR] 无法连接到服务: {}", e);
            eprintln!("[INFO] 请确保 WSL2 中的 Piper HTTP 服务正在运行");
            return Err("Service not available".into());
        }
    }

    // 加载音频文件
    println!("\n[2/7] 加载输入音频文件...");
    println!("  文件路径: {}", input_wav.display());
    let audio_frames = load_wav_to_audio_frame(&input_wav)?;
    println!("[OK] 音频文件加载成功");
    println!("  总帧数: {}", audio_frames.len());
    if let Some(first_frame) = audio_frames.first() {
        println!("  采样率: {} Hz", first_frame.sample_rate);
        println!("  声道数: {}", first_frame.channels);
    }

    // 加载 ASR
    println!("\n[3/7] 加载 Whisper ASR...");
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let asr_model_dir = crate_root.join("models/asr/whisper-base");
    if !asr_model_dir.exists() {
        return Err(format!("Whisper ASR 模型目录不存在: {}", asr_model_dir.display()).into());
    }
    
    let asr = core_engine::asr_whisper::WhisperAsrStreaming::new_from_dir(&asr_model_dir)
        .map_err(|e| format!("Failed to load Whisper ASR: {}", e))?;
    println!("[OK] Whisper ASR 加载成功");
    
    // 初始化 ASR
    AsrStreaming::initialize(&asr).await
        .map_err(|e| format!("Failed to initialize ASR: {}", e))?;

    // 加载 NMT
    println!("\n[4/7] 加载 Marian NMT...");
    let nmt_model_dir = crate_root.join("models/nmt/marian-zh-en");
    if !nmt_model_dir.exists() {
        return Err(format!("Marian NMT 模型目录不存在: {}", nmt_model_dir.display()).into());
    }
    
    let nmt = core_engine::MarianNmtOnnx::new_from_dir(&nmt_model_dir)
        .map_err(|e| format!("Failed to load Marian NMT: {}", e))?;
    println!("[OK] Marian NMT 加载成功");
    
    // 初始化 NMT
    NmtIncremental::initialize(&nmt).await
        .map_err(|e| format!("Failed to initialize NMT: {}", e))?;

    // 步骤 1: ASR 识别
    println!("\n[5/7] 执行 ASR 识别...");
    let mut all_transcripts = Vec::new();
    
    for (i, frame) in audio_frames.iter().enumerate() {
        let asr_request = AsrRequest {
            frame: frame.clone(),
            language_hint: Some("zh".to_string()), // 中文
        };
        
        let asr_result = AsrStreaming::infer(&asr, asr_request).await
            .map_err(|e| format!("ASR inference failed: {}", e))?;
        
        if let Some(ref partial) = asr_result.partial {
            println!("  部分结果 [帧 {}]: {}", i + 1, partial.text);
        }
        
        if let Some(ref final_transcript) = asr_result.final_transcript {
            println!("  最终结果 [帧 {}]: {}", i + 1, final_transcript.text);
            all_transcripts.push(final_transcript.text.clone());
        }
    }
    
    // 合并所有转录结果
    let source_text = all_transcripts.join(" ");
    if source_text.is_empty() {
        return Err("ASR 未返回任何转录结果".into());
    }
    
    println!("[OK] ASR 识别完成");
    println!("  源文本（中文）: {}", source_text);

    // 步骤 2: NMT 翻译
    println!("\n[6/7] 执行 NMT 翻译...");
    
    // 创建 PartialTranscript 用于翻译请求
    let transcript = core_engine::types::PartialTranscript {
        text: source_text.clone(),
        confidence: 1.0,
        is_final: true,
    };
    
    let translation_request = TranslationRequest {
        transcript,
        target_language: "en".to_string(),
        wait_k: None,
    };
    
    let translation_response = NmtIncremental::translate(&nmt, translation_request).await
        .map_err(|e| format!("Translation failed: {}", e))?;
    
    let target_text = translation_response.translated_text;
    println!("[OK] NMT 翻译完成");
    println!("  目标文本（英文）: {}", target_text);
    println!("  翻译稳定: {}", translation_response.is_stable);

    // 步骤 3: TTS 合成（使用 Piper HTTP TTS 合成中文语音）
    println!("\n[7/7] 执行 TTS 合成（Piper HTTP）...");
    println!("  说明: 合成中文语音用于回放源语言");
    
    // 创建 TTS 客户端
    let config = core_engine::tts_streaming::PiperHttpConfig::default();
    let tts_client = core_engine::tts_streaming::PiperHttpTts::new(config)
        .map_err(|e| format!("Failed to create PiperHttpTts: {}", e))?;
    
    let tts_request = TtsRequest {
        text: source_text.clone(), // 使用源文本（中文）进行 TTS
        voice: "zh_CN-huayan-medium".to_string(),
        locale: "zh-CN".to_string(),
    };
    
    let start_time = std::time::Instant::now();
    let chunk = TtsStreaming::synthesize(&tts_client, tts_request).await
        .map_err(|e| format!("TTS synthesis failed: {}", e))?;
    let elapsed = start_time.elapsed();
    
    println!("[OK] TTS 合成成功");
    println!("  耗时: {:?}", elapsed);
    println!("  音频大小: {} 字节", chunk.audio.len());

    // 验证 WAV 格式
    if chunk.audio.len() >= 4 {
        let header = String::from_utf8_lossy(&chunk.audio[0..4]);
        if header == "RIFF" {
            println!("  格式: WAV (RIFF)");
        }
    }

    // 保存到文件
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| manifest_dir.as_path());
    let output_file = project_root.join("test_output").join("s2s_full_simple_test.wav");
    
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&output_file, &chunk.audio)?;
    println!("\n[OK] 音频文件已保存");
    println!("  文件路径: {}", output_file.display());
    println!("  文件大小: {} 字节", fs::metadata(&output_file)?.len());

    // 清理
    AsrStreaming::finalize(&asr).await?;
    NmtIncremental::finalize(&nmt).await?;

    println!("\n=== 测试完成 ===");
    println!("\n完整 S2S 流程测试总结：");
    println!("  ✅ 步骤 1: ASR 识别（真实 Whisper）");
    println!("    输入: 音频文件 {}", input_wav.display());
    println!("    输出: \"{}\"", source_text);
    println!();
    println!("  ✅ 步骤 2: NMT 翻译（真实 Marian）");
    println!("    输入: \"{}\"", source_text);
    println!("    输出: \"{}\"", target_text);
    println!();
    println!("  ✅ 步骤 3: TTS 合成（Piper HTTP）");
    println!("    输入: \"{}\"", source_text);
    println!("    输出: {} 字节 WAV 音频", chunk.audio.len());
    println!();
    println!("  完整流程: 中文语音 → 中文文本 → 英文文本 → 中文语音");
    println!();
    println!("下一步：");
    println!("  1. 播放音频文件验证语音质量: {}", output_file.display());
    println!("  2. 验证翻译准确性");
    println!("  3. 如果正常，完整的 S2S 流程已验证通过");

    Ok(())
}

