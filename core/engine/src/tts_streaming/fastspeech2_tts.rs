use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use ort::session::Session;
use ort::{SessionBuilder, Environment};
use ort::tensor::OrtOwnedTensor;
use ort::value::Value;
use std::sync::Arc;
use async_trait::async_trait;
use ndarray::{Array1, Array2, Array3, IxDyn, Ix2, Ix3};
use std::ptr;
use ndarray::CowArray;

use crate::error::{EngineError, EngineResult};
use super::{TtsRequest, TtsStreamChunk, TtsStreaming};
use super::text_processor::TextProcessor;

/// FastSpeech2 + HiFiGAN TTS 引擎
pub struct FastSpeech2TtsEngine {
    /// FastSpeech2 模型会话（中文）
    fastspeech2_zh_session: Mutex<Session>,
    /// FastSpeech2 模型会话（英文）
    fastspeech2_en_session: Mutex<Session>,
    /// HiFiGAN 模型会话（中文）
    hifigan_zh_session: Mutex<Session>,
    /// HiFiGAN 模型会话（英文）
    hifigan_en_session: Mutex<Session>,
    /// 文本预处理器（中文）
    text_processor_zh: TextProcessor,
    /// 文本预处理器（英文）
    text_processor_en: TextProcessor,
    /// 模型目录路径
    model_dir: PathBuf,
}

impl FastSpeech2TtsEngine {
    /// 从模型目录加载 TTS 引擎
    pub fn new_from_dir(model_dir: &Path) -> Result<Self> {
        // 初始化 ONNX Runtime
        crate::onnx_utils::init_onnx_runtime()?;

        let model_dir = model_dir.to_path_buf();
        let fastspeech2_dir = model_dir.join("fastspeech2-lite");
        let hifigan_dir = model_dir.join("hifigan-lite");

        // 创建 ONNX Runtime 环境
        let env = Arc::new(
            Environment::builder()
                .with_name("fastspeech2_tts")
                .build()?
        );

        // 加载 FastSpeech2 模型（中文）
        let fastspeech2_zh_path = fastspeech2_dir.join("fastspeech2_csmsc_streaming.onnx");
        if !fastspeech2_zh_path.exists() {
            return Err(anyhow!("FastSpeech2 Chinese model not found at {}", fastspeech2_zh_path.display()));
        }
        let fastspeech2_zh_session = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create FastSpeech2 ZH Session builder: {e}"))?
            .with_model_from_file(&fastspeech2_zh_path)
            .map_err(|e| anyhow!("failed to load FastSpeech2 ZH model: {e}"))?;

        // 加载 FastSpeech2 模型（英文）
        let fastspeech2_en_path = fastspeech2_dir.join("fastspeech2_ljspeech.onnx");
        if !fastspeech2_en_path.exists() {
            return Err(anyhow!("FastSpeech2 English model not found at {}", fastspeech2_en_path.display()));
        }
        let fastspeech2_en_session = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create FastSpeech2 EN Session builder: {e}"))?
            .with_model_from_file(&fastspeech2_en_path)
            .map_err(|e| anyhow!("failed to load FastSpeech2 EN model: {e}"))?;

        // 加载 HiFiGAN 模型（中文）
        let hifigan_zh_path = hifigan_dir.join("hifigan_csmsc.onnx");
        if !hifigan_zh_path.exists() {
            return Err(anyhow!("HiFiGAN Chinese model not found at {}", hifigan_zh_path.display()));
        }
        let hifigan_zh_session = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create HiFiGAN ZH Session builder: {e}"))?
            .with_model_from_file(&hifigan_zh_path)
            .map_err(|e| anyhow!("failed to load HiFiGAN ZH model: {e}"))?;

        // 加载 HiFiGAN 模型（英文）
        let hifigan_en_path = hifigan_dir.join("hifigan_ljspeech.onnx");
        if !hifigan_en_path.exists() {
            return Err(anyhow!("HiFiGAN English model not found at {}", hifigan_en_path.display()));
        }
        let hifigan_en_session = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create HiFiGAN EN Session builder: {e}"))?
            .with_model_from_file(&hifigan_en_path)
            .map_err(|e| anyhow!("failed to load HiFiGAN EN model: {e}"))?;

        // 加载文本预处理器
        let text_processor_zh = TextProcessor::new_from_dir(&model_dir, "zh")?;
        let text_processor_en = TextProcessor::new_from_dir(&model_dir, "en")?;

        Ok(Self {
            fastspeech2_zh_session: Mutex::new(fastspeech2_zh_session),
            fastspeech2_en_session: Mutex::new(fastspeech2_en_session),
            hifigan_zh_session: Mutex::new(hifigan_zh_session),
            hifigan_en_session: Mutex::new(hifigan_en_session),
            text_processor_zh,
            text_processor_en,
            model_dir,
        })
    }

