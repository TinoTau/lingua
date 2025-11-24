//! 完整 S2S 流集成测试（使用 HTTP NMT 客户端）
//! 
//! 使用方法：
//!   cargo run --example test_s2s_full_simple_http -- <input_wav_file> [--direction <en-zh|zh-en>]
//! 
//! 示例：
//!   cargo run --example test_s2s_full_simple_http -- test_output/s2s_flow_test.wav --direction en-zh
//! 
//! 前提条件：
//!   1. Python M2M100 NMT 服务已启动（http://127.0.0.1:5008）
//!   2. WSL2 中已启动 Piper HTTP 服务（http://127.0.0.1:5005/tts）
//!   3. Whisper ASR 模型已下载到 core/engine/models/asr/whisper-base/
//!   4. 输入音频文件（WAV 格式）

use std::path::PathBuf;
use std::env;
use std::fs;
use hound::WavReader;
use core_engine::types::AudioFrame;
use core_engine::asr_streaming::{AsrRequest, AsrStreaming};
use core_engine::nmt_incremental::{TranslationRequest, NmtIncremental};
use core_engine::tts_streaming::{TtsRequest, TtsStreaming};
use core_engine::asr_whisper::WhisperAsrStreaming;
use core_engine::tts_streaming::{PiperHttpTts, PiperHttpConfig};

