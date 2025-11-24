use std::path::PathBuf;
use std::sync::Arc;

use crate::asr_streaming::{AsrStreaming, AsrResult};
use crate::asr_whisper::WhisperAsrStreaming;
use crate::cache_manager::CacheManager;
use crate::config_manager::ConfigManager;
use crate::emotion_adapter::{EmotionAdapter, EmotionRequest, EmotionResponse};
use crate::error::{EngineError, EngineResult};
use crate::event_bus::{EventBus, CoreEvent, EventTopic};
use crate::nmt_incremental::{NmtIncremental, MarianNmtOnnx, M2M100NmtOnnx, TranslationRequest, TranslationResponse};
use crate::nmt_client::{LocalM2m100HttpClient, NmtClientAdapter};
use crate::persona_adapter::{PersonaAdapter, PersonaContext};
use crate::telemetry::{TelemetryDatum, TelemetrySink};
use crate::tts_streaming::{TtsStreaming, TtsRequest, TtsStreamChunk, VitsTtsEngine, PiperHttpTts, PiperHttpConfig};
use crate::types::{PartialTranscript, StableTranscript};
use crate::vad::VoiceActivityDetector;
use crate::health_check::HealthChecker;
use crate::post_processing::TextPostProcessor;
use crate::performance_logger::{PerformanceLog, PerformanceLogger};
use serde_json::json;
use std::path::Path;
use std::time::Instant;
use uuid::Uuid;


pub struct CoreEngine {
    event_bus: Arc<dyn EventBus>,
    vad: Arc<dyn VoiceActivityDetector>,
    asr: Arc<dyn AsrStreaming>,
    nmt: Arc<dyn NmtIncremental>,
    emotion: Arc<dyn EmotionAdapter>,
    persona: Arc<dyn PersonaAdapter>,
    tts: Arc<dyn TtsStreaming>,
    config: Arc<dyn ConfigManager>,
    cache: Arc<dyn CacheManager>,
    telemetry: Arc<dyn TelemetrySink>,
    // 优化模块
    post_processor: Option<Arc<TextPostProcessor>>,
    perf_logger: Option<Arc<PerformanceLogger>>,
    // 服务 URL（用于健康检查）
    nmt_service_url: Option<String>,
    tts_service_url: Option<String>,
}

pub struct CoreEngineBuilder {
    event_bus: Option<Arc<dyn EventBus>>,
    vad: Option<Arc<dyn VoiceActivityDetector>>,
    asr: Option<Arc<dyn AsrStreaming>>,
    nmt: Option<Arc<dyn NmtIncremental>>,
    emotion: Option<Arc<dyn EmotionAdapter>>,
    persona: Option<Arc<dyn PersonaAdapter>>,
    tts: Option<Arc<dyn TtsStreaming>>,
    config: Option<Arc<dyn ConfigManager>>,
    cache: Option<Arc<dyn CacheManager>>,
    telemetry: Option<Arc<dyn TelemetrySink>>,
    // 优化模块
    post_processor: Option<Arc<TextPostProcessor>>,
    perf_logger: Option<Arc<PerformanceLogger>>,
    // 服务 URL（用于健康检查）
    nmt_service_url: Option<String>,
    tts_service_url: Option<String>,
}

impl CoreEngineBuilder {
    pub fn new() -> Self {
        Self {
            event_bus: None,
            vad: None,
            asr: None,
            nmt: None,
            emotion: None,
            persona: None,
            tts: None,
            config: None,
            cache: None,
            telemetry: None,
            post_processor: None,
            perf_logger: None,
            nmt_service_url: None,
            tts_service_url: None,
        }
    }