    /// 根据 locale 选择对应的模型会话
    fn get_fastspeech2_session(&self, locale: &str) -> Result<&Mutex<Session>> {
        match locale {
            "zh" | "chinese" | "zh-CN" => Ok(&self.fastspeech2_zh_session),
            "en" | "english" | "en-US" => Ok(&self.fastspeech2_en_session),
            _ => Err(anyhow!("Unsupported locale: {}", locale)),
        }
    }

    fn get_hifigan_session(&self, locale: &str) -> Result<&Mutex<Session>> {
        match locale {
            "zh" | "chinese" | "zh-CN" => Ok(&self.hifigan_zh_session),
            "en" | "english" | "en-US" => Ok(&self.hifigan_en_session),
            _ => Err(anyhow!("Unsupported locale: {}", locale)),
        }
    }

    fn get_text_processor(&self, locale: &str) -> Result<&TextProcessor> {
        match locale {
            "zh" | "chinese" | "zh-CN" => Ok(&self.text_processor_zh),
            "en" | "english" | "en-US" => Ok(&self.text_processor_en),
            _ => Err(anyhow!("Unsupported locale: {}", locale)),
        }
    }

    /// 运行 FastSpeech2 推理：音素 ID → Mel-spectrogram
    fn run_fastspeech2(
        &self,
        phone_ids: &[i64],
        locale: &str,
    ) -> Result<Array3<f32>> {
        let session_guard = self.get_fastspeech2_session(locale)?.lock().unwrap();
        
        // 准备输入：音素 ID 序列
        // FastSpeech2 模型输入通常是 [batch, seq_len] 或 [batch, seq_len, dim]
        // 根据之前检查，模型输入是 xs: [1, seq_len, 384]
        // 但实际 ONNX 模型可能接受 [1, seq_len] 的整数 ID，内部会做 embedding
        // 先尝试 [1, seq_len] 形状
        
        let batch_size = 1usize;
        let seq_len = phone_ids.len();
        
        if seq_len == 0 {
            return Err(anyhow!("phone_ids is empty"));
        }

        // 创建输入张量 [1, seq_len]
        // 注意：如果模型需要 [1, seq_len, 384]，需要在模型内部或预处理阶段做 embedding
        // 这里假设模型接受整数 ID 输入，内部会处理 embedding
        let input_array: Array2<i64> = Array2::from_shape_vec(
            (batch_size, seq_len),
            phone_ids.to_vec(),
        )?;

        // 转换为 ONNX Value
        let arr_dyn = input_array.into_dyn();
        let arr_owned = arr_dyn.to_owned();
        let cow_arr = CowArray::from(arr_owned);
        let input_value = Value::from_array(ptr::null_mut(), &cow_arr)
            .map_err(|e| anyhow!("failed to convert input to Value: {:?}", e))?;
        let input_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(input_value)
        };

        // 运行推理
        let inputs = vec![input_value];
        let outputs: Vec<Value> = session_guard.run(inputs)
            .map_err(|e| anyhow!("failed to run FastSpeech2: {e}"))?;

        // 提取 mel-spectrogram 输出
        let mel_value = outputs.get(0)
            .ok_or_else(|| anyhow!("FastSpeech2 output is empty"))?;

        // 转换为 ndarray
        let tensor: OrtOwnedTensor<f32, IxDyn> = mel_value.try_extract()
            .map_err(|e| anyhow!("failed to extract mel-spectrogram tensor: {e}"))?;
        let view = tensor.view();
        let mel: Array3<f32> = view
            .to_owned()
            .into_dimensionality::<Ix3>()
            .map_err(|e| anyhow!("failed to reshape mel-spectrogram: {e}"))?;

