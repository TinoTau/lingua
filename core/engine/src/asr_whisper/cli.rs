// core/engine/src/asr_whisper/cli.rs

use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, bail};

use crate::asr_whisper::{AsrEngine, AsrFinal};

/// Whisper CLI 调用的配置
#[derive(Debug, Clone)]
pub struct WhisperCliConfig {
    /// whisper-cli.exe 相对于项目根目录的路径
    pub exe_path: String,
    /// ggml 模型相对于项目根目录的路径
    pub model_path: String,
}

impl Default for WhisperCliConfig {
    fn default() -> Self {
        Self {
            exe_path: "third_party/whisper.cpp/build/bin/whisper-cli.exe".to_string(),
            model_path: "third_party/whisper.cpp/models/ggml-base.en.bin".to_string(),
        }
    }
}

/// 基于 whisper-cli.exe 的 ASR 引擎实现
pub struct WhisperCliEngine {
    pub cfg: WhisperCliConfig,
}

impl WhisperCliEngine {
    pub fn new(cfg: WhisperCliConfig) -> Self {
        Self { cfg }
    }

    fn project_root() -> PathBuf {
        static mut CACHE: Option<PathBuf> = None;
        unsafe {
            if let Some(ref cached) = CACHE {
                return cached.clone();
            }
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let project_root = manifest_dir
                .parent() // core/
                .and_then(Path::parent) // workspace root
                .map(Path::to_path_buf)
                .unwrap_or_else(|| manifest_dir.clone());
            CACHE = Some(project_root.clone());
            project_root
        }
    }

    fn resolve_path(path: &str) -> PathBuf {
        let p = PathBuf::from(path);
        if p.is_absolute() {
            p
        } else {
            Self::project_root().join(p)
        }
    }

    fn run_cli(&self, wav_path: &Path) -> Result<String> {
        let exe_path = Self::resolve_path(&self.cfg.exe_path);
        let model_path = Self::resolve_path(&self.cfg.model_path);
        let wav_path = if wav_path.is_relative() {
            Self::project_root().join(wav_path)
        } else {
            wav_path.to_path_buf()
        };

        if !exe_path.exists() {
            bail!("whisper-cli.exe not found at: {}", exe_path.display());
        }
        if !model_path.exists() {
            bail!("whisper model not found at: {}", model_path.display());
        }
        if !wav_path.exists() {
            bail!("wav file not found: {}", wav_path.display());
        }

        let output = Command::new(&exe_path)
            .args([
                "-m",
                &model_path.to_string_lossy(),
                "-f",
                &wav_path.to_string_lossy(),
                // 你也可以加更多参数，如指定语言、禁用进度条等
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("whisper-cli failed: {}", stderr);
        }

        // whisper-cli 默认会把结果写到 .srt / .vtt / .txt 文件里
        // 这里为了简单，我们先直接用 stdout（新版本会输出一份带时间戳的文本）
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}

impl AsrEngine for WhisperCliEngine {
    fn transcribe_wav_file(&self, wav_path: &Path) -> Result<AsrFinal> {
        let raw = self.run_cli(wav_path)?;

        // 最简单版本：直接把 stdout 当成最终文本
        // 后面你可以解析 [00:00:00.000 --> ...] 这样的行，拼接出纯文本
        Ok(AsrFinal { text: raw })
    }
}
