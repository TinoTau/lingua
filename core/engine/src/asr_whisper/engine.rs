// core/engine/src/asr_whisper/engine.rs
// Whisper ASR 推理引擎

use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use whisper_rs::{
    WhisperContext, WhisperContextParameters, 
    FullParams, SamplingStrategy,
};

use crate::asr_whisper::audio_preprocessing::{preprocess_audio_frame, accumulate_audio_frames};
use crate::types::AudioFrame;

/// Whisper ASR 推理引擎
pub struct WhisperAsrEngine {
    ctx: Arc<WhisperContext>,
    model_path: PathBuf,
    language: Option<String>,
}

impl WhisperAsrEngine {
    /// 从模型路径加载 Whisper 模型
    /// 
    /// # Arguments
    /// * `model_path` - GGML 模型文件路径
    /// 
    /// # Returns
    /// 返回 `WhisperAsrEngine` 实例
    pub fn new_from_model_path(model_path: &Path) -> Result<Self> {
        if !model_path.exists() {
            return Err(anyhow!("Model file not found: {}", model_path.display()));
        }

        let ctx = WhisperContext::new_with_params(
            model_path.to_str()
                .ok_or_else(|| anyhow!("Invalid model path: {}", model_path.display()))?,
            WhisperContextParameters::default(),
        )?;
        
        eprintln!("[ASR] Whisper context initialized (GPU support will be auto-detected at inference time)");

        Ok(Self {
            ctx: Arc::new(ctx),
            model_path: model_path.to_path_buf(),
            language: None,
        })
    }

    /// 从模型目录加载 Whisper 模型
    /// 
    /// # Arguments
    /// * `model_dir` - 模型目录路径（如 `models/asr/whisper-base/`）
    /// 
    /// # Returns
    /// 返回 `WhisperAsrEngine` 实例
    pub fn new_from_dir(model_dir: &Path) -> Result<Self> {
        // 尝试查找常见的模型文件名
        let possible_names = ["ggml-base.bin", "model.ggml", "ggml-model.bin"];
        
        for name in &possible_names {
            let model_path = model_dir.join(name);
            if model_path.exists() {
                return Self::new_from_model_path(&model_path);
            }
        }

        Err(anyhow!(
            "No Whisper model file found in directory: {}. Tried: {:?}",
            model_dir.display(),
            possible_names
        ))
    }

    /// 设置语言
    /// 
    /// # Arguments
    /// * `language` - 语言代码（如 "en", "zh"），`None` 表示自动检测
    pub fn set_language(&mut self, language: Option<String>) {
        self.language = language;
    }

    /// 获取当前语言设置
    pub fn get_language(&self) -> Option<String> {
        self.language.clone()
    }

    /// 对完整音频进行转录
    /// 
    /// # Arguments
    /// * `audio_data` - 预处理后的音频数据（16kHz 单声道 PCM f32）
    /// 
    /// # Returns
    /// 返回 (转录文本, 检测到的语言)
    pub fn transcribe_full(&self, audio_data: &[f32]) -> Result<(String, Option<String>)> {
        // 创建推理状态
        let mut state = self.ctx.create_state()
            .map_err(|e| anyhow!("Failed to create Whisper state: {:?}", e))?;

        // 配置推理参数
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // 设置语言
        if let Some(ref lang) = self.language {
            params.set_language(Some(lang.as_str()));
        }
        
        // 设置其他参数
        // 使用所有可用的 CPU 核心（留一个给系统）
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get().saturating_sub(1).max(1))
            .unwrap_or(4);
        params.set_n_threads(num_threads as i32);
        eprintln!("[ASR] Using {} CPU threads for inference", num_threads);
        params.set_translate(false);
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // 运行推理
        state.full(params, audio_data)
            .map_err(|e| anyhow!("Failed to run inference: {:?}", e))?;

        // 提取检测到的语言
        // Whisper 会在推理后设置检测到的语言
        let detected_lang = if self.language.is_none() {
            // 如果使用自动检测，尝试从 state 中获取检测到的语言
            // 注意：whisper_rs 可能不直接提供这个 API，我们需要从 segment 中推断
            // 或者使用其他方法
            None  // 暂时返回 None，后续可以从 segment 中提取
        } else {
            self.language.clone()
        };

