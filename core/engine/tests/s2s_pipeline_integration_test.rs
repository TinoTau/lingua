use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use core_engine::asr_whisper::WhisperAsrEngine;
use core_engine::nmt_client::{LocalM2m100HttpClient, NmtClient, NmtTranslateRequest};
use core_engine::tts_streaming::{TtsRequest, TtsStreaming, PiperHttpTts, PiperHttpConfig};
use core_engine::types::AudioFrame;
use core_engine::post_processing::TextPostProcessor;

const TEST_OUTPUT_DIR: &str = r"D:\Programs\github\lingua\test_output";

/// 读取 WAV 文件并切分为 AudioFrame 列表（单声道/浮点）
fn read_wav_frames(wav_path: &Path) -> Result<Vec<AudioFrame>> {
    let mut reader = hound::WavReader::open(wav_path)
        .with_context(|| format!("无法打开音频文件 {}", wav_path.display()))?;
    let spec = reader.spec();

    let mut samples = Vec::new();
    match spec.sample_format {
        hound::SampleFormat::Float => {
            for sample in reader.samples::<f32>() {
                samples.push(sample?);
            }
        }
        hound::SampleFormat::Int => {
            let max_val = (1i32 << (spec.bits_per_sample - 1)) as f32;
            for sample in reader.samples::<i32>() {
                samples.push(sample? as f32 / max_val);
            }
        }
    }

    // Whisper 期望 16kHz 单声道，这里按 10ms 一帧拆分
    let frame_size = (spec.sample_rate / 100) as usize;
    let mut frames = Vec::new();
    for (idx, chunk) in samples.chunks(frame_size).enumerate() {
        frames.push(AudioFrame {
            sample_rate: spec.sample_rate,
            channels: spec.channels as u8,
            data: chunk.to_vec(),
            timestamp_ms: (idx * 10) as u64,
        });
    }

    Ok(frames)
}

fn contains_cjk(text: &str) -> bool {
    text.chars().any(|c| matches!(c as u32, 0x4E00..=0x9FFF))
}

/// 规范化中文标点符号，确保使用正确的中文标点以改善 TTS 停顿
fn normalize_chinese_punctuation(text: &str) -> String {
    let mut result = text.to_string();
    
    // 将英文标点转换为中文标点
    result = result.replace(',', "，");  // 逗号
    result = result.replace('.', "。");  // 句号
    result = result.replace('!', "！");  // 感叹号
    result = result.replace('?', "？");  // 问号
    result = result.replace(';', "；");  // 分号
    result = result.replace(':', "：");  // 冒号
    
    // 确保标点符号前后有适当的空格（中文通常不需要，但某些 TTS 可能需要）
    // 这里先不做处理，因为中文通常不需要在标点前后加空格
    
    result
}

/// 文本分段信息（包含分段文本和停顿类型）
#[derive(Debug, Clone)]
struct TextSegment {
    text: String,
    pause_type: PauseType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PauseType {
    /// 句子结束标点后的停顿（较长，如句号、问号、感叹号）
    SentenceEnd,
    /// 逗号后的停顿（较短）
    Comma,
    /// 无停顿（最后一段）
    None,
}

/// 专门用于 TTS 的文本分段函数
/// 在句子结束标点和逗号处都分割，但停顿时间不同
fn segment_for_tts(text: &str) -> Vec<TextSegment> {
    let text = text.trim();
    if text.is_empty() {
        return vec![];
    }

    let mut segments = Vec::new();
    let mut current_segment = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        current_segment.push(ch);

        // 检查是否为句子结束标点（句号、问号、感叹号）
        let is_sentence_end = matches!(
            ch,
            '.' | '!' | '?' | '。' | '！' | '？'
        );

        // 检查是否为逗号
        let is_comma = matches!(ch, ',' | '，');

        if is_sentence_end {
            // 检查是否为缩写（简单规则：如果后面是小写字母，可能是缩写）
            let is_abbreviation = if let Some(&next_ch) = chars.peek() {
                next_ch.is_alphabetic() && next_ch.is_lowercase()
            } else {
                false
            };

            if !is_abbreviation {
                // 句子结束
                let segment_text = current_segment.trim().to_string();
                if !segment_text.is_empty() {
                    segments.push(TextSegment {
                        text: segment_text,
                        pause_type: PauseType::SentenceEnd,
                    });
                }
                current_segment.clear();
                continue;
            }
        } else if is_comma {
            // 逗号处也分割，但停顿时间较短
            let segment_text = current_segment.trim().to_string();
            if !segment_text.is_empty() {
                segments.push(TextSegment {
                    text: segment_text,
                    pause_type: PauseType::Comma,
                });
            }
            current_segment.clear();
            continue;
        }
    }

    // 添加最后一个句子（如果没有以标点结尾）
    let last_segment = current_segment.trim().to_string();
    if !last_segment.is_empty() {
        segments.push(TextSegment {
            text: last_segment,
            pause_type: PauseType::None,
        });
    }

