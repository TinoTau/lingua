use std::sync::Arc;

use crate::asr_streaming::AsrStreaming;
use crate::audio_buffer::AudioBufferManager;
use crate::cache_manager::CacheManager;
use crate::config_manager::ConfigManager;
use crate::emotion_adapter::EmotionAdapter;
use crate::event_bus::EventBus;
use crate::nmt_incremental::NmtIncremental;
use crate::persona_adapter::PersonaAdapter;
use crate::performance_logger::PerformanceLogger;
use crate::post_processing::TextPostProcessor;
use crate::speaker_identifier::SpeakerIdentifier;
use crate::speaker_voice_mapper::SpeakerVoiceMapper;
use crate::telemetry::TelemetrySink;
use crate::text_segmentation::TextSegmenter;
use crate::translation_quality::TranslationQualityChecker;
use crate::tts_audio_enhancement::AudioEnhancer;
use crate::tts_streaming::TtsStreaming;
use crate::vad::VoiceActivityDetector;

pub struct CoreEngine {
    pub(crate) event_bus: Arc<dyn EventBus>,
    pub(crate) vad: Arc<dyn VoiceActivityDetector>,
    pub(crate) asr: Arc<dyn AsrStreaming>,
    pub(crate) nmt: Arc<dyn NmtIncremental>,
    pub(crate) emotion: Arc<dyn EmotionAdapter>,
    pub(crate) persona: Arc<dyn PersonaAdapter>,
    pub(crate) tts: Arc<dyn TtsStreaming>,
    pub(crate) fallback_tts: Option<Arc<dyn TtsStreaming>>,  // 回退 TTS 服务（当主 TTS 不支持该语言时使用）
    pub(crate) config: Arc<dyn ConfigManager>,
    pub(crate) cache: Arc<dyn CacheManager>,
    pub(crate) telemetry: Arc<dyn TelemetrySink>,
    // 优化模块
    pub(crate) post_processor: Option<Arc<TextPostProcessor>>,
    pub(crate) perf_logger: Option<Arc<PerformanceLogger>>,
    pub(crate) text_segmenter: Option<Arc<TextSegmenter>>,
    pub(crate) audio_enhancer: Option<Arc<AudioEnhancer>>,
    pub(crate) quality_checker: Option<Arc<TranslationQualityChecker>>,
    // 服务 URL（用于健康检查）
    pub(crate) nmt_service_url: Option<String>,
    pub(crate) tts_service_url: Option<String>,
    // TTS 增量播放配置
    pub(crate) tts_incremental_enabled: bool,
    pub(crate) tts_buffer_sentences: usize,
    // 连续输入输出支持
    pub(crate) audio_buffer: Option<Arc<AudioBufferManager>>,
    pub(crate) continuous_mode: bool,
    // TTS 多说话者音色区分
    pub(crate) speaker_voice_mapper: Option<Arc<SpeakerVoiceMapper>>,
    // 说话者识别
    pub(crate) speaker_identifier: Option<Arc<dyn SpeakerIdentifier>>,
}

impl Clone for CoreEngine {
    fn clone(&self) -> Self {
        Self {
            event_bus: Arc::clone(&self.event_bus),
            vad: Arc::clone(&self.vad),
            asr: Arc::clone(&self.asr),
            nmt: Arc::clone(&self.nmt),
            emotion: Arc::clone(&self.emotion),
            persona: Arc::clone(&self.persona),
            tts: Arc::clone(&self.tts),
            fallback_tts: self.fallback_tts.as_ref().map(Arc::clone),
            config: Arc::clone(&self.config),
            cache: Arc::clone(&self.cache),
            telemetry: Arc::clone(&self.telemetry),
            post_processor: self.post_processor.as_ref().map(Arc::clone),
            perf_logger: self.perf_logger.as_ref().map(Arc::clone),
            text_segmenter: self.text_segmenter.as_ref().map(Arc::clone),
            audio_enhancer: self.audio_enhancer.as_ref().map(Arc::clone),
            quality_checker: self.quality_checker.as_ref().map(Arc::clone),
            nmt_service_url: self.nmt_service_url.clone(),
            tts_service_url: self.tts_service_url.clone(),
            tts_incremental_enabled: self.tts_incremental_enabled,
            tts_buffer_sentences: self.tts_buffer_sentences,
            audio_buffer: self.audio_buffer.as_ref().map(Arc::clone),
            continuous_mode: self.continuous_mode,
            speaker_voice_mapper: self.speaker_voice_mapper.as_ref().map(Arc::clone),
            speaker_identifier: self.speaker_identifier.as_ref().map(Arc::clone),
        }
    }
}

