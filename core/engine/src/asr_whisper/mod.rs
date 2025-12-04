// core/engine/src/asr_whisper/mod.rs

pub mod cli;
pub mod audio_preprocessing;
pub mod engine;
pub mod streaming;
pub mod faster_whisper_streaming;

// 以后你可以在这里定义统一的 trait，比如：
#[derive(Debug, Clone)]
pub struct AsrFinal {
    pub text: String,
}

pub trait AsrEngine {
    fn transcribe_wav_file(&self, wav_path: &std::path::Path) -> anyhow::Result<AsrFinal>;
}

// 导出音频预处理函数
pub use audio_preprocessing::{
    preprocess_audio_frame,
    accumulate_audio_frames,
    resample_audio,
    WHISPER_SAMPLE_RATE,
    WHISPER_N_MEL,
};

// 导出推理引擎
pub use engine::WhisperAsrEngine;

// 导出流式实现
pub use streaming::WhisperAsrStreaming;
pub use faster_whisper_streaming::FasterWhisperAsrStreaming;