    // 如果没有任何分割，返回整个文本
    if segments.is_empty() {
        segments.push(TextSegment {
            text: text.to_string(),
            pause_type: PauseType::None,
        });
    }

    segments
}

/// 从 WAV 文件中提取音频数据（跳过 WAV 头部）
fn extract_wav_audio_data(wav_data: &[u8]) -> Result<Vec<u8>> {
    // WAV 文件格式：
    // - RIFF header (12 bytes)
    // - fmt chunk (至少 24 bytes)
    // - data chunk header (8 bytes)
    // - audio data
    
    if wav_data.len() < 44 {
        return Err(anyhow::anyhow!("WAV 文件太小，无法解析"));
    }
    
    // 验证 RIFF 头部
    if &wav_data[0..4] != b"RIFF" {
        return Err(anyhow::anyhow!("不是有效的 WAV 文件（缺少 RIFF 头部）"));
    }
    
    if &wav_data[8..12] != b"WAVE" {
        return Err(anyhow::anyhow!("不是有效的 WAV 文件（缺少 WAVE 标识）"));
    }
    
    // 查找 data chunk
    let mut offset = 12;
    while offset + 8 < wav_data.len() {
        let chunk_id = &wav_data[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            wav_data[offset + 4],
            wav_data[offset + 5],
            wav_data[offset + 6],
            wav_data[offset + 7],
        ]) as usize;
        
        if chunk_id == b"data" {
            // 找到 data chunk，提取音频数据
            let data_start = offset + 8;
            let data_end = data_start + chunk_size.min(wav_data.len() - data_start);
            return Ok(wav_data[data_start..data_end].to_vec());
        }
        
        // 移动到下一个 chunk（chunk_size 需要对齐到 2 字节边界）
        offset += 8 + ((chunk_size + 1) & !1);
    }
    
    Err(anyhow::anyhow!("未找到 data chunk"))
}

