use std::path::PathBuf;
use std::sync::Arc;

use crate::asr_streaming::AsrStreaming;
use crate::asr_whisper::WhisperAsrStreaming;
use crate::cache_manager::CacheManager;
use crate::config_manager::ConfigManager;
use crate::emotion_adapter::EmotionAdapter;
use crate::error::{EngineError, EngineResult};
use crate::event_bus::EventBus;
use crate::nmt_incremental::{NmtIncremental, MarianNmtOnnx};
use crate::persona_adapter::PersonaAdapter;
use crate::telemetry::{TelemetryDatum, TelemetrySink};
use crate::tts_streaming::TtsStreaming;
use crate::vad::VoiceActivityDetector;


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
        })
    }
}

impl CoreEngine {
    pub async fn boot(&self) -> EngineResult<()> {
        self.event_bus.start().await?;
        let config = self.config.load().await?;
        self.cache.warm_up().await?;
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
}