        // 提取结果
        let num_segments = state.full_n_segments();
        let mut full_text = String::new();

        for i in 0..num_segments {
            if let Some(segment) = state.get_segment(i) {
                // 从 Debug 输出中提取文本（因为字段可能是私有的）
                let segment_debug = format!("{:?}", segment);
                
                if let Some(start_idx) = segment_debug.find("text: Ok(\"") {
                    let text_start = start_idx + 10;
                    if let Some(end_idx) = segment_debug[text_start..].find("\")") {
                        let text = &segment_debug[text_start..text_start + end_idx];
                        let text_trimmed = text.trim();
                        if !text_trimmed.is_empty() {
                            full_text.push_str(text_trimmed);
                            full_text.push(' ');
                        }
                    }
                }
            }
        }

        Ok((full_text.trim().to_string(), detected_lang))
    }

    /// 从 AudioFrame 转录
    /// 
    /// # Arguments
    /// * `frame` - 音频帧
    /// 
    /// # Returns
    /// 返回 (转录文本, 检测到的语言)
    pub fn transcribe_frame(&self, frame: &AudioFrame) -> Result<(String, Option<String>)> {
        let audio_data = preprocess_audio_frame(frame)?;
        self.transcribe_full(&audio_data)
    }

    /// 从多个 AudioFrame 累积并转录
    /// 
    /// # Arguments
    /// * `frames` - 音频帧序列
    /// 
    /// # Returns
    /// 返回 (转录文本, 检测到的语言)
    pub fn transcribe_frames(&self, frames: &[AudioFrame]) -> Result<(String, Option<String>)> {
        let audio_data = accumulate_audio_frames(frames)?;
        self.transcribe_full(&audio_data)
    }

    /// 获取模型路径
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    /// 获取当前设置的语言
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_whisper_engine_load() {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");

        if !model_path.exists() {
            println!("⚠ 跳过测试: 模型文件不存在");
            return;
        }

        let engine = WhisperAsrEngine::new_from_model_path(&model_path)
            .expect("Failed to load Whisper engine");

        println!("✓ WhisperAsrEngine 加载成功");
        println!("  模型路径: {}", engine.model_path().display());
    }

    #[test]
    fn test_whisper_engine_from_dir() {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let model_dir = crate_root.join("models/asr/whisper-base");

        if !model_dir.exists() {
            println!("⚠ 跳过测试: 模型目录不存在");
            return;
        }

        let engine = WhisperAsrEngine::new_from_dir(&model_dir)
            .expect("Failed to load Whisper engine from directory");

        println!("✓ WhisperAsrEngine 从目录加载成功");
        println!("  模型路径: {}", engine.model_path().display());
    }

    #[test]
    fn test_whisper_engine_transcribe() {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let project_root = crate_root
            .parent()
            .and_then(|p| p.parent())
            .expect("failed to resolve project root");

        let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");
        let wav_path = project_root.join("third_party/whisper.cpp/samples/jfk.wav");

        if !model_path.exists() || !wav_path.exists() {
            println!("⚠ 跳过测试: 模型或音频文件不存在");
            return;
        }

        // 加载引擎
        let mut engine = WhisperAsrEngine::new_from_model_path(&model_path)
            .expect("Failed to load Whisper engine");

        engine.set_language(Some("en".to_string()));

        // 加载音频
        let mut reader = hound::WavReader::open(&wav_path)
            .expect("Failed to open WAV file");
        let spec = reader.spec();

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

        let audio_data: Vec<f32> = if spec.channels == 2 {
            samples.chunks(2)
                .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
                .collect()
        } else {
            samples
        };

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

        // 进行转录
        println!("\n开始转录...");
        let (result, _detected_lang) = engine.transcribe_full(&audio_16k)
            .expect("Failed to transcribe");

        println!("\n转录结果:");
        println!("{}", result);

        // 验证结果
        let result_lower = result.to_lowercase();
        assert!(result_lower.contains("ask not what your country can do for you") ||
                result_lower.contains("what you can do for your country"),
                "转录结果应该包含 JFK 演讲的关键内容");
    }
}