#[tokio::test]
async fn test_s2s_pipeline_end_to_end() -> Result<()> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .context("无法定位项目根目录")?;

    let whisper_model = crate_root.join("models/asr/whisper-base/ggml-base.bin");
    let wav_path = project_root.join("test_output/s2s_pipeline_output.wav");
    
    // Python NMT 服务 URL（默认端口 5008）
    let nmt_service_url = std::env::var("NMT_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:5008".to_string());
    
    // Piper TTS 服务 URL（默认端口 5005）
    let tts_service_url = std::env::var("TTS_SERVICE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:5005/tts".to_string());
    let tts_default_voice = std::env::var("TTS_DEFAULT_VOICE")
        .unwrap_or_else(|_| "zh_CN-huayan-medium".to_string());

    for (path, desc) in [
        (&whisper_model, "Whisper 模型"),
        (&wav_path, "输入音频"),
    ] {
        if !path.exists() {
            eprintln!("⚠ 跳过测试：{} 不存在 -> {}", desc, path.display());
            return Ok(());
        }
    }

    // 1. ASR
    let mut asr_engine = WhisperAsrEngine::new_from_model_path(&whisper_model)
        .context("加载 Whisper ASR 失败")?;
    asr_engine.set_language(Some("en".to_string()));

    let frames = read_wav_frames(&wav_path)?;
    let (transcript, detected_lang) = asr_engine
        .transcribe_frames(&frames)
        .context("ASR 转录失败")?;

    println!("ASR 检测语言: {:?}", detected_lang);
    println!("ASR 输出: {}", transcript);

    // 2. NMT（通过 Python NMT 服务）
    let nmt_client = LocalM2m100HttpClient::new(&nmt_service_url);
    let translation_response = nmt_client
        .translate(&NmtTranslateRequest {
            src_lang: "en".to_string(),
            tgt_lang: "zh".to_string(),
            text: transcript.clone(),
        })
        .await
        .context("NMT 翻译失败")?;
    
    if !translation_response.ok {
        return Err(anyhow::anyhow!(
            "NMT 服务返回错误: {}",
            translation_response.error.unwrap_or_else(|| "Unknown error".to_string())
        ));
    }
    
    let translation = translation_response.text
        .ok_or_else(|| anyhow::anyhow!("NMT 服务未返回翻译文本"))?;
    println!("NMT 输出（原始）: {}", translation);
    assert!(
        contains_cjk(&translation),
        "NMT 输出未包含中文内容"
    );

    // 2.5. 文本后处理（规范化标点符号，确保停顿正确）
    let post_processor = TextPostProcessor::default();
    let processed_text = post_processor.process(&translation, "zh");
    
    // 规范化中文标点符号（将英文标点转换为中文标点）
    let processed_text = normalize_chinese_punctuation(&processed_text);
    println!("NMT 输出（处理后）: {}", processed_text);

    // 3. TTS（通过 Piper HTTP 服务，分段处理以添加停顿）
    let tts_config = PiperHttpConfig {
        endpoint: tts_service_url.clone(),
        default_voice: tts_default_voice.clone(),
        timeout_ms: 8000,
    };
    let tts_engine = PiperHttpTts::new(tts_config)
        .map_err(|e| anyhow::anyhow!("创建 Piper HTTP TTS 客户端失败: {}", e))?;
    
    // 3.1. 分段文本（按句子分割，只在句子结束标点处分割）
    println!("\n=== 文本分段调试 ===");
    println!("原始文本: '{}'", processed_text);
    println!("文本长度: {} 字符", processed_text.chars().count());
    
    // 打印每个字符及其 Unicode 码点（用于调试）
    println!("字符详情（标点符号）:");
    for (idx, ch) in processed_text.char_indices() {
        let code = ch as u32;
        let is_sentence_end = matches!(ch, '.' | '!' | '?' | '。' | '！' | '？');
        let is_comma = matches!(ch, ',' | '，');
        if is_sentence_end {
            println!("  [{}] '{}' (U+{:04X}) <- 句子结束标点", idx, ch, code);
        } else if is_comma {
            println!("  [{}] '{}' (U+{:04X}) <- 逗号（会在此处分割，添加短停顿）", idx, ch, code);
        }
    }
    
    // 使用专门用于 TTS 的分段函数（在句子结束标点和逗号处都分割）
    let segments = segment_for_tts(&processed_text);
    println!("\n文本分割为 {} 个段落:", segments.len());
    for (idx, segment) in segments.iter().enumerate() {
        let pause_desc = match segment.pause_type {
            PauseType::SentenceEnd => "句子结束停顿",
            PauseType::Comma => "逗号停顿",
            PauseType::None => "无停顿",
        };
        println!("  段落 {}: '{}' (长度: {}, {})", 
            idx + 1, segment.text, segment.text.chars().count(), pause_desc);
    }
    println!("=== 分段结束 ===\n");
    
    if segments.is_empty() {
        return Err(anyhow::anyhow!("文本分割后为空"));
    }
    
    // 3.2. 为每个段落生成音频
    let mut all_audio_chunks: Vec<Vec<u8>> = Vec::new();
    for (idx, segment) in segments.iter().enumerate() {
        println!("[TTS {}/{}] 合成: '{}'", idx + 1, segments.len(), segment.text);
        
        let request = TtsRequest {
            text: segment.text.clone(),
            voice: tts_default_voice.clone(),
            locale: "zh-CN".into(),
        };
        
        let chunk = tts_engine
            .synthesize(request)
            .await
            .map_err(|e| anyhow::anyhow!("TTS 合成失败（段落 {}）: {}", idx + 1, e))?;
        
        // 提取 WAV 音频数据（跳过 WAV 头部，只保留音频数据）
        let audio_data = extract_wav_audio_data(&chunk.audio)?;
        all_audio_chunks.push(audio_data);
        
        // 根据停顿类型添加不同时长的停顿
        if segment.pause_type != PauseType::None {
            let pause_duration_ms = match segment.pause_type {
                PauseType::SentenceEnd => 250,  // 句子结束：250ms
                PauseType::Comma => 150,        // 逗号：150ms
                PauseType::None => 0,
            };
            
            let sample_rate = 22050;
            let pause_samples = (pause_duration_ms as f32 * sample_rate as f32 / 1000.0) as usize;
            let pause_audio = vec![0i16; pause_samples];
            let pause_bytes: Vec<u8> = pause_audio.iter()
                .flat_map(|&sample| sample.to_le_bytes())
                .collect();
            all_audio_chunks.push(pause_bytes);
            
            let pause_desc = match segment.pause_type {
                PauseType::SentenceEnd => "句子结束",
                PauseType::Comma => "逗号",
                PauseType::None => "",
            };
            println!("[TTS] 添加 {}ms {}停顿", pause_duration_ms, pause_desc);
        }
    }
    
    // 3.3. 合并所有音频块
    let total_audio_size: usize = all_audio_chunks.iter().map(|chunk| chunk.len()).sum();
    let mut merged_audio = Vec::with_capacity(total_audio_size);
    for chunk in all_audio_chunks {
        merged_audio.extend_from_slice(&chunk);
    }
    
    println!("合并后的音频大小: {} 字节 ({} 个样本)", 
        merged_audio.len(), 
        merged_audio.len() / 2);
    
    // 3.4. 保存为 WAV 文件
    std::fs::create_dir_all(TEST_OUTPUT_DIR)
        .context("创建测试输出目录失败")?;
    let output_path = Path::new(TEST_OUTPUT_DIR).join("s2s_pipeline_output_zh.wav");
    
    // 将合并的音频数据保存为 WAV（22050 Hz，16-bit，单声道）
    core_engine::tts_streaming::save_pcm_to_wav(&merged_audio, &output_path, 22050, 1)
        .context("保存中文 TTS 音频失败")?;
    println!("✅ S2S 中文音频已保存到 {}", output_path.display());

    Ok(())
}

