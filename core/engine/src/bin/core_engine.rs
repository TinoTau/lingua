use std::path::PathBuf;
use std::sync::Arc;
use std::io::Cursor;
use std::time::Instant;
use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State},
    http::StatusCode,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use base64::{Engine as _, engine::general_purpose};

use core_engine::bootstrap::{CoreEngine, CoreEngineBuilder, ProcessResult};
use core_engine::config_manager::{ConfigManager, EngineConfig};
use core_engine::error::EngineResult;
use core_engine::types::AudioFrame;
use core_engine::health_check::HealthChecker;
use core_engine::emotion_adapter::EmotionStub;
use core_engine::persona_adapter::PersonaStub;
use core_engine::event_bus::{EventBus, CoreEvent, EventTopic, EventSubscription, ChannelEventBus};
use core_engine::vad::{VoiceActivityDetector, DetectionOutcome, SileroVad};
use core_engine::cache_manager::CacheManager;
use core_engine::telemetry::{TelemetrySink, TelemetryDatum};
use core_engine::speaker_identifier::{SpeakerIdentifierMode, EmbeddingBasedMode, EmbeddingBasedSpeakerIdentifier};
use core_engine::tts_streaming::YourTtsHttpConfig;
use async_trait::async_trait;

/// 运行时配置（从 TOML 文件加载）
#[derive(Debug, Clone, Deserialize)]
struct RuntimeConfig {
    nmt: NmtConfig,
    tts: TtsConfig,
    #[serde(default)]
    asr: Option<AsrConfig>,
    #[serde(default)]
    speaker_embedding: Option<SpeakerEmbeddingConfig>,
    #[serde(default)]
    yourtts: Option<YourTtsConfig>,
    engine: EngineRuntimeConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct NmtConfig {
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TtsConfig {
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AsrConfig {
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SpeakerEmbeddingConfig {
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct YourTtsConfig {
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct EngineRuntimeConfig {
    port: u16,
    whisper_model_path: Option<String>,
    silero_vad_model_path: Option<String>,
}

/// S2S 请求（整句翻译）
#[derive(Debug, Deserialize)]
struct S2SRequest {
    audio: String, // base64 编码的音频数据
    src_lang: String,
    tgt_lang: String,
}

/// S2S 响应
#[derive(Debug, Serialize)]
struct S2SResponse {
    audio: String, // base64 编码的音频数据
    transcript: String,
    translation: String,
}

/// 健康检查响应
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    services: ServiceHealth,
}

#[derive(Debug, Serialize)]
struct ServiceHealth {
    nmt: bool,
    tts: bool,
    engine: bool,
}

/// 应用状态
#[derive(Clone)]
struct AppState {
    engine: Arc<CoreEngine>,
    config: RuntimeConfig,
    simple_config: Arc<SimpleConfig>,  // 用于动态更新语言配置
    event_bus: Arc<ChannelEventBus>,  // 事件总线（用于 WebSocket 订阅）
    speaker_mode: Arc<RwLock<EmbeddingBasedMode>>,  // 当前说话者识别模式
    speaker_identifier: Option<Arc<EmbeddingBasedSpeakerIdentifier>>,  // 说话者识别器引用（用于动态切换模式）
}

// 简单的默认实现
struct SimpleEventBus;

#[async_trait]
impl EventBus for SimpleEventBus {
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

const FINAL_FRAME_FLAG: u64 = 1u64 << 63;

struct SimpleVad;

#[async_trait]
impl VoiceActivityDetector for SimpleVad {
    async fn detect(&self, frame: AudioFrame) -> EngineResult<DetectionOutcome> {
        let is_final = (frame.timestamp_ms & FINAL_FRAME_FLAG) != 0;
        let cleaned_timestamp = frame.timestamp_ms & !FINAL_FRAME_FLAG;
        let mut cleaned_frame = frame.clone();
        cleaned_frame.timestamp_ms = cleaned_timestamp;
        Ok(DetectionOutcome {
            boundary_type: None,
            is_boundary: is_final,
            confidence: 1.0,
            frame: cleaned_frame,
        })
    }
}

use tokio::sync::RwLock;

struct SimpleConfig {
    source_lang: Arc<RwLock<String>>,
    target_lang: Arc<RwLock<String>>,
}

impl SimpleConfig {
    fn new(source_lang: String, target_lang: String) -> Self {
        Self {
            source_lang: Arc::new(RwLock::new(source_lang)),
            target_lang: Arc::new(RwLock::new(target_lang)),
        }
    }

    async fn set_target_language(&self, lang: String) {
        *self.target_lang.write().await = lang;
    }

    async fn set_source_language(&self, lang: String) {
        *self.source_lang.write().await = lang;
    }
}

#[async_trait]
impl ConfigManager for SimpleConfig {
    async fn load(&self) -> EngineResult<EngineConfig> {
        let source_lang = self.source_lang.read().await.clone();
        let target_lang = self.target_lang.read().await.clone();
        Ok(EngineConfig {
            mode: "balanced".to_string(),
            source_language: source_lang,
            target_language: target_lang,
        })
    }

    async fn current(&self) -> EngineResult<EngineConfig> {
        self.load().await
    }
}

struct SimpleCache;

#[async_trait]
impl CacheManager for SimpleCache {
    async fn warm_up(&self) -> EngineResult<()> {
        Ok(())
    }

    async fn purge(&self) -> EngineResult<()> {
        Ok(())
    }
}

struct SimpleTelemetry;

#[async_trait]
impl TelemetrySink for SimpleTelemetry {
    async fn record(&self, _datum: TelemetryDatum) -> EngineResult<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let config_path = args
        .iter()
        .position(|a| a == "--config")
        .and_then(|i| args.get(i + 1))
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| PathBuf::from("lingua_core_config.toml"));

    eprintln!("[INFO] Loading config from: {}", config_path.display());

    // 2. 加载配置文件
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;
    let runtime_config: RuntimeConfig = toml::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

    eprintln!("[INFO] Config loaded:");
    eprintln!("[INFO]   NMT URL: {}", runtime_config.nmt.url);
    eprintln!("[INFO]   TTS URL: {}", runtime_config.tts.url);
    eprintln!("[INFO]   Engine Port: {}", runtime_config.engine.port);

    // 2.5. 初始化 ASR 过滤器配置（必须在创建 CoreEngine 之前）
    let _ = core_engine::asr_filters::config::init_config_from_file();
    eprintln!("[INFO] ASR filter config initialized");

    // 3. 创建 SimpleConfig（用于动态更新语言）
    let simple_config = Arc::new(SimpleConfig::new("en".to_string(), "zh".to_string()));
    
    // 4. 初始化事件总线（使用 ChannelEventBus 以支持真正的发布/订阅）
    let event_bus = Arc::new(ChannelEventBus::new());
    event_bus.start().await
        .map_err(|e| anyhow::anyhow!("Failed to start event bus: {}", e))?;
    
    // 5. 初始化 CoreEngine 和 Speaker Identifier
    let (engine, speaker_identifier) = initialize_engine(&runtime_config, simple_config.clone(), event_bus.clone()).await?;
    eprintln!("[INFO] CoreEngine initialized successfully");

    // 6. 启动 HTTP 服务器
    let app_state = AppState {
        engine: Arc::new(engine),
        config: runtime_config.clone(),
        simple_config: simple_config.clone(),
        event_bus: event_bus.clone(),
        speaker_mode: Arc::new(RwLock::new(EmbeddingBasedMode::SingleUser)),  // 默认单人模式
        speaker_identifier,  // 说话者识别器引用（用于动态切换模式）
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/s2s", post(s2s_handler))
        .route("/stream", get(stream_handler))
        .route("/config/speaker-mode", get(get_speaker_mode))
        .route("/config/speaker-mode", post(set_speaker_mode))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", runtime_config.engine.port);
    eprintln!("[INFO] Starting HTTP server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 初始化 CoreEngine
/// 返回 (CoreEngine, Option<Arc<EmbeddingBasedSpeakerIdentifier>>)
async fn initialize_engine(
    config: &RuntimeConfig, 
    simple_config: Arc<SimpleConfig>,
    event_bus: Arc<ChannelEventBus>,
) -> EngineResult<(CoreEngine, Option<Arc<EmbeddingBasedSpeakerIdentifier>>)> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    // 1. 初始化 SileroVad
    // 注意：配置文件中的路径可以是绝对路径或相对路径
    // - 绝对路径：直接使用（例如：D:\Programs\github\lingua\core\engine\models\vad\silero\silero_vad_official.onnx）
    // - 相对路径：从 crate_root 解析（例如：models/vad/silero/silero_vad_official.onnx）
    let silero_vad_model_path = config.engine.silero_vad_model_path.clone()
        .map(|p| {
            let path = PathBuf::from(&p);
            // 如果是绝对路径，直接使用；否则从 crate_root 解析
            if path.is_absolute() {
                path
            } else {
                // 相对路径：从 crate_root 解析
                // 注意：Rust 的 PathBuf::join() 会自动处理路径分隔符（/ 和 \）
                crate_root.join(&p)
            }
        })
        .unwrap_or_else(|| crate_root.join("models/vad/silero/silero_vad_official.onnx"));
    
    eprintln!("[INFO] Crate root: {}", crate_root.display());
    eprintln!("[INFO] SileroVad model path from config: {:?}", config.engine.silero_vad_model_path);
    eprintln!("[INFO] Resolved SileroVad model path: {} (exists: {})", 
              silero_vad_model_path.display(), 
              silero_vad_model_path.exists());
    
    let vad: Arc<dyn VoiceActivityDetector> = if silero_vad_model_path.exists() {
        eprintln!("[INFO] Initializing SileroVad from: {}", silero_vad_model_path.display());
        Arc::new(SileroVad::new(&silero_vad_model_path)
            .map_err(|e| core_engine::error::EngineError::new(format!("Failed to initialize SileroVad: {}", e)))?) as Arc<dyn VoiceActivityDetector>
    } else {
        eprintln!("[WARN] SileroVad model not found at: {}, using SimpleVad", silero_vad_model_path.display());
        eprintln!("[WARN] Crate root: {}", crate_root.display());
        Arc::new(SimpleVad) as Arc<dyn VoiceActivityDetector>
    };

    // 2. 初始化 ASR（优先使用 faster-whisper，否则使用本地 whisper-rs）
    let mut builder = CoreEngineBuilder::new()
        .event_bus(event_bus.clone() as Arc<dyn EventBus>)
        .vad(vad);
    
    if let Some(ref asr_config) = config.asr {
        eprintln!("[INFO] Initializing Faster-Whisper ASR: {}", asr_config.url);
        builder = builder.asr_with_faster_whisper(asr_config.url.clone(), 30)
            .map_err(|e| core_engine::error::EngineError::new(format!("Failed to initialize Faster-Whisper ASR: {}", e)))?;
    } else {
        eprintln!("[WARN] ASR config not found, using default Whisper");
        builder = builder.asr_with_default_whisper()
            .map_err(|e| core_engine::error::EngineError::new(format!("Failed to initialize ASR: {}", e)))?;
    }

    // 3. 初始化 NMT
    builder = builder.nmt_with_m2m100_http_client(Some(&config.nmt.url))
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to initialize NMT: {}", e)))?;

    // 4. 初始化 TTS（优先使用 YourTTS，否则使用 Piper TTS）
    if let Some(ref yourtts_config) = config.yourtts {
        eprintln!("[INFO] Initializing YourTTS: {}", yourtts_config.url);
        builder = builder.tts_with_yourtts_http(YourTtsHttpConfig {
            endpoint: yourtts_config.url.clone(),
            timeout_ms: 30000,
        })
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to initialize YourTTS: {}", e)))?;
    } else {
        eprintln!("[WARN] YourTTS config not found, using Piper TTS");
        builder = builder.tts_with_piper_http(core_engine::tts_streaming::PiperHttpConfig {
            endpoint: config.tts.url.clone(),
            default_voice: "zh_CN-huayan-medium".to_string(),
            timeout_ms: 8000,
        })
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to initialize TTS: {}", e)))?;
    }

    // 5. 初始化说话者识别（如果配置了 Speaker Embedding 服务）
    // 创建 identifier 并保存引用，然后让 builder 使用同一个实例
    let speaker_identifier_ref: Option<Arc<EmbeddingBasedSpeakerIdentifier>> = if let Some(ref speaker_config) = config.speaker_embedding {
        eprintln!("[INFO] Initializing Speaker Identification: {}", speaker_config.url);
        // 创建 identifier 并保存引用
        let identifier = EmbeddingBasedSpeakerIdentifier::new(
            Some(speaker_config.url.clone()),
            0.4,
            core_engine::speaker_identifier::EmbeddingBasedMode::SingleUser,
        )?;
        let identifier_arc = Arc::new(identifier);
        // 将 identifier 转换为 trait 对象用于 builder
        let identifier_for_builder: Arc<dyn core_engine::speaker_identifier::SpeakerIdentifier> = identifier_arc.clone();
        // 直接设置到 builder，使用同一个实例（这样模式切换才能生效）
        builder = builder.with_speaker_identifier_custom(identifier_for_builder);
        Some(identifier_arc)
    } else {
        eprintln!("[WARN] Speaker Embedding config not found, speaker identification disabled");
        None
    };

    // 6. 构建 CoreEngine
    let engine = builder
        .emotion(Arc::new(EmotionStub))
        .persona(Arc::new(PersonaStub))
        .config(simple_config.clone() as Arc<dyn ConfigManager>)
        .cache(Arc::new(SimpleCache))
        .telemetry(Arc::new(SimpleTelemetry))
        .with_post_processing(None, true)
        .with_tts_incremental_playback(true, 0, 50)
        .with_audio_enhancement(core_engine::tts_audio_enhancement::AudioEnhancementConfig::default())
        .with_continuous_mode(true, 5000, 200)  // 启用连续模式以支持 WebSocket 流式处理 (max_buffer=5s, min_segment=200ms)
        .build()
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to build engine: {}", e)))?;

    // 启动引擎
    engine.boot().await
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to boot engine: {}", e)))?;

    // 从 engine 中获取 speaker_identifier（如果是 EmbeddingBasedSpeakerIdentifier）
    // 由于无法从 trait 对象直接获取具体类型，我们需要在创建时就保存引用
    // 这里我们使用之前创建的 identifier_arc（如果存在）
    Ok((engine, speaker_identifier_ref))
}

/// 健康检查端点
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let checker = HealthChecker::new();
    let nmt_health = checker.check_nmt_service(&state.config.nmt.url).await;
    let tts_health = checker.check_tts_service(&state.config.tts.url).await;

    Json(HealthResponse {
        status: "ok".to_string(),
        services: ServiceHealth {
            nmt: nmt_health.is_healthy,
            tts: tts_health.is_healthy,
            engine: true,
        },
    })
}

/// S2S 整句翻译端点
async fn s2s_handler(
    State(state): State<AppState>,
    Json(request): Json<S2SRequest>,
) -> Result<Json<S2SResponse>, StatusCode> {
    let s2s_start = Instant::now();
    eprintln!("[S2S] ===== Request started =====");
    
    // 1. 解码 base64 音频
    let audio_data = general_purpose::STANDARD
        .decode(&request.audio)
        .map_err(|e| {
            eprintln!("[ERROR] Failed to decode base64 audio: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // 2. 解析 WAV 音频并转换为 AudioFrame 列表
    let audio_frames = parse_wav_to_frames(&audio_data)
        .map_err(|e| {
            eprintln!("[ERROR] Failed to parse WAV audio: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    if audio_frames.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let frame_info = audio_frames.first().map(|frame| {
        format!(
            "{}Hz {}ch {} samples",
            frame.sample_rate,
            frame.channels,
            frame.data.len()
        )
    }).unwrap_or_else(|| "unknown format".into());
    eprintln!(
        "[S2S] Received audio: {} bytes -> {} frames (first frame: {}) [src={}, tgt={}]",
        audio_data.len(),
        audio_frames.len(),
        frame_info,
        request.src_lang,
        request.tgt_lang
    );

    // 3. 根据请求更新目标语言配置
    state.simple_config.set_target_language(request.tgt_lang.clone()).await;
    state.simple_config.set_source_language(request.src_lang.clone()).await;
    eprintln!("[S2S] Updated language config: src={}, tgt={}", request.src_lang, request.tgt_lang);

    // 4. 处理所有音频帧，累积到 ASR 缓冲区
    // 对于整句翻译，我们需要处理所有帧，最后一帧应该触发边界检测
    let mut final_result: Option<ProcessResult> = None;
    
    // 处理所有帧，除了最后一帧
    for frame in audio_frames.iter().take(audio_frames.len().saturating_sub(1)) {
        match state.engine.process_audio_frame(frame.clone(), Some(request.src_lang.clone())).await {
            Ok(Some(result)) => {
                // 记录最新结果，但继续处理剩余帧，确保音频被完整消耗
                final_result = Some(result);
            }
            Ok(None) => {
                // 继续处理下一帧（帧被累积到缓冲区）
                continue;
            }
            Err(e) => {
                eprintln!("[ERROR] Error processing audio frame: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
    
    // 5. 处理最后一帧，应该触发边界检测和完整推理
    if final_result.is_none() {
        if let Some(last_frame) = audio_frames.last() {
            // 创建一个标记为边界的帧（通过修改 timestamp 或使用特殊处理）
            // 实际上，SimpleVad 总是返回 is_boundary=true，所以最后一帧应该触发推理
            match state.engine.process_audio_frame(last_frame.clone(), Some(request.src_lang.clone())).await {
                Ok(Some(result)) => {
                    final_result = Some(result);
                }
                Ok(None) => {
                    // 如果没有结果，可能是音频太短或没有检测到语音
                    // 返回错误
                    return Err(StatusCode::BAD_REQUEST);
                }
                Err(e) => {
                    eprintln!("[ERROR] Error processing final audio frame: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
    }

    let result = final_result.ok_or(StatusCode::BAD_REQUEST)?;

    // 6. 提取结果
    let transcript = result
        .asr
        .final_transcript
        .as_ref()
        .map(|t| t.text.clone())
        .unwrap_or_default();

    let translation = result
        .translation
        .as_ref()
        .map(|t| t.translated_text.clone())
        .unwrap_or_default();

    if !transcript.trim().is_empty() {
        eprintln!("[S2S] Transcript: {}", transcript);
    } else {
        eprintln!("[S2S] Transcript: <empty>");
    }
    if !translation.trim().is_empty() {
        eprintln!("[S2S] Translation: {}", translation);
    } else {
        eprintln!("[S2S] Translation: <empty>");
    }

    // 7. 获取 TTS 音频（base64 编码）
    let audio_base64 = if let Some(tts_chunk) = result.tts {
        let audio_size = tts_chunk.audio.len();
        eprintln!("[S2S] TTS audio size: {} bytes", audio_size);
        if audio_size > 0 {
            general_purpose::STANDARD.encode(&tts_chunk.audio)
        } else {
            eprintln!("[S2S] WARNING: TTS audio is empty!");
            String::new()
        }
    } else {
        eprintln!("[S2S] WARNING: TTS result is None!");
        String::new()
    };

    // 8. 计算总时长并返回结果
    let s2s_total_ms = s2s_start.elapsed().as_millis() as u64;
    eprintln!("[S2S] ===== Request completed in {}ms =====", s2s_total_ms);
    
    // 输出详细的时间统计（如果之前记录了各步骤时间）
    // 注意：这里只输出总时长，各步骤的详细时间需要在 process_audio_frame 中记录
    
    Ok(Json(S2SResponse {
        audio: audio_base64,
        transcript,
        translation,
    }))
}

/// 解析 WAV 音频数据为 AudioFrame 列表
fn parse_wav_to_frames(wav_data: &[u8]) -> anyhow::Result<Vec<AudioFrame>> {
    use hound::WavReader;
    
    let cursor = Cursor::new(wav_data);
    let mut reader = WavReader::new(cursor)
        .map_err(|e| anyhow::anyhow!("Failed to create WAV reader: {}", e))?;
    
    let spec = reader.spec();
    
    // 读取所有样本
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

    // 如果音频是立体声，转换为单声道（取平均值）
    let mono_samples = if spec.channels == 2 {
        samples
            .chunks(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    } else {
        samples
    };

    // 如果采样率不是 16kHz，需要重采样
    // 为了简化，这里假设输入音频已经是 16kHz
    // 实际应用中应该添加重采样逻辑
    
    // 按 10ms 一帧拆分（Whisper 期望的格式）
    let frame_size = (spec.sample_rate / 100) as usize;
    let mut frames = Vec::new();
    
    for (idx, chunk) in mono_samples.chunks(frame_size).enumerate() {
        frames.push(AudioFrame {
            sample_rate: spec.sample_rate,
            channels: 1, // 转换为单声道
            data: chunk.to_vec(),
            timestamp_ms: (idx * 10) as u64,
        });
    }

    if let Some(last) = frames.last_mut() {
        last.timestamp_ms |= FINAL_FRAME_FLAG;
    }

    Ok(frames)
}

/// WebSocket 流式翻译端点
async fn stream_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        handle_socket(socket, state).await;
    })
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    eprintln!("[WebSocket] ✅ Client connected");

    // 分离 WebSocket 的发送端和接收端
    let (sender, mut receiver) = socket.split();
    
    // 使用 Arc<Mutex<>> 包装 sender，以便在多个任务中共享
    let sender = Arc::new(tokio::sync::Mutex::new(sender));
    
    let mut src_lang = "en".to_string(); // 默认源语言
    let mut tgt_lang = "zh".to_string(); // 默认目标语言
    let mut frame_count = 0u64;
    
    // 订阅 TTS 事件，用于接收增量音频输出
    let mut tts_receiver_from_bus = state.event_bus.subscribe_receiver(EventTopic("Tts".to_string()));
    eprintln!("[WebSocket] 📡 Subscribed to TTS events");
    
    // 启动任务：从事件总线接收 TTS 事件，按 timestamp_ms 排序后发送到 WebSocket
    let sender_for_tts = Arc::clone(&sender);
    tokio::spawn(async move {
        let mut pending_events: Vec<CoreEvent> = Vec::new();
        let mut next_expected_timestamp = 0u64;
        
        while let Some(event) = tts_receiver_from_bus.recv().await {
            pending_events.push(event);
            
            // 按 timestamp_ms 排序
            pending_events.sort_by_key(|e| e.timestamp_ms);
            
            // 发送所有可以发送的事件（按顺序）
            while let Some(pos) = pending_events.iter().position(|e| e.timestamp_ms >= next_expected_timestamp) {
                let event = pending_events.remove(pos);
                next_expected_timestamp = event.timestamp_ms + 1;  // 更新期望的时间戳
                
                // 解析事件 payload
                if let Some(audio_base64) = event.payload.get("audio").and_then(|v| v.as_str()) {
                    let response_json = serde_json::json!({
                        "type": "tts_chunk",
                        "audio": audio_base64,
                        "timestamp_ms": event.timestamp_ms,
                        "is_last": event.payload.get("is_last").and_then(|v| v.as_bool()).unwrap_or(false),
                    });
                    
                    let mut sender_guard = sender_for_tts.lock().await;
                    if let Err(e) = sender_guard.send(Message::Text(response_json.to_string())).await {
                        eprintln!("[WebSocket] ❌ Failed to send TTS event: {}", e);
                        return;
                    }
                    drop(sender_guard); // 显式释放锁
                    
                    eprintln!("[WebSocket] 📤 Sent TTS chunk (timestamp: {}ms, is_last: {}, audio_size: {} chars)", 
                        event.timestamp_ms,
                        event.payload.get("is_last").and_then(|v| v.as_bool()).unwrap_or(false),
                        audio_base64.len());
                }
            }
        }
    });

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
                            Err(e) => {
                eprintln!("[WebSocket] ❌ Error receiving message: {}", e);
                return;
            }
        };

        match msg {
            Message::Text(text) => {
                // 尝试解析为 JSON（配置或音频帧）
                if let Ok(json_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json_msg["type"] == "config" {
                        // 处理配置消息
                        if let Some(lang) = json_msg["src_lang"].as_str() {
                            src_lang = lang.to_string();
                        }
                        if let Some(lang) = json_msg["tgt_lang"].as_str() {
                            tgt_lang = lang.to_string();
                        }
                        state.simple_config.set_source_language(src_lang.clone()).await;
                        state.simple_config.set_target_language(tgt_lang.clone()).await;
                        eprintln!("[WebSocket] ⚙️ Config updated: src={}, tgt={}", src_lang, tgt_lang);
                    } else if json_msg["type"] == "audio_frame" {
                        // 处理音频帧
                        if let (Some(base64_audio), Some(timestamp_ms), Some(sample_rate), Some(channels)) = (
                            json_msg["data"].as_str(),
                            json_msg["timestamp_ms"].as_u64(),
                            json_msg["sample_rate"].as_u64(),
                            json_msg["channels"].as_u64(),
                        ) {
                            frame_count += 1;
                            
                            // 解码 base64 音频数据
                            let audio_data = match general_purpose::STANDARD.decode(base64_audio) {
                                Ok(data) => data,
                    Err(e) => {
                                    eprintln!("[WebSocket] ❌ Failed to decode base64 audio (frame #{}): {}", frame_count, e);
                                    continue;
                                }
                            };

                            // 将 16-bit PCM 转换为 f32
                            let pcm_data: Vec<i16> = audio_data
                    .chunks_exact(2)
                                .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();
                            let float_data: Vec<f32> = pcm_data.into_iter().map(|s| s as f32 / 32768.0).collect();
                
                // 计算音频统计信息
                            let max_amplitude = float_data.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
                            let rms = (float_data.iter().map(|x| x * x).sum::<f32>() / float_data.len() as f32).sqrt();

                            let audio_frame = AudioFrame {
                                sample_rate: sample_rate as u32,
                                channels: channels as u8,
                                data: float_data,
                                timestamp_ms,
                            };

                            // 每 50 帧输出一次日志，避免日志过多
                            if frame_count % 50 == 0 {
                                eprintln!("[WebSocket] 📥 Received audio frame #{}: {}Hz {}ch, {} samples ({}ms), max={:.4}, rms={:.4}", 
                                    frame_count, sample_rate, channels, audio_frame.data.len(), 
                                    timestamp_ms, max_amplitude, rms);
                            }

                            // 处理音频帧（如果启用了连续模式，会自动使用连续处理逻辑）
                            match state.engine.process_audio_frame(audio_frame, Some(src_lang.clone())).await {
                    Ok(Some(result)) => {
                                    // 发送 ASR 转录、NMT 翻译和 TTS 音频
                                    let tts_audio_base64 = result.tts.as_ref().and_then(|t| {
                                        if t.audio.is_empty() {
                                            eprintln!("[WebSocket] ⚠️ TTS audio is empty!");
                                            None
                                        } else {
                                            eprintln!("[WebSocket] 📤 Sending TTS audio: {} bytes (base64: {} chars)", 
                                                t.audio.len(), 
                                                general_purpose::STANDARD.encode(&t.audio).len());
                                            Some(general_purpose::STANDARD.encode(&t.audio))
                                        }
                                    });
                                    
                                    let response_json = serde_json::json!({
                                        "transcript": result.asr.final_transcript.as_ref().map(|t| t.text.clone()),
                                        "translation": result.translation.as_ref().map(|t| t.translated_text.clone()),
                                        "audio": tts_audio_base64,
                                    });
                                    
                                    eprintln!("[WebSocket] 📤 Sending response: transcript={:?}, translation={:?}, audio={}", 
                                        result.asr.final_transcript.as_ref().map(|t| t.text.as_str()),
                                        result.translation.as_ref().map(|t| t.translated_text.as_str()),
                                        if tts_audio_base64.is_some() { "Yes" } else { "No" });
                                    
                                    let mut sender_guard = sender.lock().await;
                                    if let Err(e) = sender_guard.send(Message::Text(response_json.to_string())).await {
                                        eprintln!("[WebSocket] ❌ Failed to send response: {}", e);
                                        drop(sender_guard);
                                        break;
                                    }
                                    drop(sender_guard); // 显式释放锁
                                    }
                                    Ok(None) => {
                                    // 没有最终结果，继续处理
                                    eprintln!("[WebSocket] ⏳ 处理中，暂无最终结果");
                                    }
                                    Err(e) => {
                                    eprintln!("[WebSocket] ❌ Error processing audio frame #{}: {}", frame_count, e);
                                }
                            }
                        } else {
                            eprintln!("[WebSocket] ⚠️ Invalid audio_frame message format (frame #{})", frame_count);
                        }
                    } else {
                        eprintln!("[WebSocket] ⚠️ Unknown message type: {}", json_msg["type"]);
                    }
                } else {
                    eprintln!("[WebSocket] ⚠️ Failed to parse JSON message");
                }
            }
            Message::Binary(data) => {
                eprintln!("[WebSocket] 📦 Received binary message: {} bytes", data.len());
            }
            Message::Ping(payload) => {
                let mut sender_guard = sender.lock().await;
                if let Err(e) = sender_guard.send(Message::Pong(payload)).await {
                    eprintln!("[WebSocket] ❌ Failed to send Pong: {}", e);
                    drop(sender_guard);
                    break;
                }
                drop(sender_guard); // 显式释放锁
            }
            Message::Pong(_) => {
                // 不做处理
            }
            Message::Close(close_frame) => {
                eprintln!("[WebSocket] 🔌 Client disconnected (frames received: {})", frame_count);
                if let Some(frame) = close_frame {
                    eprintln!("[WebSocket] Close frame: code={:?}, reason={:?}", frame.code, frame.reason);
                }
                break;
            }
        }
    }
    eprintln!("[WebSocket] 👋 Connection closed (total frames: {})", frame_count);
}

/// 获取当前说话者识别模式
#[derive(Debug, Serialize)]
struct SpeakerModeResponse {
    mode: String,  // "single_user" 或 "multi_user"
}

/// 设置说话者识别模式请求
#[derive(Debug, Deserialize)]
struct SetSpeakerModeRequest {
    mode: String,  // "single_user" 或 "multi_user"
}

/// 设置说话者识别模式响应
#[derive(Debug, Serialize)]
struct SetSpeakerModeResponse {
    success: bool,
    message: String,
    current_mode: String,
}

/// 获取当前说话者识别模式
async fn get_speaker_mode(State(state): State<AppState>) -> Json<SpeakerModeResponse> {
    let mode = state.speaker_mode.read().await;
    let mode_str = match *mode {
        EmbeddingBasedMode::SingleUser => "single_user",
        EmbeddingBasedMode::MultiUser => "multi_user",
    };
    Json(SpeakerModeResponse {
        mode: mode_str.to_string(),
    })
}

/// 设置说话者识别模式
async fn set_speaker_mode(
    State(state): State<AppState>,
    Json(request): Json<SetSpeakerModeRequest>,
) -> Result<Json<SetSpeakerModeResponse>, StatusCode> {
    let new_mode = match request.mode.as_str() {
        "single_user" => EmbeddingBasedMode::SingleUser,
        "multi_user" => EmbeddingBasedMode::MultiUser,
        _ => {
            return Ok(Json(SetSpeakerModeResponse {
                success: false,
                message: format!("无效的模式: {}. 有效值: single_user, multi_user", request.mode),
                current_mode: {
                    let current = state.speaker_mode.read().await;
                    match *current {
                        EmbeddingBasedMode::SingleUser => "single_user".to_string(),
                        EmbeddingBasedMode::MultiUser => "multi_user".to_string(),
                    }
                },
            }));
        }
    };
    
    {
        let mut mode = state.speaker_mode.write().await;
        *mode = new_mode;
    }
    
    let mode_str = match new_mode {
        EmbeddingBasedMode::SingleUser => "single_user",
        EmbeddingBasedMode::MultiUser => "multi_user",
    };
    
    // 如果存在 speaker_identifier，直接调用其 set_mode 方法（动态切换，数据保留）
    if let Some(ref identifier) = state.speaker_identifier {
        identifier.set_mode(new_mode).await;
        eprintln!("[CONFIG] 说话者识别模式已动态更新为: {} (数据已保留，不会清空另一种模式的记录)", mode_str);
    } else {
        eprintln!("[CONFIG] 说话者识别模式已更新为: {} (但未找到 identifier，可能需要重启引擎)", mode_str);
    }
    
    Ok(Json(SetSpeakerModeResponse {
        success: true,
        message: format!("模式已更新为: {}. 数据已保留，切换模式不会清空另一种模式的记录", mode_str),
        current_mode: mode_str.to_string(),
    }))
}
