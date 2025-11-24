//! 完整 S2S 流集成测试（简化版）：使用真实的 ASR 和 NMT
//! 
//! 使用方法：
//!   cargo run --example test_s2s_full_simple -- <input_wav_file> [--direction <en-zh|zh-en>] [--nmt-model <marian|m2m100>]
//! 
//! 示例：
//!   cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav --direction en-zh --nmt-model m2m100
//!   cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav --direction zh-en --nmt-model marian
//! 
//! 前提条件：
//!   1. WSL2 中已启动 Piper HTTP 服务（http://127.0.0.1:5005/tts）
//!   2. Whisper ASR 模型已下载到 core/engine/models/asr/whisper-base/
//!   3. NMT 模型已下载（根据 --nmt-model 参数选择）：
//!      - Marian: core/engine/models/nmt/marian-en-zh/ 或 marian-zh-en/
//!      - M2M100: core/engine/models/nmt/m2m100-en-zh/ 或 m2m100-zh-en/
//!   4. 输入音频文件（WAV 格式）

use std::fs;
use std::path::PathBuf;
use std::env;
use hound::WavReader;
use core_engine::types::AudioFrame;
use core_engine::asr_streaming::{AsrRequest, AsrStreaming};
use core_engine::nmt_incremental::{TranslationRequest, NmtIncremental, MarianNmtOnnx, M2M100NmtOnnx};
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

/// 翻译方向配置
#[derive(Debug, Clone)]
enum TranslationDirection {
    EnToZh,  // 英文 → 中文
    ZhToEn,  // 中文 → 英文
}

