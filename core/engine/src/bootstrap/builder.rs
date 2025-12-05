use std::path::PathBuf;
use std::sync::Arc;
use std::path::Path;

use crate::asr_streaming::AsrStreaming;
use crate::asr_whisper::{WhisperAsrStreaming, FasterWhisperAsrStreaming};
use crate::audio_buffer::AudioBufferManager;
use crate::speaker_voice_mapper::SpeakerVoiceMapper;
use crate::speaker_identifier::{SpeakerIdentifier, SpeakerIdentifierMode, VadBasedSpeakerIdentifier, EmbeddingBasedSpeakerIdentifier};
use crate::cache_manager::CacheManager;
use crate::config_manager::ConfigManager;
use crate::emotion_adapter::EmotionAdapter;
use crate::error::{EngineError, EngineResult};
use crate::event_bus::EventBus;
use crate::nmt_incremental::{NmtIncremental, MarianNmtOnnx, M2M100NmtOnnx};
use crate::nmt_client::{LocalM2m100HttpClient, NmtClientAdapter};
use crate::persona_adapter::PersonaAdapter;
use crate::telemetry::TelemetrySink;
use crate::tts_streaming::{TtsStreaming, VitsTtsEngine, PiperHttpTts, PiperHttpConfig, YourTtsHttp, YourTtsHttpConfig};
use crate::vad::VoiceActivityDetector;
use crate::post_processing::TextPostProcessor;
use crate::performance_logger::PerformanceLogger;
use crate::text_segmentation::TextSegmenter;
use crate::translation_quality::TranslationQualityChecker;
use crate::tts_audio_enhancement::{AudioEnhancer, AudioEnhancementConfig};

use super::core::CoreEngine;


pub struct CoreEngineBuilder {
    event_bus: Option<Arc<dyn EventBus>>,
    vad: Option<Arc<dyn VoiceActivityDetector>>,
    asr: Option<Arc<dyn AsrStreaming>>,
    nmt: Option<Arc<dyn NmtIncremental>>,
    emotion: Option<Arc<dyn EmotionAdapter>>,
    persona: Option<Arc<dyn PersonaAdapter>>,
    tts: Option<Arc<dyn TtsStreaming>>,
    fallback_tts: Option<Arc<dyn TtsStreaming>>,  // 回退 TTS 服务
    config: Option<Arc<dyn ConfigManager>>,
    cache: Option<Arc<dyn CacheManager>>,
    telemetry: Option<Arc<dyn TelemetrySink>>,
    // 优化模块
    post_processor: Option<Arc<TextPostProcessor>>,
    perf_logger: Option<Arc<PerformanceLogger>>,
    text_segmenter: Option<Arc<TextSegmenter>>,
    audio_enhancer: Option<Arc<AudioEnhancer>>,
    quality_checker: Option<Arc<TranslationQualityChecker>>,
    // 服务 URL（用于健康检查）
    nmt_service_url: Option<String>,
    tts_service_url: Option<String>,
    // TTS 增量播放配置
    tts_incremental_enabled: bool,
    tts_buffer_sentences: usize,
    // 连续输入输出支持
    audio_buffer: Option<Arc<AudioBufferManager>>,
    continuous_mode: bool,
    // TTS 多说话者音色区分
    speaker_voice_mapper: Option<Arc<SpeakerVoiceMapper>>,
    // 说话者识别
    speaker_identifier: Option<Arc<dyn SpeakerIdentifier>>,
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
            fallback_tts: None,
            config: None,
            cache: None,
            telemetry: None,
            post_processor: None,
            perf_logger: None,
            text_segmenter: None,
            audio_enhancer: None,
            quality_checker: None,
            nmt_service_url: None,
            tts_service_url: None,
            tts_incremental_enabled: false,
            tts_buffer_sentences: 0,
            audio_buffer: None,
            continuous_mode: false,
            speaker_voice_mapper: None,
            speaker_identifier: None,
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