    pub fn event_bus(mut self, event_bus: Arc<dyn EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub fn vad(mut self, vad: Arc<dyn VoiceActivityDetector>) -> Self {
        self.vad = Some(vad);
        self
    }

    pub fn asr(mut self, asr: Arc<dyn AsrStreaming>) -> Self {
        self.asr = Some(asr);
        self
    }

    pub fn nmt(mut self, nmt: Arc<dyn NmtIncremental>) -> Self {
        self.nmt = Some(nmt);
        self
    }

    pub fn emotion(mut self, emotion: Arc<dyn EmotionAdapter>) -> Self {
        self.emotion = Some(emotion);
        self
    }

    pub fn persona(mut self, persona: Arc<dyn PersonaAdapter>) -> Self {
        self.persona = Some(persona);
        self
    }

    pub fn tts(mut self, tts: Arc<dyn TtsStreaming>) -> Self {
        self.tts = Some(tts);
        self
    }

    pub fn config(mut self, config: Arc<dyn ConfigManager>) -> Self {
        self.config = Some(config);
        self
    }

    pub fn cache(mut self, cache: Arc<dyn CacheManager>) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn telemetry(mut self, telemetry: Arc<dyn TelemetrySink>) -> Self {
        self.telemetry = Some(telemetry);
        self
    }

    /// 使用默认的 Marian NMT ONNX 模型初始化 NMT 模块
    /// 
    /// 模型路径：`core/engine/models/nmt/marian-en-zh/`
    pub fn nmt_with_default_marian_onnx(mut self) -> EngineResult<Self> {
        // 1. 找到 core/engine 目录
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        
        // 2. 约定的 Marian 模型目录路径
        let model_dir = crate_root.join("models/nmt/marian-en-zh");

        // 3. 检查模型目录是否存在
        if !model_dir.exists() {
            return Err(EngineError::new(format!(
                "Marian NMT model directory not found at: {}. Please ensure the model is exported.",
                model_dir.display()
            )));
        }

        // 4. 加载真实的 ONNX 实现
        let nmt_impl = MarianNmtOnnx::new_from_dir(&model_dir)
            .map_err(|e| EngineError::new(format!("Failed to load MarianNmtOnnx: {}", e)))?;

        // 5. 存入 builder 的 nmt 字段
        self.nmt = Some(Arc::new(nmt_impl));

        Ok(self)
    }

    /// 使用默认的 M2M100 NMT ONNX 模型初始化 NMT 模块
    /// 
    /// 模型路径：`core/engine/models/nmt/m2m100-en-zh/`
    /// 
    /// 注意：M2M100 是新的 NMT 模型，推荐使用此方法替代 Marian
    /// 
    /// @deprecated 推荐使用 `nmt_with_m2m100_http_client()` 替代，以获得更好的翻译质量和稳定性
    pub fn nmt_with_default_m2m100_onnx(mut self) -> EngineResult<Self> {
        // 1. 找到 core/engine 目录
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        
        // 2. 约定的 M2M100 模型目录路径（默认使用 en-zh）
        let model_dir = crate_root.join("models/nmt/m2m100-en-zh");

        // 3. 检查模型目录是否存在
        if !model_dir.exists() {
            return Err(EngineError::new(format!(
                "M2M100 NMT model directory not found at: {}. Please ensure the model is exported.",
                model_dir.display()
            )));
        }

        // 4. 加载真实的 ONNX 实现
        let nmt_impl = M2M100NmtOnnx::new_from_dir(&model_dir)
            .map_err(|e| EngineError::new(format!("Failed to load M2M100NmtOnnx: {}", e)))?;

        // 5. 存入 builder 的 nmt 字段
        self.nmt = Some(Arc::new(nmt_impl));

        Ok(self)
    }

    /// 使用 M2M100 HTTP 客户端初始化 NMT 模块（推荐）
    /// 
    /// 此方法连接到本地运行的 Python M2M100 服务，提供更稳定和高质量的翻译。
    /// 
    /// # Arguments
    /// * `service_url` - Python 服务的 URL，默认为 "http://127.0.0.1:5008"
    /// 
    /// # 前置条件
    /// 需要先启动 Python M2M100 服务：
    /// ```bash
    /// cd services/nmt_m2m100
    /// uvicorn nmt_service:app --host 127.0.0.1 --port 5008
    /// ```
    /// 
    /// # 优势
    /// - 翻译质量更稳定（使用 HuggingFace Transformers）
    /// - 代码更简洁（无需管理复杂的 KV Cache）
    /// - 易于维护和调试
    pub fn nmt_with_m2m100_http_client(mut self, service_url: Option<&str>) -> EngineResult<Self> {
        let url = service_url.unwrap_or("http://127.0.0.1:5008");
        
        // 保存服务 URL（用于健康检查）
        self.nmt_service_url = Some(url.to_string());
        
        // 创建 HTTP 客户端
        let client = Arc::new(LocalM2m100HttpClient::new(url));
        
        // 创建适配器（实现 NmtIncremental trait）
        let nmt_impl = NmtClientAdapter::new(client);
        
        // 存入 builder 的 nmt 字段
        self.nmt = Some(Arc::new(nmt_impl));
        
        Ok(self)
    }

    /// 使用指定语言对的 M2M100 NMT ONNX 模型初始化 NMT 模块
    /// 
    /// # Arguments
    /// * `direction` - 翻译方向，如 "en-zh" 或 "zh-en"
    /// 
    /// 模型路径：
    /// - `core/engine/models/nmt/m2m100-en-zh/` (en-zh)
    /// - `core/engine/models/nmt/m2m100-zh-en/` (zh-en)
    pub fn nmt_with_m2m100_onnx(mut self, direction: &str) -> EngineResult<Self> {
        // 1. 找到 core/engine 目录
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        
        // 2. 根据方向确定模型目录
        let model_dir = if direction == "en-zh" || direction == "en_zh" {
            crate_root.join("models/nmt/m2m100-en-zh")
        } else if direction == "zh-en" || direction == "zh_en" {
            crate_root.join("models/nmt/m2m100-zh-en")
        } else {
            return Err(EngineError::new(format!(
                "Invalid translation direction: {}. Supported: en-zh, zh-en",
                direction
            )));
        };

        // 3. 检查模型目录是否存在
        if !model_dir.exists() {
            return Err(EngineError::new(format!(
                "M2M100 NMT model directory not found at: {}. Please ensure the model is exported.",
                model_dir.display()
            )));
        }

        // 4. 加载真实的 ONNX 实现
        let nmt_impl = M2M100NmtOnnx::new_from_dir(&model_dir)
            .map_err(|e| EngineError::new(format!("Failed to load M2M100NmtOnnx: {}", e)))?;

        // 5. 存入 builder 的 nmt 字段
        self.nmt = Some(Arc::new(nmt_impl));

        Ok(self)
    }

    /// 使用默认的 Marian NMT Stub（已废弃，保留用于向后兼容）
    /// 
    /// @deprecated 请使用 `nmt_with_default_marian_onnx()` 代替
    #[deprecated(note = "Use nmt_with_default_marian_onnx() instead")]
    pub fn nmt_with_default_marian_stub(self) -> EngineResult<Self> {
        // 为了向后兼容，调用新的方法
        self.nmt_with_default_marian_onnx()
    }

    /// 使用默认的 Whisper ASR 模型初始化 ASR 模块
    /// 
    /// 模型路径：`core/engine/models/asr/whisper-base/`
    /// 
    /// # Returns
    /// 返回 `EngineResult<Self>`，如果模型目录不存在或加载失败则返回错误
    pub fn asr_with_default_whisper(mut self) -> EngineResult<Self> {
        // 1. 找到 core/engine 目录
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        
        // 2. 约定的 Whisper 模型目录路径
        let model_dir = crate_root.join("models/asr/whisper-base");

        // 3. 检查模型目录是否存在
        if !model_dir.exists() {
            return Err(EngineError::new(format!(
                "Whisper ASR model directory not found at: {}. Please ensure the model is downloaded.",
                model_dir.display()
            )));
        }

        // 4. 加载 Whisper ASR 实现
        let asr_impl = WhisperAsrStreaming::new_from_dir(&model_dir)
            .map_err(|e| EngineError::new(format!("Failed to load WhisperAsrStreaming: {}", e)))?;

        // 5. 存入 builder 的 asr 字段
        self.asr = Some(Arc::new(asr_impl));

        Ok(self)
    }

    /// 使用默认的 VITS TTS 模型初始化 TTS 模块（支持多语言）
    /// 
    /// 模型路径：`core/engine/models/tts/`
    /// - 英文模型：`mms-tts-eng/`（必需）
    /// - 中文模型：`mms-tts-zh-Hans/`（可选）
    /// 
    /// # Returns
    /// 返回 `EngineResult<Self>`，如果英文模型目录不存在或加载失败则返回错误
    pub fn tts_with_default_vits(mut self) -> EngineResult<Self> {
        // 1. 找到 core/engine 目录
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        
        // 2. 约定的 VITS TTS 模型根目录路径
        let models_root = crate_root.join("models/tts");

        // 3. 检查模型根目录是否存在
        if !models_root.exists() {
            return Err(EngineError::new(format!(
                "VITS TTS models root directory not found at: {}. Please ensure the models are downloaded.",
                models_root.display()
            )));
        }

        // 4. 加载 VITS TTS 实现（支持多语言）
        let tts_impl = VitsTtsEngine::new_from_models_root(&models_root)
            .map_err(|e| EngineError::new(format!("Failed to load VitsTtsEngine: {}", e)))?;

        // 5. 存入 builder 的 tts 字段
        self.tts = Some(Arc::new(tts_impl));

        Ok(self)
    }

    /// 使用默认的 Piper HTTP TTS 服务初始化 TTS 模块
    /// 
    /// 配置：
    /// - 端点：http://127.0.0.1:5005/tts
    /// - 默认语音：zh_CN-huayan-medium
    /// - 超时：8000ms
    /// 
    /// 注意：此方法需要 WSL2 中运行 Piper HTTP 服务
    pub fn tts_with_default_piper_http(mut self) -> EngineResult<Self> {
        let config = PiperHttpConfig::default();
        
        // 保存服务 URL（用于健康检查）
        // 从 endpoint 中提取基础 URL（去掉 /tts 后缀）
        let base_url = config.endpoint
            .strip_suffix("/tts")
            .unwrap_or(&config.endpoint)
            .to_string();
        self.tts_service_url = Some(base_url);
        
        let tts_impl = PiperHttpTts::new(config)
            .map_err(|e| EngineError::new(format!("Failed to create PiperHttpTts: {}", e)))?;
        
        self.tts = Some(Arc::new(tts_impl));
        Ok(self)
    }

    /// 使用自定义配置的 Piper HTTP TTS 服务初始化 TTS 模块
    /// 
    /// # Arguments
    /// * `config` - Piper HTTP 配置
    pub fn tts_with_piper_http(mut self, config: PiperHttpConfig) -> EngineResult<Self> {
        // 保存服务 URL（用于健康检查）
        let base_url = config.endpoint
            .strip_suffix("/tts")
            .unwrap_or(&config.endpoint)
            .to_string();
        self.tts_service_url = Some(base_url);
        
        let tts_impl = PiperHttpTts::new(config)
            .map_err(|e| EngineError::new(format!("Failed to create PiperHttpTts: {}", e)))?;
        
        self.tts = Some(Arc::new(tts_impl));
        Ok(self)
    }
    
    /// 启用文本后处理
    /// 
    /// # Arguments
    /// * `terms_file` - 术语表文件路径（可选）
    /// * `enabled` - 是否启用后处理
    pub fn with_post_processing(mut self, terms_file: Option<&Path>, enabled: bool) -> Self {
        let processor = TextPostProcessor::new(terms_file, enabled);
        self.post_processor = Some(Arc::new(processor));
        self
    }
    
    /// 启用性能日志
    /// 
    /// # Arguments
    /// * `enabled` - 是否启用性能日志
    /// * `log_suspect` - 是否记录可疑翻译
    pub fn with_performance_logging(mut self, enabled: bool, log_suspect: bool) -> Self {
        let logger = PerformanceLogger::new(enabled, log_suspect);
        self.perf_logger = Some(Arc::new(logger));
        self
    }

    pub fn build(self) -> EngineResult<CoreEngine> {
        Ok(CoreEngine {
            event_bus: self.event_bus.ok_or_else(|| EngineError::new("event_bus is missing"))?,
            vad: self.vad.ok_or_else(|| EngineError::new("vad is missing"))?,
            asr: self.asr.ok_or_else(|| EngineError::new("asr is missing"))?,
            nmt: self.nmt.ok_or_else(|| EngineError::new("nmt is missing"))?,
            emotion: self.emotion.ok_or_else(|| EngineError::new("emotion is missing"))?,
            persona: self.persona.ok_or_else(|| EngineError::new("persona is missing"))?,
            tts: self.tts.ok_or_else(|| EngineError::new("tts is missing"))?,
            config: self.config.ok_or_else(|| EngineError::new("config is missing"))?,
            cache: self.cache.ok_or_else(|| EngineError::new("cache is missing"))?,
            telemetry: self.telemetry.ok_or_else(|| EngineError::new("telemetry is missing"))?,
            post_processor: self.post_processor,
            perf_logger: self.perf_logger,
            nmt_service_url: self.nmt_service_url,
            tts_service_url: self.tts_service_url,
        })
    }
}

impl CoreEngine {
    pub async fn boot(&self) -> EngineResult<()> {
        self.event_bus.start().await?;
        let config = self.config.load().await?;
        self.cache.warm_up().await?;
        self.asr.initialize().await?;
        self.nmt.initialize().await?;
        
        // 健康检查：检查 NMT 和 TTS 服务
        if let (Some(nmt_url), Some(tts_url)) = (&self.nmt_service_url, &self.tts_service_url) {
            let checker = HealthChecker::new();
            let (nmt_health, tts_health) = checker.check_all_services(nmt_url, tts_url).await;
            
            if !nmt_health.is_healthy {
                eprintln!("[WARN] NMT service is not healthy: {} - {:?}", nmt_url, nmt_health.error);
                // 不阻止启动，但记录警告
            } else {
                println!("[INFO] NMT service health check passed: {}", nmt_url);
            }
            
            if !tts_health.is_healthy {
                eprintln!("[WARN] TTS service is not healthy: {} - {:?}", tts_url, tts_health.error);
                // 不阻止启动，但记录警告
            } else {
                println!("[INFO] TTS service health check passed: {}", tts_url);
            }
        }
        
        self.telemetry
            .record(TelemetryDatum {
                name: "core_engine.boot".to_string(),
                value: 1.0,
                unit: "count".to_string(),
            })
            .await?;
        self.telemetry
            .record(TelemetryDatum {
                name: format!("core_engine.mode.{}", config.mode),
                value: 1.0,
                unit: "count".to_string(),
            })
            .await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> EngineResult<()> {
        self.asr.finalize().await?;
        self.nmt.finalize().await?;
        self.tts.close().await?;
        self.cache.purge().await?;
        self.event_bus.stop().await?;
        self.telemetry
            .record(TelemetryDatum {
                name: "core_engine.shutdown".to_string(),
                value: 1.0,
                unit: "count".to_string(),
            })
            .await?;
        Ok(())
    }

    /// 处理音频帧（完整业务流程：VAD → ASR → NMT → 事件发布）
    /// 
    /// 流程：
    /// 1. 通过 VAD 检测语音活动
    /// 2. 如果检测到语音，累积到 ASR 缓冲区
    /// 3. 如果检测到语音边界（is_boundary），触发 ASR 推理
    /// 4. 如果 ASR 返回最终结果，自动触发 NMT 翻译
    /// 5. 发布事件到 EventBus（ASR 部分结果、ASR 最终结果、翻译结果）
    /// 
    /// # Arguments
    /// * `frame` - 音频帧
    /// * `language_hint` - 语言提示（可选）
    /// 
    /// # Returns
    /// 返回处理结果（包含 ASR 和 NMT 结果）
    pub async fn process_audio_frame(
        &self,
        frame: crate::types::AudioFrame,
        language_hint: Option<String>,
    ) -> EngineResult<Option<ProcessResult>> {
        // 性能日志：记录总耗时
        let total_start = Instant::now();
        let request_id = Uuid::new_v4().to_string();
        
        // 1. 通过 VAD 检测语音活动
        let vad_result = self.vad.detect(frame).await?;

        // 2. 累积音频帧到 ASR 缓冲区
        // 尝试将 ASR 转换为 WhisperAsrStreaming
        let asr_ptr = Arc::as_ptr(&self.asr);
        let whisper_asr_ptr = asr_ptr as *const WhisperAsrStreaming;
        
        unsafe {
            let whisper_asr_ref = whisper_asr_ptr.as_ref();
            if let Some(whisper_asr) = whisper_asr_ref {
                // 累积帧
                whisper_asr.accumulate_frame(vad_result.frame.clone())?;
                
                // 3. 如果检测到语音边界，触发 ASR 推理（返回最终结果）
                if vad_result.is_boundary {
                    let asr_start = Instant::now();
                    let asr_result = whisper_asr.infer_on_boundary().await?;
                    let asr_ms = asr_start.elapsed().as_millis() as u64;
                    
                    // 4. 发布 ASR 最终结果事件
                    if let Some(ref final_transcript) = asr_result.final_transcript {
                        self.publish_asr_final_event(final_transcript, vad_result.frame.timestamp_ms).await?;
                    }
                    
                    // 5. 如果 ASR 返回最终结果，进行 Emotion 分析、Persona 个性化，然后触发 NMT 翻译
                    let (emotion_result, translation_result, tts_result, nmt_ms, tts_ms) = if let Some(ref final_transcript) = asr_result.final_transcript {
                        // 5.1. Emotion 情感分析
                        let emotion_result = self.analyze_emotion(final_transcript, vad_result.frame.timestamp_ms).await.ok();
                        
                        // 5.2. 应用 Persona 个性化
                        let personalized_transcript = self.personalize_transcript(final_transcript).await?;
                        
                        // 5.3. 使用个性化后的 transcript 进行翻译
                        let nmt_start = Instant::now();
                        let translation_result = self.translate_and_publish(&personalized_transcript, vad_result.frame.timestamp_ms).await.ok();
                        let nmt_ms = nmt_start.elapsed().as_millis() as u64;
                        
                        // 5.4. 如果翻译成功，进行 TTS 合成
                        let (tts_result, tts_ms) = if let Some(ref translation) = translation_result {
                            let tts_start = Instant::now();
                            let result = self.synthesize_and_publish(translation, vad_result.frame.timestamp_ms).await.ok();
                            let tts_ms = tts_start.elapsed().as_millis() as u64;
                            (result, tts_ms)
                        } else {
                            (None, 0)
                        };
                        
                        // 性能日志记录
                        if let Some(ref logger) = self.perf_logger {
                            let total_ms = total_start.elapsed().as_millis() as u64;
                            let config = self.config.current().await.ok();
                            let src_lang = final_transcript.language.clone();
                            let tgt_lang = config.as_ref().map(|c| c.target_language.clone()).unwrap_or_else(|| "zh".to_string());
                            
                            let mut perf_log = PerformanceLog::new(
                                request_id.clone(),
                                src_lang,
                                tgt_lang,
                                asr_ms,
                                nmt_ms,
                                tts_ms,
                                total_ms,
                                translation_result.is_some(),
                            );
                            
                            if let Some(ref translation) = translation_result {
                                perf_log.check_suspect_translation(&final_transcript.text, &translation.translated_text);
                            }
                            
                            logger.log(&perf_log);
                        }
                        
                        (emotion_result, translation_result, tts_result, nmt_ms, tts_ms)
                    } else {
                        (None, None, None, 0, 0)
                    };
                    
                    return Ok(Some(ProcessResult {
                        asr: asr_result,
                        emotion: emotion_result,
                        translation: translation_result,
                        tts: tts_result,
                    }));
                } else {
                    // 未检测到边界，检查是否需要输出部分结果（如果启用流式推理）
                    if whisper_asr.is_streaming_enabled() {
                        if let Some(partial) = whisper_asr.infer_partial(vad_result.frame.timestamp_ms).await? {
                            // 发布 ASR 部分结果事件
                            self.publish_asr_partial_event(&partial, vad_result.frame.timestamp_ms).await?;
                            
                            return Ok(Some(ProcessResult {
                                asr: AsrResult {
                                    partial: Some(partial),
                                    final_transcript: None,
                                },
                                emotion: None,
                                translation: None,
                                tts: None,
                            }));
                        }
                    }
                    // 不需要输出部分结果，返回 None
                    return Ok(None);
                }
            } else {
                // 如果不是 WhisperAsrStreaming，使用原来的 infer 方法
                let frame_timestamp = vad_result.frame.timestamp_ms;
                let asr_result = self.asr.infer(crate::asr_streaming::AsrRequest {
                    frame: vad_result.frame,
                    language_hint: language_hint.clone(),
                }).await?;
                
                // 如果检测到边界且有最终结果，进行 Emotion 分析、Persona 个性化，然后触发翻译
                if vad_result.is_boundary {
                    if let Some(ref final_transcript) = asr_result.final_transcript {
                        self.publish_asr_final_event(final_transcript, frame_timestamp).await?;
                        
                        // Emotion 情感分析
                        let emotion_result = self.analyze_emotion(final_transcript, frame_timestamp).await.ok();
                        
                        // 应用 Persona 个性化
                        let personalized_transcript = self.personalize_transcript(final_transcript).await?;
                        
                        // 使用个性化后的 transcript 进行翻译
                        let translation_result = self.translate_and_publish(&personalized_transcript, frame_timestamp).await.ok();
                        
                        // 如果翻译成功，进行 TTS 合成
                        let tts_result = if let Some(ref translation) = translation_result {
                            self.synthesize_and_publish(translation, frame_timestamp).await.ok()
                        } else {
                            None
                        };
                        
                        return Ok(Some(ProcessResult {
                            asr: asr_result,
                            emotion: emotion_result,
                            translation: translation_result,
                            tts: tts_result,
                        }));
                    }
                }
                
                // 如果有部分结果，发布事件
                if let Some(ref partial) = asr_result.partial {
                    self.publish_asr_partial_event(partial, frame_timestamp).await?;
                }
                
                return Ok(Some(ProcessResult {
                    asr: asr_result,
                    emotion: None,
                    translation: None,
                    tts: None,
                }));
            }
        }
    }

    /// 分析情感
    async fn analyze_emotion(
        &self,
        transcript: &StableTranscript,
        timestamp_ms: u64,
    ) -> EngineResult<EmotionResponse> {
        // 构造 Emotion 请求（根据 Emotion_Adapter_Spec.md）
        let request = EmotionRequest {
            text: transcript.text.clone(),
            lang: transcript.language.clone(),
        };
        
        // 执行情感分析
        let response = self.emotion.analyze(request).await?;
        
        // 发布 Emotion 事件
        self.publish_emotion_event(&response, timestamp_ms).await?;
        
        Ok(response)
    }

    /// 应用 Persona 个性化
    async fn personalize_transcript(
        &self,
        transcript: &StableTranscript,
    ) -> EngineResult<StableTranscript> {
        // 从配置中获取 PersonaContext（简化版：使用默认值）
        // TODO: 后续可以从用户配置或数据库获取真实的 PersonaContext
        let context = PersonaContext {
            user_id: "default_user".to_string(),
            tone: "formal".to_string(),  // 默认使用正式语调
            culture: transcript.language.clone(),
        };
        
        // 应用个性化
        self.persona.personalize(transcript.clone(), context).await
    }

    /// 翻译并发布事件
    async fn translate_and_publish(
        &self,
        transcript: &StableTranscript,
        timestamp_ms: u64,
    ) -> EngineResult<TranslationResponse> {
        // 1. 获取目标语言（从配置中）
        let config = self.config.current().await?;
        let target_language = config.target_language.clone();
        
        // 2. 构造翻译请求
        let translation_request = TranslationRequest {
            transcript: PartialTranscript {
                text: transcript.text.clone(),
                confidence: 1.0,  // 最终转录的置信度
                is_final: true,
            },
            target_language: target_language.clone(),
            wait_k: None,
        };
        
        // 3. 执行翻译
        let mut translation_response = self.nmt.translate(translation_request).await?;
        
        // 4. 文本后处理
        if let Some(ref processor) = self.post_processor {
            let processed_text = processor.process(&translation_response.translated_text, &target_language);
            translation_response.translated_text = processed_text;
        }
        
        // 5. 发布翻译事件
        self.publish_translation_event(&translation_response, timestamp_ms).await?;
        
        Ok(translation_response)
    }

    /// 发布 ASR 部分结果事件
    async fn publish_asr_partial_event(
        &self,
        partial: &PartialTranscript,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("AsrPartial".to_string()),
            payload: json!({
                "text": partial.text,
                "confidence": partial.confidence,
                "is_final": partial.is_final,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }

    /// 发布 ASR 最终结果事件
    async fn publish_asr_final_event(
        &self,
        final_transcript: &StableTranscript,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("AsrFinal".to_string()),
            payload: json!({
                "text": final_transcript.text,
                "speaker_id": final_transcript.speaker_id,
                "language": final_transcript.language,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }

    /// 发布 Emotion 事件
    async fn publish_emotion_event(
        &self,
        emotion: &EmotionResponse,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("Emotion".to_string()),
            payload: json!({
                "primary": emotion.primary,
                "intensity": emotion.intensity,
                "confidence": emotion.confidence,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }

    /// 发布翻译事件
    async fn publish_translation_event(
        &self,
        translation: &TranslationResponse,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("Translation".to_string()),
            payload: json!({
                "translated_text": translation.translated_text,
                "is_stable": translation.is_stable,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }

    /// TTS 合成并发布事件
    async fn synthesize_and_publish(
        &self,
        translation: &TranslationResponse,
        timestamp_ms: u64,
    ) -> EngineResult<TtsStreamChunk> {
        // 1. 获取目标语言（用于 TTS locale）
        let config = self.config.current().await?;
        let target_language = config.target_language.clone();
        
        // 2. 构造 TTS 请求
        let tts_request = TtsRequest {
            text: translation.translated_text.clone(),
            voice: "default".to_string(),  // TODO: 后续可以从配置或 Emotion 结果中选择 voice
            locale: target_language.clone(),
        };
        
        // 3. 执行 TTS 合成
        let tts_chunk = self.tts.synthesize(tts_request).await?;
        
        // 4. 发布 TTS 事件
        self.publish_tts_event(&tts_chunk, timestamp_ms).await?;
        
        Ok(tts_chunk)
    }

    /// 发布 TTS 事件
    async fn publish_tts_event(
        &self,
        tts_chunk: &TtsStreamChunk,
        timestamp_ms: u64,
    ) -> EngineResult<()> {
        let event = CoreEvent {
            topic: EventTopic("Tts".to_string()),
            payload: json!({
                "audio_length": tts_chunk.audio.len(),
                "timestamp_ms": tts_chunk.timestamp_ms,
                "is_last": tts_chunk.is_last,
            }),
            timestamp_ms,
        };
        self.event_bus.publish(event).await?;
        Ok(())
    }
}

/// 处理结果（包含 ASR、Emotion、NMT 和 TTS 结果）
#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub asr: AsrResult,
    pub emotion: Option<EmotionResponse>,
    pub translation: Option<TranslationResponse>,
    pub tts: Option<TtsStreamChunk>,
}