        // mel 形状应该是 [batch, mel_dim, time_steps] = [1, 80, time_steps]
        // 或者可能是 [batch, time_steps, mel_dim] = [1, time_steps, 80]
        // 需要根据实际模型输出调整
        
        // 如果形状是 [1, time_steps, 80]，需要转置为 [1, 80, time_steps]
        let mel_shape = mel.shape();
        if mel_shape.len() == 3 {
            let (b, dim1, dim2) = (mel_shape[0], mel_shape[1], mel_shape[2]);
            if dim1 == 80 && dim2 > 80 {
                // 形状是 [1, 80, time_steps]，正确
                Ok(mel)
            } else if dim1 > 80 && dim2 == 80 {
                // 形状是 [1, time_steps, 80]，需要转置
                // 使用 swap_axes 转置维度 1 和 2
                let mel_transposed = mel.swap_axes(1, 2);
                Ok(mel_transposed)
            } else {
                // 未知形状，直接返回（可能需要根据实际模型调整）
                println!("[WARN] Unexpected mel-spectrogram shape: {:?}", mel_shape);
                Ok(mel)
            }
        } else {
            Ok(mel)
        }
    }

    /// 运行 HiFiGAN 推理：Mel-spectrogram → 音频波形
    fn run_hifigan(
        &self,
        mel: &Array3<f32>,
        locale: &str,
    ) -> Result<Array1<f32>> {
        let session_guard = self.get_hifigan_session(locale)?.lock().unwrap();

        // 准备输入：mel-spectrogram [batch, mel_dim, time_steps]
        let arr_dyn = mel.clone().into_dyn();
        let arr_owned = arr_dyn.to_owned();
        let cow_arr = CowArray::from(arr_owned);
        let input_value = Value::from_array(ptr::null_mut(), &cow_arr)
            .map_err(|e| anyhow!("failed to convert mel to Value: {:?}", e))?;
        let input_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(input_value)
        };

        // 运行推理
        let inputs = vec![input_value];
        let outputs: Vec<Value> = session_guard.run(inputs)
            .map_err(|e| anyhow!("failed to run HiFiGAN: {e}"))?;

        // 提取音频波形输出
        let audio_value = outputs.get(0)
            .ok_or_else(|| anyhow!("HiFiGAN output is empty"))?;

        // 转换为 ndarray
        let tensor: OrtOwnedTensor<f32, IxDyn> = audio_value.try_extract()
            .map_err(|e| anyhow!("failed to extract audio tensor: {e}"))?;
        let view = tensor.view();
        
        // 音频形状可能是 [batch, samples] 或 [samples]
        let audio: Array1<f32> = if view.ndim() == 2 {
            // [batch, samples] -> 取第一行
            let audio_2d: Array2<f32> = view
                .to_owned()
                .into_dimensionality::<Ix2>()
                .map_err(|e| anyhow!("failed to reshape audio to 2D: {e}"))?;
            audio_2d.row(0).to_owned()
        } else {
            // [samples]
            view.to_owned()
                .into_dimensionality::<ndarray::Ix1>()
                .map_err(|e| anyhow!("failed to reshape audio to 1D: {e}"))?
        };

        Ok(audio)
    }

    /// 将 f32 音频波形转换为 PCM 16-bit 字节
    /// 
    /// 输入：f32 音频波形（范围通常是 [-1.0, 1.0]）
    /// 输出：PCM 16-bit 小端字节序字节序列
    fn audio_to_pcm16(&self, audio: &Array1<f32>) -> Vec<u8> {
        if audio.is_empty() {
            return vec![];
        }
        
        let mut pcm_bytes = Vec::with_capacity(audio.len() * 2);
        
        for &sample in audio.iter() {
            // 限制范围到 [-1.0, 1.0]
            let clamped = sample.max(-1.0).min(1.0);
            
            // 转换为 16-bit 整数（小端字节序）
            // 范围：[-32768, 32767]
            let sample_i16 = (clamped * 32767.0).round() as i16;
            let bytes = sample_i16.to_le_bytes();
            pcm_bytes.extend_from_slice(&bytes);
        }
        
        pcm_bytes
    }

    /// 将音频分割为多个 chunk（用于流式输出）
    /// 
    /// # Arguments
    /// * `audio` - PCM 音频字节
    /// * `chunk_size_samples` - 每个 chunk 的样本数（默认 2048，约 128ms @ 16kHz）
    /// * `sample_rate` - 采样率（默认 16000 Hz）
    /// 
    /// # Returns
    /// 返回 chunk 列表，每个 chunk 包含音频数据和元数据
    fn split_audio_to_chunks(
        &self,
        audio: &[u8],
        chunk_size_samples: usize,
        sample_rate: u32,
    ) -> Vec<(Vec<u8>, u64, bool)> {
        // PCM 16-bit = 2 字节/样本
        let bytes_per_sample = 2;
        let chunk_size_bytes = chunk_size_samples * bytes_per_sample;
        
        let mut chunks = Vec::new();
        let mut offset = 0;
        let mut chunk_index = 0;
        
        while offset < audio.len() {
            let chunk_end = (offset + chunk_size_bytes).min(audio.len());
            let chunk_audio = audio[offset..chunk_end].to_vec();
            
            // 计算时间戳（毫秒）
            let samples_so_far = offset / bytes_per_sample;
            let timestamp_ms = (samples_so_far as u64 * 1000) / sample_rate as u64;
            
            // 判断是否是最后一个 chunk
            let is_last = chunk_end >= audio.len();
            
            chunks.push((chunk_audio, timestamp_ms, is_last));
            
            offset = chunk_end;
            chunk_index += 1;
        }
        
        // 如果没有 chunk，至少返回一个空 chunk
        if chunks.is_empty() {
            chunks.push((vec![], 0, true));
        }
        
        chunks
    }
}