    /// 使用 Faster-Whisper ASR 服务初始化 ASR 模块（通过 HTTP 调用 Python 服务）
    /// 
    /// # Arguments
    /// * `service_url` - ASR 服务的 URL（例如："http://127.0.0.1:6006"）
    /// * `timeout_secs` - HTTP 请求超时时间（秒），默认 30 秒
    /// 
    /// # Returns
    /// 返回 `EngineResult<Self>`，如果创建失败则返回错误
    /// 
    /// # Example
    /// ```rust
    /// let builder = CoreEngineBuilder::new()
    ///     .asr_with_faster_whisper("http://127.0.0.1:6006", 30)?;
    /// ```
    pub fn asr_with_faster_whisper(mut self, service_url: String, timeout_secs: u64) -> EngineResult<Self> {
        eprintln!("[CoreEngineBuilder] 🔧 Initializing Faster-Whisper ASR (service: {})", service_url);
        
        // 创建 FasterWhisperAsrStreaming 实例
        let asr_impl = FasterWhisperAsrStreaming::new(service_url, timeout_secs);
        
        // 存入 builder 的 asr 字段
        self.asr = Some(Arc::new(asr_impl));
        
        eprintln!("[CoreEngineBuilder] ✅ Faster-Whisper ASR initialized successfully");
        
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

    /// 使用 YourTTS HTTP 服务初始化 TTS 模块（支持零样本音色克隆）
    /// 
    /// # Arguments
    /// * `config` - YourTTS HTTP 配置
    pub fn tts_with_yourtts_http(mut self, config: YourTtsHttpConfig) -> EngineResult<Self> {
        // 保存服务 URL（用于健康检查）
        self.tts_service_url = Some(config.endpoint.clone());
        
        let tts_impl = YourTtsHttp::new(config)
            .map_err(|e| EngineError::new(format!("Failed to create YourTtsHttp: {}", e)))?;
        
        self.tts = Some(Arc::new(tts_impl));
        Ok(self)
    }
    
    /// 设置回退 TTS 服务（当主 TTS 不支持某些语言时使用）
    /// 
    /// # Arguments
    /// * `fallback_tts` - 回退 TTS 服务实例
    pub fn with_fallback_tts(mut self, fallback_tts: Arc<dyn TtsStreaming>) -> Self {
        self.fallback_tts = Some(fallback_tts);
        self
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
    
    /// 启用 TTS 增量播放
    /// 
    /// # Arguments
    /// * `enabled` - 是否启用增量播放
    /// * `buffer_sentences` - 缓冲的短句数量（0 = 立即播放，> 0 = 缓冲模式）
    /// * `max_sentence_length` - 最大句子长度（字符）
    pub fn with_tts_incremental_playback(
        mut self,
        enabled: bool,
        buffer_sentences: usize,
        max_sentence_length: usize,
    ) -> Self {
        if enabled {
            // 使用支持逗号分割的分段器（用于在逗号处添加停顿）
            let segmenter = TextSegmenter::new_with_comma_splitting(max_sentence_length);
            self.text_segmenter = Some(Arc::new(segmenter));
            self.tts_incremental_enabled = true;
            self.tts_buffer_sentences = buffer_sentences;
        }
        self
    }
    
    /// 启用 TTS 音频增强（fade in/out、停顿）
    /// 
    /// # Arguments
    /// * `config` - 音频增强配置
    pub fn with_audio_enhancement(mut self, config: AudioEnhancementConfig) -> Self {
        let enhancer = AudioEnhancer::new(config);
        self.audio_enhancer = Some(Arc::new(enhancer));
        self
    }
    
    /// 启用翻译质量检查
    /// 
    /// # Arguments
    /// * `enabled` - 是否启用质量检查
    pub fn with_translation_quality_check(mut self, enabled: bool) -> Self {
        if enabled {
            let checker = TranslationQualityChecker::new(true);
            self.quality_checker = Some(Arc::new(checker));
        }
        self
    }
    
    /// 启用连续输入输出模式
    /// 
    /// 在此模式下，系统会：
    /// - 使用音频缓冲管理器累积音频帧
    /// - 当 VAD 检测到边界时，异步处理当前片段
    /// - 在处理当前片段的同时，继续接收新的音频输入
    /// 
    /// # Arguments
    /// * `enabled` - 是否启用连续模式
    /// * `max_buffer_duration_ms` - 最大缓冲时长（毫秒），防止缓冲区溢出
    /// * `min_segment_duration_ms` - 最小片段时长（毫秒），防止过短片段
    pub fn with_continuous_mode(
        mut self,
        enabled: bool,
        max_buffer_duration_ms: u64,
        min_segment_duration_ms: u64,
    ) -> Self {
        if enabled {
            let buffer = AudioBufferManager::with_config(
                max_buffer_duration_ms,
                min_segment_duration_ms,
            );
            self.audio_buffer = Some(Arc::new(buffer));
            self.continuous_mode = true;
        }
        self
    }
    
    /// 启用 TTS 多说话者音色区分
    /// 
    /// 在此模式下，系统会为每个说话者分配不同的 TTS 音色（voice）
    /// 实现第二阶段目标：TTS 多说话者音色区分
    /// 
    /// # Arguments
    /// * `available_voices` - 可用的 voice 列表（例如：["zh_CN-huayan-medium", "zh_CN-xiaoyan-medium"]）
    pub fn with_speaker_voice_mapping(
        mut self,
        available_voices: Vec<String>,
    ) -> Self {
        if !available_voices.is_empty() {
            let mapper = SpeakerVoiceMapper::new(available_voices);
            self.speaker_voice_mapper = Some(Arc::new(mapper));
        }
        self
    }
    
    /// 启用说话者识别
    /// 
    /// 支持两种模式：
    /// - VadBased: 基于 VAD 边界的简单模式（免费用户）
    /// - EmbeddingBased: 基于 Speaker Embedding 的准确模式（付费用户）
    /// 
    /// # Arguments
    /// * `mode` - 说话者识别模式
    pub fn with_speaker_identification(
        mut self,
        mode: SpeakerIdentifierMode,
    ) -> EngineResult<Self> {
        let identifier: Arc<dyn SpeakerIdentifier> = match mode {
            SpeakerIdentifierMode::VadBased { min_switch_interval_ms, max_same_speaker_interval_ms } => {
                Arc::new(VadBasedSpeakerIdentifier::new(
                    min_switch_interval_ms,
                    max_same_speaker_interval_ms,
                ))
            }
            SpeakerIdentifierMode::EmbeddingBased { service_url, similarity_threshold, mode } => {
                Arc::new(EmbeddingBasedSpeakerIdentifier::new(
                    service_url,
                    similarity_threshold,
                    mode,
                )?)
            }
        };
        
        self.speaker_identifier = Some(identifier);
        Ok(self)
    }
    
    /// 直接设置已创建的说话者识别器（用于动态切换模式）
    pub fn with_speaker_identifier_custom(
        mut self,
        identifier: Arc<dyn SpeakerIdentifier>,
    ) -> Self {
        self.speaker_identifier = Some(identifier);
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
            fallback_tts: self.fallback_tts,
            config: self.config.ok_or_else(|| EngineError::new("config is missing"))?,
            cache: self.cache.ok_or_else(|| EngineError::new("cache is missing"))?,
            telemetry: self.telemetry.ok_or_else(|| EngineError::new("telemetry is missing"))?,
            post_processor: self.post_processor,
            perf_logger: self.perf_logger,
            text_segmenter: self.text_segmenter,
            audio_enhancer: self.audio_enhancer,
            quality_checker: self.quality_checker,
            nmt_service_url: self.nmt_service_url,
            tts_service_url: self.tts_service_url,
            tts_incremental_enabled: self.tts_incremental_enabled,
            tts_buffer_sentences: self.tts_buffer_sentences,
            audio_buffer: self.audio_buffer,
            continuous_mode: self.continuous_mode,
            speaker_voice_mapper: self.speaker_voice_mapper,
            speaker_identifier: self.speaker_identifier,
        })
    }
}
