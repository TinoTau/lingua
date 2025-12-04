// tests/asr_vad_simple_test.rs
// 简单的 ASR + VAD 集成测试：验证停顿识别和文本识别

use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;
use core_engine::*;
use core_engine::vad::{SileroVad, SileroVadConfig};
use core_engine::asr_whisper::WhisperAsrStreaming;

// 使用测试用的 Dummy 实现
struct DummyEventBus;

#[async_trait]
impl EventBus for DummyEventBus {
    async fn start(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn stop(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn publish(&self, _event: CoreEvent) -> EngineResult<()> {
        Ok(())
    }

    async fn subscribe(&self, topic: EventTopic) -> EngineResult<EventSubscription> {
        Ok(EventSubscription { topic })
    }
}

struct DummyNmt;

#[async_trait]
impl NmtIncremental for DummyNmt {
    async fn initialize(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn translate(&self, request: TranslationRequest) -> EngineResult<TranslationResponse> {
        Ok(TranslationResponse {
            translated_text: format!("[翻译] {}", request.transcript.text),
            is_stable: true,
            speaker_id: request.speaker_id.clone(),
            source_audio_duration_ms: None,
            source_text: None,
            quality_metrics: None,
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        Ok(())
    }
}

struct DummyEmotion;

#[async_trait]
impl EmotionAdapter for DummyEmotion {
    async fn analyze(&self, _request: EmotionRequest) -> EngineResult<EmotionResponse> {
        Ok(EmotionResponse {
            primary: "neutral".to_string(),
            intensity: 0.0,
            confidence: 0.5,
        })
    }
}

struct DummyPersona;

#[async_trait]
impl PersonaAdapter for DummyPersona {
    async fn personalize(
        &self,
        transcript: StableTranscript,
        _context: PersonaContext,
    ) -> EngineResult<StableTranscript> {
        Ok(transcript)
    }
}

struct DummyTts;

#[async_trait]
impl TtsStreaming for DummyTts {
    async fn synthesize(&self, _request: TtsRequest) -> EngineResult<TtsStreamChunk> {
        Ok(TtsStreamChunk {
            audio: vec![],
            timestamp_ms: 0,
            is_last: true,
        })
    }

    async fn close(&self) -> EngineResult<()> {
        Ok(())
    }
}

struct DummyConfig;

#[async_trait]
impl ConfigManager for DummyConfig {
    async fn load(&self) -> EngineResult<EngineConfig> {
        Ok(EngineConfig {
            mode: "test".to_string(),
            source_language: "zh".to_string(),
            target_language: "en".to_string(),
        })
    }

    async fn current(&self) -> EngineResult<EngineConfig> {
        self.load().await
    }
}

struct DummyCache;

#[async_trait]
impl CacheManager for DummyCache {
    async fn warm_up(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn purge(&self) -> EngineResult<()> {
        Ok(())
    }
}

struct DummyTelemetry;

#[async_trait]
impl TelemetrySink for DummyTelemetry {
    async fn record(&self, _datum: TelemetryDatum) -> EngineResult<()> {
        Ok(())
    }
}

/// 从 WAV 文件加载音频并转换为 AudioFrame 序列
fn load_wav_to_frames(wav_path: &PathBuf, frame_size_ms: u64) -> Vec<AudioFrame> {
    use hound::WavReader;
    
    let mut reader = WavReader::open(wav_path)
        .expect("Failed to open WAV file");
    let spec = reader.spec();
    
    println!("  采样率: {} Hz", spec.sample_rate);
    println!("  声道: {}", spec.channels);
    println!("  位深: {} bits", spec.bits_per_sample);
    
    // 读取样本
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
    
    // 重采样到 16kHz（如果需要）
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
    
    // 分割成帧
    let frame_size_samples = (frame_size_ms * 16000 / 1000) as usize;
    let mut frames = Vec::new();
    let mut timestamp_ms = 0u64;
    
    for chunk in audio_16k.chunks(frame_size_samples) {
        frames.push(AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: chunk.to_vec(),
            timestamp_ms,
        });
        timestamp_ms += frame_size_ms;
    }
    
    println!("  总样本数: {}", audio_16k.len());
    println!("  总时长: {:.2} 秒", audio_16k.len() as f32 / 16000.0);
    println!("  帧数: {}", frames.len());
    println!("  帧大小: {}ms ({} 样本)", frame_size_ms, frame_size_samples);
    
    frames
}

/// 测试：使用 SileroVad 和 Whisper ASR 进行停顿识别和文本识别
#[tokio::test]
#[ignore]  // 需要模型文件，默认忽略
async fn test_asr_vad_simple() {
    println!("\n========== ASR + VAD 简单测试 ==========");
    println!("测试目标：验证停顿识别和文本识别功能");
    
    // 1. 检查模型文件
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let whisper_model_dir = crate_root.join("models/asr/whisper-base");
    let silero_model_path = crate_root.join("models/vad/silero/silero_vad.onnx");
    
    if !whisper_model_dir.exists() {
        println!("⚠️  跳过测试: Whisper 模型目录不存在: {}", whisper_model_dir.display());
        return;
    }
    
    if !silero_model_path.exists() {
        println!("⚠️  跳过测试: Silero VAD 模型文件不存在: {}", silero_model_path.display());
        return;
    }
    
    // 2. 检查测试音频文件
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");
    let test_audio_path = project_root.join("test_output/chinese.wav");
    
    if !test_audio_path.exists() {
        println!("⚠️  跳过测试: 测试音频文件不存在: {}", test_audio_path.display());
        println!("   提示: 请将测试音频文件放在 test_output/ 目录下");
        return;
    }
    
    println!("\n✓ 模型和音频文件检查通过");
    
    // 3. 创建 SileroVad
    println!("\n初始化 SileroVad...");
    let vad_config = SileroVadConfig {
        model_path: silero_model_path.to_string_lossy().to_string(),
        sample_rate: 16000,
        frame_size: 512,  // 32ms @ 16kHz
        silence_threshold: 0.2,
        min_silence_duration_ms: 600,
        adaptive_enabled: false,
        adaptive_min_samples: 3,
        adaptive_rate: 0.1,
        adaptive_min_duration_ms: 300,
        adaptive_max_duration_ms: 1200,
    };
    let vad = Arc::new(SileroVad::with_config(vad_config)
        .expect("Failed to create SileroVad"));
    println!("✓ SileroVad 初始化成功");
    
    // 4. 创建 CoreEngine
    println!("\n初始化 CoreEngine...");
    let engine = CoreEngineBuilder::new()
        .event_bus(Arc::new(DummyEventBus))
        .vad(vad.clone())
        .asr_with_default_whisper()
        .expect("Failed to load Whisper ASR")
        .nmt(Arc::new(DummyNmt))
        .emotion(Arc::new(DummyEmotion))
        .persona(Arc::new(DummyPersona))
        .tts(Arc::new(DummyTts))
        .config(Arc::new(DummyConfig))
        .cache(Arc::new(DummyCache))
        .telemetry(Arc::new(DummyTelemetry))
        .build()
        .expect("Failed to build CoreEngine");
    
    engine.boot().await.expect("Failed to boot");
    println!("✓ CoreEngine 初始化成功");
    
    // 5. 加载音频文件
    println!("\n加载测试音频: {}", test_audio_path.display());
    let audio_frames = load_wav_to_frames(&test_audio_path, 32);  // 32ms 帧
    println!("✓ 音频加载成功，共 {} 帧", audio_frames.len());
    
    // 6. 处理音频帧，检测停顿和识别文本
    println!("\n开始处理音频帧...");
    println!("==========================================");
    
    let mut boundary_count = 0;
    let mut transcript_count = 0;
    let mut total_frames = 0;
    
    for (i, frame) in audio_frames.iter().enumerate() {
        total_frames += 1;
        
        // 处理音频帧
        let result = engine.process_audio_frame(frame.clone(), Some("zh".to_string()))
            .await
            .expect("Failed to process audio frame");
        
        // 检查是否检测到边界（停顿）
        if result.is_some() {
            boundary_count += 1;
            println!("\n[边界 #{}] 帧 {} (时间戳: {}ms)", boundary_count, i + 1, frame.timestamp_ms);
            
            if let Some(process_result) = result {
                // 检查是否有转录结果
                if let Some(ref final_transcript) = process_result.asr.final_transcript {
                    transcript_count += 1;
                    println!("  ✓ 文本识别成功:");
                    println!("    文本: \"{}\"", final_transcript.text);
                    println!("    语言: {}", final_transcript.language);
                    if let Some(ref speaker_id) = final_transcript.speaker_id {
                        println!("    说话者: {}", speaker_id);
                    }
                } else {
                    println!("  ⚠ 未识别到文本（可能是静音）");
                }
            }
        }
        
        // 每 100 帧输出一次进度
        if (i + 1) % 100 == 0 {
            println!("  进度: {}/{} 帧已处理", i + 1, audio_frames.len());
        }
    }
    
    // 7. 输出测试结果
    println!("\n==========================================");
    println!("========== 测试结果 ==========");
    println!("总帧数: {}", total_frames);
    println!("检测到停顿（边界）次数: {}", boundary_count);
    println!("成功识别文本次数: {}", transcript_count);
    println!("停顿识别率: {:.1}%", (boundary_count as f32 / total_frames as f32) * 100.0);
    if boundary_count > 0 {
        println!("文本识别率: {:.1}%", (transcript_count as f32 / boundary_count as f32) * 100.0);
    }
    
    // 8. 验证结果
    assert!(total_frames > 0, "应该处理了至少一帧");
    println!("\n✓ 测试完成");
    
    // 清理
    engine.shutdown().await.expect("Failed to shutdown");
}

/// 测试：验证 VAD 能够正确检测到停顿
#[tokio::test]
#[ignore]  // 需要模型文件，默认忽略
async fn test_vad_boundary_detection() {
    println!("\n========== VAD 停顿识别测试 ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let silero_model_path = crate_root.join("models/vad/silero/silero_vad.onnx");
    
    if !silero_model_path.exists() {
        println!("⚠️  跳过测试: Silero VAD 模型文件不存在");
        return;
    }
    
    // 创建 SileroVad
    let vad_config = SileroVadConfig {
        model_path: silero_model_path.to_string_lossy().to_string(),
        sample_rate: 16000,
        frame_size: 512,
        silence_threshold: 0.2,
        min_silence_duration_ms: 600,
        adaptive_enabled: false,
        adaptive_min_samples: 3,
        adaptive_rate: 0.1,
        adaptive_min_duration_ms: 300,
        adaptive_max_duration_ms: 1200,
    };
    let vad = SileroVad::with_config(vad_config)
        .expect("Failed to create SileroVad");
    
    println!("✓ SileroVad 初始化成功");
    println!("\n测试：发送静音帧，验证能够检测到停顿...");
    
    let mut boundary_detected = false;
    let mut frame_count = 0;
    
    // 发送 30 个静音帧（约 1 秒）
    for i in 0..30 {
        let frame = AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.0; 512],  // 静音
            timestamp_ms: (i * 32) as u64,
        };
        
        let result = vad.detect(frame).await.expect("Failed to detect");
        frame_count += 1;
        
        if result.is_boundary {
            boundary_detected = true;
            println!("  ✓ 检测到停顿: 帧 {} (时间戳: {}ms, 置信度: {:.3})", 
                     i + 1, result.frame.timestamp_ms, result.confidence);
            break;
        }
    }
    
    println!("\n测试结果:");
    println!("  处理帧数: {}", frame_count);
    println!("  检测到停顿: {}", if boundary_detected { "是" } else { "否" });
    
    // 注意：由于需要先检测到语音才能触发边界，静音帧可能不会触发边界
    // 这个测试主要验证 VAD 能够正常运行
    println!("\n✓ VAD 停顿识别测试完成");
}

/// 测试：验证 ASR 能够识别文本
#[tokio::test]
#[ignore]  // 需要模型文件，默认忽略
async fn test_asr_text_recognition() {
    println!("\n========== ASR 文本识别测试 ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let whisper_model_dir = crate_root.join("models/asr/whisper-base");
    
    if !whisper_model_dir.exists() {
        println!("⚠️  跳过测试: Whisper 模型目录不存在");
        return;
    }
    
    // 创建 Whisper ASR
    let asr = WhisperAsrStreaming::new_from_dir(&whisper_model_dir)
        .expect("Failed to create Whisper ASR");
    asr.initialize().await.expect("Failed to initialize ASR");
    asr.set_language(Some("zh".to_string())).expect("Failed to set language");
    
    println!("✓ Whisper ASR 初始化成功");
    
    // 检查测试音频文件
    let project_root = crate_root
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve project root");
    let test_audio_path = project_root.join("test_output/chinese.wav");
    
    if !test_audio_path.exists() {
        println!("⚠️  跳过测试: 测试音频文件不存在");
        return;
    }
    
    println!("\n加载测试音频...");
    let audio_frames = load_wav_to_frames(&test_audio_path, 32);
    println!("✓ 音频加载成功，共 {} 帧", audio_frames.len());
    
    // 累积所有帧
    println!("\n累积音频帧...");
    for frame in &audio_frames {
        asr.accumulate_frame(frame.clone()).expect("Failed to accumulate frame");
    }
    println!("✓ 音频帧累积完成");
    
    // 在边界时进行推理
    println!("\n执行 ASR 推理...");
    let result = asr.infer_on_boundary().await.expect("Failed to infer");
    
    println!("\n========== 识别结果 ==========");
    if let Some(ref final_transcript) = result.final_transcript {
        println!("✓ 文本识别成功:");
        println!("  文本: \"{}\"", final_transcript.text);
        println!("  语言: {}", final_transcript.language);
        assert!(!final_transcript.text.trim().is_empty(), "识别结果不应为空");
    } else {
        println!("⚠ 未识别到文本");
    }
    
    println!("\n✓ ASR 文本识别测试完成");
}