#[async_trait]
impl TtsStreaming for FastSpeech2TtsEngine {
    async fn synthesize(&self, request: TtsRequest) -> EngineResult<TtsStreamChunk> {
        // 输入验证
        if request.text.trim().is_empty() {
            return Ok(TtsStreamChunk {
                audio: vec![],
                timestamp_ms: 0,
                is_last: true,
            });
        }

        // 1. 文本预处理：文本 → 音素 ID
        let text_processor = self.get_text_processor(&request.locale)
            .map_err(|e| EngineError::new(format!("unsupported locale: {e}")))?;
        
        let phone_ids = text_processor.text_to_phone_ids(&request.text)
            .map_err(|e| EngineError::new(format!("text preprocessing failed: {e}")))?;

        if phone_ids.is_empty() {
            return Ok(TtsStreamChunk {
                audio: vec![],
                timestamp_ms: 0,
                is_last: true,
            });
        }

        // 2. FastSpeech2 推理：音素 ID → Mel-spectrogram
        let mel = self.run_fastspeech2(&phone_ids, &request.locale)
            .map_err(|e| EngineError::new(format!("FastSpeech2 inference failed: {e}")))?;

        // 验证 mel-spectrogram 形状
        let mel_shape = mel.shape();
        if mel_shape.len() != 3 || mel_shape[0] != 1 {
            return Err(EngineError::new(format!(
                "Invalid mel-spectrogram shape: {:?}, expected [1, mel_dim, time_steps]",
                mel_shape
            )));
        }

        // 3. HiFiGAN 推理：Mel-spectrogram → 音频波形
        let audio_waveform = self.run_hifigan(&mel, &request.locale)
            .map_err(|e| EngineError::new(format!("HiFiGAN inference failed: {e}")))?;

        if audio_waveform.is_empty() {
            return Err(EngineError::new("HiFiGAN produced empty audio"));
        }

        // 4. 转换为 PCM 16-bit 字节
        let pcm_audio = self.audio_to_pcm16(&audio_waveform);

        if pcm_audio.is_empty() {
            return Err(EngineError::new("PCM conversion produced empty audio"));
        }

        // 5. 创建 chunk（当前实现：一次性返回完整音频）
        // 未来可以实现真正的流式 chunk 分割
        // 当前：返回完整音频作为单个 chunk
        Ok(TtsStreamChunk {
            audio: pcm_audio,
            timestamp_ms: 0,  // 时间戳从 0 开始
            is_last: true,    // 单个 chunk，标记为最后一个
        })
    }

    async fn close(&self) -> EngineResult<()> {
        Ok(())
    }
}