/// 加载 WAV 文件并转换为 AudioFrame
fn load_wav_to_audio_frame(wav_path: &PathBuf) -> Result<Vec<AudioFrame>, Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(wav_path)?;
    let spec = reader.spec();
    
    println!("  WAV 规格:");
    println!("    采样率: {} Hz", spec.sample_rate);
    println!("    声道数: {}", spec.channels);
    println!("    位深: {} bit", spec.bits_per_sample);
    
    // 支持多种音频格式
    let audio_data: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?
        }
        hound::SampleFormat::Int => {
            let max_val = (1i32 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>()
                .map(|s| s.map(|sample| sample as f32 / max_val))
                .collect::<Result<Vec<_>, _>>()?
        }
    };
    
    let mono_data = if spec.channels == 2 {
        audio_data
            .chunks(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    } else {
        audio_data
    };
    
    let frame_size = spec.sample_rate as usize;
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
        timestamp_ms += 1000;
    }
    
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
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("用法: cargo run --example test_s2s_full_simple_http -- <input_wav_file> [--direction <en-zh|zh-en>]");
        eprintln!("示例: cargo run --example test_s2s_full_simple_http -- test_output/s2s_flow_test.wav --direction en-zh");
        return Ok(());
    }
    
    let wav_path = PathBuf::from(&args[1]);
    let mut direction = "en-zh";
    
    for i in 2..args.len() {
        if args[i] == "--direction" && i + 1 < args.len() {
            direction = &args[i + 1];
        }
    }
    
    println!("=== S2S 完整流程测试（HTTP NMT 客户端）===\n");
    println!("输入文件: {}", wav_path.display());
    println!("翻译方向: {}\n", direction);
    
    // 1. 加载 WAV 文件
    println!("[1/5] 加载音频文件...");
    let audio_frames = load_wav_to_audio_frame(&wav_path)?;
    println!("  ✅ 加载成功，共 {} 帧\n", audio_frames.len());
    
    // 2. 初始化 ASR
    println!("[2/5] 初始化 Whisper ASR...");
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let asr_model_dir = crate_root.join("models/asr/whisper-base");
    
    if !asr_model_dir.exists() {
        return Err(format!("Whisper ASR 模型目录不存在: {}", asr_model_dir.display()).into());
    }
    
    let asr = WhisperAsrStreaming::new_from_dir(&asr_model_dir)
        .map_err(|e| format!("Failed to load Whisper ASR: {}", e))?;
    asr.initialize().await
        .map_err(|e| format!("Failed to initialize ASR: {}", e))?;
    println!("  ✅ ASR 初始化成功\n");
    
    // 3. 初始化 NMT（使用 HTTP 客户端）
    println!("[3/5] 初始化 M2M100 HTTP NMT 客户端...");
    let nmt = core_engine::CoreEngineBuilder::new()
        .nmt_with_m2m100_http_client(Some("http://127.0.0.1:5008"))
        .map_err(|e| format!("Failed to create NMT client: {}", e))?;
    
    // 从 builder 中提取 nmt（这里需要重新设计，暂时直接创建）
    use std::sync::Arc;
    use core_engine::nmt_client::{LocalM2m100HttpClient, NmtClientAdapter};
    let nmt_client = Arc::new(LocalM2m100HttpClient::new("http://127.0.0.1:5008"));
    let nmt = Arc::new(NmtClientAdapter::new(nmt_client));
    
    nmt.initialize().await
        .map_err(|e| format!("Failed to initialize NMT: {}", e))?;
    println!("  ✅ NMT 客户端初始化成功\n");
    
    // 4. 初始化 TTS
    println!("[4/5] 初始化 Piper HTTP TTS...");
    let tts_config = PiperHttpConfig::default();
    let tts = PiperHttpTts::new(tts_config)
        .map_err(|e| format!("Failed to create Piper TTS: {}", e))?;
    println!("  ✅ TTS 初始化成功\n");
    
    // 5. 处理音频帧
    println!("[5/5] 处理音频帧...");
    let mut all_transcripts = Vec::new();
    
    for (idx, frame) in audio_frames.iter().enumerate() {
        println!("  处理帧 {}/{}...", idx + 1, audio_frames.len());
        
        // ASR
        let asr_request = AsrRequest {
            frame: frame.clone(),
            language_hint: None,
        };
        let asr_result = asr.infer(asr_request).await?;
        
        if let Some(final_transcript) = asr_result.final_transcript {
            println!("    ASR 输出: {}", final_transcript.text);
            all_transcripts.push(final_transcript.text.clone());
            
            // NMT
            let target_lang = if direction == "en-zh" { "zh" } else { "en" };
            let translation_request = TranslationRequest {
                transcript: core_engine::types::PartialTranscript {
                    text: final_transcript.text.clone(),
                    confidence: 1.0,
                    is_final: true,
                },
                target_language: target_lang.to_string(),
                wait_k: None,
            };
            
            let translation_result = nmt.translate(translation_request).await?;
            println!("    NMT 输出: {}", translation_result.translated_text);
            
            // TTS：根据目标语言选择合适的语音模型
            // 问题修复：之前总是使用中文语音，导致英文文本无法正确发音
            let (tts_voice, tts_locale) = if target_lang == "zh" {
                ("zh_CN-huayan-medium", "zh")
            } else {
                // 英文目标语言：尝试使用英文语音模型
                ("en_US-lessac-medium", "en")
            };
            
            let tts_request = TtsRequest {
                text: translation_result.translated_text.clone(),
                voice: tts_voice.to_string(),
                locale: tts_locale.to_string(),
            };
            println!("    TTS 请求: voice={}, locale={}, text=\"{}\"", 
                tts_request.voice, tts_request.locale, tts_request.text);
            
            // 尝试使用目标语言的语音模型
            let mut tts_success = false;
            match tts.synthesize(tts_request.clone()).await {
                Ok(result) => {
                    println!("    ✅ TTS 完成，生成音频长度: {} 字节", result.audio.len());
                    println!("      时间戳: {} ms, 是否最后: {}", result.timestamp_ms, result.is_last);
                    
                    // 保存音频文件
                    let output_dir = PathBuf::from("test_output");
                    if !output_dir.exists() {
                        fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
                            eprintln!("警告: 无法创建输出目录: {}", e);
                        });
                    }
                    
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let output_file = output_dir.join(format!("tts_output_{}_{}_{}.wav", 
                        idx, target_lang, timestamp));
                    match fs::write(&output_file, &result.audio) {
                        Ok(_) => {
                            println!("    💾 音频已保存: {}", output_file.display());
                        },
                        Err(e) => {
                            println!("    ⚠️  保存音频失败: {}", e);
                        }
                    }
                    tts_success = true;
                },
                Err(e) => {
                    println!("    ❌ TTS 失败: {}", e);
                    
                    // 如果英文语音模型不可用，回退到中文语音模型
                    if target_lang == "en" {
                        println!("    ⚠️  英文语音模型不可用，尝试使用中文语音模型作为回退...");
                        
                        let fallback_request = TtsRequest {
                            text: translation_result.translated_text.clone(),
                            voice: "zh_CN-huayan-medium".to_string(),
                            locale: "zh".to_string(),
                        };
                        
                        match tts.synthesize(fallback_request).await {
                            Ok(result) => {
                                println!("    ⚠️  使用中文语音模型生成（发音可能不准确），音频长度: {} 字节", result.audio.len());
                                
                                // 保存音频文件（标记为回退）
                                let output_dir = PathBuf::from("test_output");
                                if !output_dir.exists() {
                                    fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
                                        eprintln!("警告: 无法创建输出目录: {}", e);
                                    });
                                }
                                
                                let timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                let output_file = output_dir.join(format!("tts_output_{}_{}_fallback_{}.wav", 
                                    idx, target_lang, timestamp));
                                match fs::write(&output_file, &result.audio) {
                                    Ok(_) => {
                                        println!("    💾 音频已保存（回退）: {}", output_file.display());
                                    },
                                    Err(e) => {
                                        println!("    ⚠️  保存音频失败: {}", e);
                                    }
                                }
                                tts_success = true;
                            },
                            Err(e2) => {
                                println!("    ❌ 回退也失败: {}", e2);
                                println!("    ⚠️  提示: 需要配置英文 TTS 模型才能生成正确的英文语音");
                            }
                        }
                    }
                }
            }
            
            if !tts_success {
                println!("    ⚠️  TTS 生成失败，跳过此步骤");
            }
        }
    }
    
    println!("\n✅ 处理完成！");
    println!("识别到的文本: {:?}", all_transcripts);
    
    // 清理
    asr.finalize().await?;
    nmt.finalize().await?;
    tts.close().await?;
    
    Ok(())
}