impl TranslationDirection {
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "en-zh" | "en_zh" => Ok(TranslationDirection::EnToZh),
            "zh-en" | "zh_en" => Ok(TranslationDirection::ZhToEn),
            _ => Err(format!("无效的方向: {}，支持的方向: en-zh, zh-en", s)),
        }
    }

    fn nmt_model_name_marian(&self) -> &'static str {
        match self {
            TranslationDirection::EnToZh => "marian-en-zh",
            TranslationDirection::ZhToEn => "marian-zh-en",
        }
    }

    fn nmt_model_name_m2m100(&self) -> &'static str {
        match self {
            TranslationDirection::EnToZh => "m2m100-en-zh",
            TranslationDirection::ZhToEn => "m2m100-zh-en",
        }
    }

    fn source_language_hint(&self) -> &'static str {
        match self {
            TranslationDirection::EnToZh => "en",
            TranslationDirection::ZhToEn => "zh",
        }
    }

    fn target_language(&self) -> &'static str {
        match self {
            TranslationDirection::EnToZh => "zh",
            TranslationDirection::ZhToEn => "en",
        }
    }

    fn tts_voice(&self) -> &'static str {
        match self {
            TranslationDirection::EnToZh => "zh_CN-huayan-medium",  // 中文 TTS
            TranslationDirection::ZhToEn => "zh_CN-huayan-medium",  // 暂时使用中文，未来可添加英文 TTS
        }
    }

    fn tts_locale(&self) -> &'static str {
        match self {
            TranslationDirection::EnToZh => "zh-CN",
            TranslationDirection::ZhToEn => "zh-CN",  // 暂时使用中文，未来可添加英文 TTS
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 完整 S2S 流集成测试（简化版 - 真实 ASR + NMT + Piper TTS） ===\n");

    // 解析命令行参数
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: cargo run --example test_s2s_full_simple -- <input_wav_file> [--direction <en-zh|zh-en>] [--nmt-model <marian|m2m100>]");
        eprintln!("示例: cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav --direction en-zh --nmt-model m2m100");
        eprintln!("      cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav --direction zh-en --nmt-model marian");
        return Err("缺少输入音频文件路径".into());
    }
    
    let input_wav = PathBuf::from(&args[1]);
    if !input_wav.exists() {
        return Err(format!("输入文件不存在: {}", input_wav.display()).into());
    }

    // 解析翻译方向参数（默认为 en-zh）
    let mut direction = TranslationDirection::EnToZh;
    let mut nmt_model_type = "m2m100"; // 默认使用 M2M100
    
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--direction" && i + 1 < args.len() {
            direction = TranslationDirection::from_str(&args[i + 1])?;
            i += 2;
        } else if args[i] == "--nmt-model" && i + 1 < args.len() {
            nmt_model_type = &args[i + 1];
            if nmt_model_type != "marian" && nmt_model_type != "m2m100" {
                return Err(format!("无效的 NMT 模型类型: {}，支持: marian, m2m100", nmt_model_type).into());
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    if i == 2 {
        println!("[INFO] 未指定 --direction，默认使用 en-zh（英文→中文）");
        println!("[INFO] 未指定 --nmt-model，默认使用 m2m100");
    }

    println!("[配置] 翻译方向: {} → {}", 
        direction.source_language_hint(), 
        direction.target_language());
    println!("[配置] NMT 模型类型: {}", nmt_model_type);
    println!("[配置] TTS 声库: {}", direction.tts_voice());

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

    // 加载 NMT（根据翻译方向和模型类型选择）
    println!("\n[4/7] 加载 NMT 模型（{}）...", nmt_model_type);
    let nmt_model_name = if nmt_model_type == "m2m100" {
        direction.nmt_model_name_m2m100()
    } else {
        direction.nmt_model_name_marian()
    };
    let nmt_model_dir = crate_root.join("models/nmt").join(nmt_model_name);
    if !nmt_model_dir.exists() {
        return Err(format!("{} NMT 模型目录不存在: {}，请确保模型已下载", nmt_model_type, nmt_model_dir.display()).into());
    }
    
    println!("  模型路径: {}", nmt_model_dir.display());
    
    // 根据模型类型加载（使用枚举来统一处理）
    enum NmtModel {
        Marian(MarianNmtOnnx),
        M2M100(M2M100NmtOnnx),
    }
    
    let nmt = if nmt_model_type == "m2m100" {
        let m2m100 = M2M100NmtOnnx::new_from_dir(&nmt_model_dir)
            .map_err(|e| format!("Failed to load M2M100 NMT: {}", e))?;
        println!("[OK] M2M100 NMT 加载成功");
        NmtModel::M2M100(m2m100)
    } else {
        let marian = MarianNmtOnnx::new_from_dir(&nmt_model_dir)
            .map_err(|e| format!("Failed to load Marian NMT: {}", e))?;
        println!("[OK] Marian NMT 加载成功");
        NmtModel::Marian(marian)
    };
    
    // 初始化 NMT
    match &nmt {
        NmtModel::Marian(m) => NmtIncremental::initialize(m).await
            .map_err(|e| format!("Failed to initialize Marian NMT: {}", e))?,
        NmtModel::M2M100(m) => NmtIncremental::initialize(m).await
            .map_err(|e| format!("Failed to initialize M2M100 NMT: {}", e))?,
    }

    // 步骤 1: ASR 识别（自动检测语言）
    println!("\n[4/7] 执行 ASR 识别（自动检测语言）...");
    let mut all_transcript_texts = Vec::new();
    let mut detected_language: Option<String> = None;
    
    for (i, frame) in audio_frames.iter().enumerate() {
        // 使用 None 进行自动语言检测
        // 根据产品需求，应该支持自动检测，所以使用 None
        let asr_request = AsrRequest {
            frame: frame.clone(),
            language_hint: None,  // None 表示自动检测语言
        };
        
        let asr_result = AsrStreaming::infer(&asr, asr_request).await
            .map_err(|e| format!("ASR inference failed: {}", e))?;
        
        if let Some(ref partial) = asr_result.partial {
            println!("  部分结果 [帧 {}]: {}", i + 1, partial.text);
        }
        
        if let Some(ref final_transcript) = asr_result.final_transcript {
            println!("  最终结果 [帧 {}]: {}", i + 1, final_transcript.text);
            all_transcript_texts.push(final_transcript.text.clone());
            // 获取检测到的语言（从第一个有效结果中获取）
            if detected_language.is_none() {
                detected_language = Some(final_transcript.language.clone());
            }
        }
    }
    
    let source_text = all_transcript_texts.join(" ");
    if source_text.is_empty() {
        return Err("ASR 未返回任何转录结果".into());
    }
    
    // 根据检测到的语言动态选择翻译方向
    let mut detected_lang = detected_language.as_deref().unwrap_or("unknown");
    
    // 如果检测到的语言是 "unknown"，尝试从文本内容推断语言
    if detected_lang == "unknown" {
        // 简单的语言推断：检查文本中是否包含中文字符
        let has_chinese = source_text.chars().any(|c| {
            let code = c as u32;
            // 中文字符范围：CJK 统一汉字 (0x4E00-0x9FFF)
            (0x4E00..=0x9FFF).contains(&code)
        });
        
        // 检查文本中是否主要是英文字符
        let english_ratio = source_text.chars()
            .filter(|c| c.is_ascii_alphabetic() || c.is_whitespace() || c.is_ascii_punctuation())
            .count() as f32 / source_text.chars().count().max(1) as f32;
        
        if has_chinese {
            detected_lang = "zh";
            println!("[INFO] 从文本内容推断语言: 中文（检测到中文字符）");
        } else if english_ratio > 0.7 {
            detected_lang = "en";
            println!("[INFO] 从文本内容推断语言: 英文（文本主要为英文字符）");
        }
    }
    
    println!("[OK] ASR 识别完成");
    println!("  检测到的语言: {}", detected_lang);
    println!("  源文本: {}", source_text);
    
    // 根据检测到的语言选择翻译方向
    let (src_lang, tgt_lang) = match detected_lang {
        "zh" | "chinese" => ("zh", "en"),  // 中文 → 英文
        "en" | "english" => ("en", "zh"),  // 英文 → 中文
        _ => {
            // 如果无法识别，使用默认方向
            println!("[WARNING] 无法识别语言 '{}'，使用默认翻译方向: {} → {}", 
                detected_lang, direction.source_language_hint(), direction.target_language());
            (direction.source_language_hint(), direction.target_language())
        }
    };
    
    println!("\n[5/7] 加载 NMT 模型（{}）...", nmt_model_type);
    println!("  翻译方向: {} → {} (根据 ASR 检测到的语言自动选择)", src_lang, tgt_lang);
    
    // 创建 PartialTranscript 用于翻译请求
    let transcript = core_engine::types::PartialTranscript {
        text: source_text.clone(),
        confidence: 1.0,
        is_final: true,
    };
    
    let translation_request = TranslationRequest {
        transcript,
        target_language: tgt_lang.to_string(),
        wait_k: None,
    };
    
    // 根据翻译方向动态加载 NMT 模型
    let nmt_model_name = if nmt_model_type == "m2m100" {
        if src_lang == "zh" && tgt_lang == "en" {
            "m2m100-zh-en"
        } else if src_lang == "en" && tgt_lang == "zh" {
            "m2m100-en-zh"
        } else {
            return Err(format!("不支持的翻译方向: {} → {}", src_lang, tgt_lang).into());
        }
    } else {
        if src_lang == "zh" && tgt_lang == "en" {
            "marian-zh-en"
        } else if src_lang == "en" && tgt_lang == "zh" {
            "marian-en-zh"
        } else {
            return Err(format!("不支持的翻译方向: {} → {}", src_lang, tgt_lang).into());
        }
    };
    
    println!("  使用 NMT 模型: {}", nmt_model_name);
    
    // 如果模型不匹配，需要重新加载
    let nmt_model_dir = crate_root.join("models/nmt").join(nmt_model_name);
    if !nmt_model_dir.exists() {
        return Err(format!("NMT 模型目录不存在: {}", nmt_model_dir.display()).into());
    }
    
    // 重新加载 NMT 模型（如果需要）
    let nmt = if nmt_model_type == "m2m100" {
        let m2m100 = M2M100NmtOnnx::new_from_dir(&nmt_model_dir)
            .map_err(|e| format!("Failed to load M2M100 NMT: {}", e))?;
        println!("[OK] M2M100 NMT 加载成功: {}", nmt_model_name);
        NmtModel::M2M100(m2m100)
    } else {
        let marian = MarianNmtOnnx::new_from_dir(&nmt_model_dir)
            .map_err(|e| format!("Failed to load Marian NMT: {}", e))?;
        println!("[OK] Marian NMT 加载成功: {}", nmt_model_name);
        NmtModel::Marian(marian)
    };
    
    // 初始化 NMT
    match &nmt {
        NmtModel::Marian(m) => NmtIncremental::initialize(m).await
            .map_err(|e| format!("Failed to initialize Marian NMT: {}", e))?,
        NmtModel::M2M100(m) => NmtIncremental::initialize(m).await
            .map_err(|e| format!("Failed to initialize M2M100 NMT: {}", e))?,
    }
    
    let translation_response = match &nmt {
        NmtModel::Marian(m) => NmtIncremental::translate(m, translation_request).await
            .map_err(|e| format!("Translation failed: {}", e))?,
        NmtModel::M2M100(m) => NmtIncremental::translate(m, translation_request).await
            .map_err(|e| format!("Translation failed: {}", e))?,
    };
    
    let target_text = translation_response.translated_text;
    println!("[OK] NMT 翻译完成");
    println!("  目标文本（{}）: {}", tgt_lang, target_text);
    println!("  翻译稳定: {}", translation_response.is_stable);
    
    // 日志：NMT 输出的纯文本（送进 TTS 之前）
    println!("\n[日志] NMT 输出文本（送进 TTS 之前）: \"{}\"", target_text);

    // 步骤 3: TTS 合成（使用 Piper HTTP TTS 合成目标语言语音）
    println!("\n[7/7] 执行 TTS 合成（Piper HTTP）...");
    println!("  说明: 合成{}语音用于回放翻译结果", direction.target_language());
    
    // 创建 TTS 客户端
    let config = core_engine::tts_streaming::PiperHttpConfig::default();
    let tts_client = core_engine::tts_streaming::PiperHttpTts::new(config)
        .map_err(|e| format!("Failed to create PiperHttpTts: {}", e))?;
    
    // 修复：使用 target_text 而不是 source_text
    let tts_request = TtsRequest {
        text: target_text.clone(), // 使用翻译后的目标文本进行 TTS
        voice: direction.tts_voice().to_string(),
        locale: direction.tts_locale().to_string(),
    };
    
    // 日志：送进 Piper 的文本（如果中间有任何正则/分词/拼音转换，要看转换后的结果）
    println!("\n[日志] 送进 Piper TTS 的文本: \"{}\"", tts_request.text);
    
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
    match &nmt {
        NmtModel::Marian(m) => NmtIncremental::finalize(m).await?,
        NmtModel::M2M100(m) => NmtIncremental::finalize(m).await?,
    }

    println!("\n=== 测试完成 ===");
    println!("\n完整 S2S 流程测试总结：");
    println!("  ✅ 步骤 1: ASR 识别（真实 Whisper）");
    println!("    输入: 音频文件 {}", input_wav.display());
    println!("    输出（{}）: \"{}\"", direction.source_language_hint(), source_text);
    println!();
    println!("  ✅ 步骤 2: NMT 翻译（真实 {}，{}）", nmt_model_type, nmt_model_name);
    println!("    输入（{}）: \"{}\"", direction.source_language_hint(), source_text);
    println!("    输出（{}）: \"{}\"", direction.target_language(), target_text);
    println!();
    println!("  ✅ 步骤 3: TTS 合成（Piper HTTP）");
    println!("    输入（{}）: \"{}\"", direction.target_language(), target_text);
    println!("    输出: {} 字节 WAV 音频", chunk.audio.len());
    println!();
    println!("  完整流程: {}语音 → {}文本 → {}文本 → {}语音", 
        direction.source_language_hint(),
        direction.source_language_hint(),
        direction.target_language(),
        direction.target_language());
    println!();
    println!("下一步：");
    println!("  1. 播放音频文件验证语音质量: {}", output_file.display());
    println!("  2. 验证翻译准确性");
    println!("  3. 如果正常，完整的 S2S 流程已验证通过");

    Ok(())
}

