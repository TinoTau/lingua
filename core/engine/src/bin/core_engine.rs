use std::path::PathBuf;
use std::sync::Arc;
use std::io::Cursor;
use axum::{
    extract::{ws::WebSocketUpgrade, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
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
use core_engine::event_bus::{EventBus, CoreEvent, EventTopic, EventSubscription};
use core_engine::vad::{VoiceActivityDetector, DetectionOutcome};
use core_engine::cache_manager::CacheManager;
use core_engine::telemetry::{TelemetrySink, TelemetryDatum};
use async_trait::async_trait;

/// 运行时配置（从 TOML 文件加载）
#[derive(Debug, Clone, Deserialize)]
struct RuntimeConfig {
    nmt: NmtConfig,
    tts: TtsConfig,
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
struct EngineRuntimeConfig {
    port: u16,
    whisper_model_path: Option<String>,
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

struct SimpleVad;

#[async_trait]
impl VoiceActivityDetector for SimpleVad {
    async fn detect(&self, frame: AudioFrame) -> EngineResult<DetectionOutcome> {
        Ok(DetectionOutcome {
            is_boundary: true,
            confidence: 1.0,
            frame,
        })
    }
}

struct SimpleConfig {
    source_lang: String,
    target_lang: String,
}

#[async_trait]
impl ConfigManager for SimpleConfig {
    async fn load(&self) -> EngineResult<EngineConfig> {
        Ok(EngineConfig {
            mode: "balanced".to_string(),
            source_language: self.source_lang.clone(),
            target_language: self.target_lang.clone(),
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

    println!("Loading config from: {}", config_path.display());

    // 2. 加载配置文件
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))?;
    let runtime_config: RuntimeConfig = toml::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;

    println!("Config loaded:");
    println!("  NMT URL: {}", runtime_config.nmt.url);
    println!("  TTS URL: {}", runtime_config.tts.url);
    println!("  Engine Port: {}", runtime_config.engine.port);

    // 3. 初始化 CoreEngine
    let engine = initialize_engine(&runtime_config).await?;
    println!("CoreEngine initialized successfully");

    // 4. 启动 HTTP 服务器
    let app_state = AppState {
        engine: Arc::new(engine),
        config: runtime_config.clone(),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/s2s", post(s2s_handler))
        .route("/stream", get(stream_handler))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let addr = format!("0.0.0.0:{}", runtime_config.engine.port);
    println!("Starting HTTP server on {}", addr);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 初始化 CoreEngine
async fn initialize_engine(config: &RuntimeConfig) -> EngineResult<CoreEngine> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    // 确定 Whisper 模型路径
    let whisper_model_path = config.engine.whisper_model_path.clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| crate_root.join("models/asr/whisper-base"));

    // 构建 CoreEngine
    let engine = CoreEngineBuilder::new()
        .event_bus(Arc::new(SimpleEventBus))
        .vad(Arc::new(SimpleVad))
        .asr_with_default_whisper()
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to initialize ASR: {}", e)))?
        .nmt_with_m2m100_http_client(Some(&config.nmt.url))
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to initialize NMT: {}", e)))?
        .emotion(Arc::new(EmotionStub))
        .persona(Arc::new(PersonaStub))
        .tts_with_piper_http(core_engine::tts_streaming::PiperHttpConfig {
            endpoint: config.tts.url.clone(),
            default_voice: "zh_CN-huayan-medium".to_string(),
            timeout_ms: 8000,
        })
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to initialize TTS: {}", e)))?
        .config(Arc::new(SimpleConfig {
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
        }))
        .cache(Arc::new(SimpleCache))
        .telemetry(Arc::new(SimpleTelemetry))
        .with_text_post_processing(true)
        .with_incremental_tts(true, 0, 50)
        .with_audio_enhancement(core_engine::tts_audio_enhancement::AudioEnhancementConfig::default())
        .build()
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to build engine: {}", e)))?;

    // 启动引擎
    engine.boot().await
        .map_err(|e| core_engine::error::EngineError::new(format!("Failed to boot engine: {}", e)))?;

    Ok(engine)
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
    // 1. 解码 base64 音频
    let audio_data = general_purpose::STANDARD
        .decode(&request.audio)
        .map_err(|e| {
            eprintln!("Failed to decode base64 audio: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    // 2. 解析 WAV 音频并转换为 AudioFrame 列表
    let audio_frames = parse_wav_to_frames(&audio_data)
        .map_err(|e| {
            eprintln!("Failed to parse WAV audio: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    if audio_frames.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 3. 设置目标语言（从请求中获取）
    // 注意：这里需要更新 ConfigManager 的目标语言
    // 为了简化，我们假设配置已经正确设置

    // 4. 处理所有音频帧，累积到 ASR 缓冲区
    // 对于整句翻译，我们需要处理所有帧，最后一帧应该触发边界检测
    let mut final_result: Option<ProcessResult> = None;
    
    // 处理所有帧，除了最后一帧
    for frame in audio_frames.iter().take(audio_frames.len().saturating_sub(1)) {
        match state.engine.process_audio_frame(frame.clone(), Some(request.src_lang.clone())).await {
            Ok(Some(result)) => {
                // 如果提前返回了结果，使用它
                final_result = Some(result);
                break;
            }
            Ok(None) => {
                // 继续处理下一帧（帧被累积到缓冲区）
                continue;
            }
            Err(e) => {
                eprintln!("Error processing audio frame: {}", e);
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
                    eprintln!("Error processing final audio frame: {}", e);
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

    // 7. 获取 TTS 音频（base64 编码）
    let audio_base64 = if let Some(tts_chunk) = result.tts {
        general_purpose::STANDARD.encode(&tts_chunk.audio)
    } else {
        String::new()
    };

    // 8. 返回结果
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

    Ok(frames)
}

/// WebSocket 流式翻译端点
async fn stream_handler(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
) -> Response {
    // TODO: 实现 WebSocket 流式处理
    ws.on_upgrade(|_socket| async move {
        // WebSocket 处理逻辑
    })
}
