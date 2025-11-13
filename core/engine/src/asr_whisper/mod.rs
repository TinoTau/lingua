// core/engine/src/asr_whisper/mod.rs

pub mod cli;

// 以后你可以在这里定义统一的 trait，比如：
#[derive(Debug, Clone)]
pub struct AsrFinal {
    pub text: String,
}

pub trait AsrEngine {
    fn transcribe_wav_file(&self, wav_path: &std::path::Path) -> anyhow::Result<AsrFinal>;
}
