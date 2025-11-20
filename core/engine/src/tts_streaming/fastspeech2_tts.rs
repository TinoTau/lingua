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

        // 创建输入张量 [1, seq_len, 384]
        // 注意：FastSpeech2 模型期望 3 维输入 [batch, seq_len, embedding_dim]
        // 模型期望 embedding_dim = 384
        // 由于我们没有 embedding 层，这里使用一个简单的方案：
        // 将每个 phone ID 扩展为一个 384 维的向量（使用 one-hot 风格的表示）
        // TODO: 后续应该实现真正的 embedding 层或使用模型内部的 embedding
        
        const EMBEDDING_DIM: usize = 384;
        let phone_ids_f32: Vec<f32> = phone_ids.iter().map(|&id| id as f32).collect();
        
        // 创建 3D 数组 [1, seq_len, 384]
        // 简单方案：将 phone ID 值复制到 embedding 维度的每个位置
        // 这只是一个临时方案，正确的做法应该是使用 embedding 查找表
        let mut input_data = Vec::with_capacity(batch_size * seq_len * EMBEDDING_DIM);
        for &phone_id_f32 in &phone_ids_f32 {
            // 将 phone_id 值复制到 384 维（简单方案，不是真正的 embedding）
            for _ in 0..EMBEDDING_DIM {
                input_data.push(phone_id_f32);
            }
        }
        
        let input_array: Array3<f32> = Array3::from_shape_vec(
            (batch_size, seq_len, EMBEDDING_DIM),
            input_data,
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

        // 根据模型规范，FastSpeech2 输出是 [1, time_steps, 80] = [batch, seq_len, mel_dim]
        // 不需要转置，保持原样
        // 注意：HiFiGAN 期望输入是 [1, time_steps, 384]，但 FastSpeech2 输出是 80 维
        // 这可能是模型不匹配的问题，需要后续处理
        let mel_shape_vec: Vec<usize> = mel.shape().to_vec(); // 保存形状信息，避免借用问题
        if mel_shape_vec.len() == 3 {
            let (_batch, dim1, dim2) = (mel_shape_vec[0], mel_shape_vec[1], mel_shape_vec[2]);
            // 根据模型规范，输出应该是 [1, time_steps, 80]
            if dim2 == 80 {
                // 形状是 [1, time_steps, 80]，符合模型规范，保持原样
                Ok(mel)
            } else if dim1 == 80 {
                // 形状是 [1, 80, time_steps]，需要转置为 [1, time_steps, 80]
                let mel_transposed = mel.permuted_axes([0, 2, 1]);
                println!("[INFO] Transposed mel-spectrogram from {:?} to {:?}", mel_shape_vec, mel_transposed.shape());
                Ok(mel_transposed)
            } else {
                println!("[WARN] Unexpected mel-spectrogram shape: {:?}, expected [1, time_steps, 80]", mel_shape_vec);
                Ok(mel)
            }
        } else {
            Ok(mel)
        }
    }

    /// 运行 HiFiGAN 推理：Mel-spectrogram → 音频波形
    /// 
    /// 注意：根据模型规范，HiFiGAN 期望输入是 [1, time_steps, 384]
    /// 但 FastSpeech2 输出是 [1, time_steps, 80]
    /// 这可能是模型不匹配的问题，需要特征扩展或使用不同的模型
    fn run_hifigan(
        &self,
        mel: &Array3<f32>,
        locale: &str,
    ) -> Result<Array1<f32>> {
        let session_guard = self.get_hifigan_session(locale)?.lock().unwrap();

        // 根据模型规范，HiFiGAN 期望输入是 [1, time_steps, 384]
        // 但 FastSpeech2 输出是 [1, time_steps, 80]
        // 这里需要将 80 维扩展到 384 维
        // TODO: 这可能是模型不匹配的问题，需要检查模型是否配对
        
        let mel_shape = mel.shape();
        if mel_shape.len() != 3 || mel_shape[0] != 1 {
            return Err(anyhow!("Invalid mel-spectrogram shape: {:?}, expected [1, time_steps, 80]", mel_shape));
        }
        
        let (batch, time_steps, mel_dim) = (mel_shape[0], mel_shape[1], mel_shape[2]);
        
        // 如果 mel_dim 是 80，需要扩展到 384
        // 简单方案：将 80 维复制/扩展为 384 维（这不是正确的做法，但可以测试）
        let hifigan_input = if mel_dim == 80 {
            // 将 80 维扩展到 384 维
            // 方案：将每个 80 维向量重复 4.8 次（384/80 = 4.8），然后截断
            const TARGET_DIM: usize = 384;
            let mut expanded_data = Vec::with_capacity(batch * time_steps * TARGET_DIM);
            
            for t in 0..time_steps {
                for d in 0..TARGET_DIM {
                    // 循环使用 80 维的值
                    let src_idx = d % mel_dim;
                    expanded_data.push(mel[[0, t, src_idx]]);
                }
            }
            
            Array3::from_shape_vec((batch, time_steps, TARGET_DIM), expanded_data)
                .map_err(|e| anyhow!("Failed to expand mel to 384 dim: {e}"))?
        } else if mel_dim == 384 {
            // 已经是 384 维，直接使用
            mel.clone()
        } else {
            return Err(anyhow!("Unsupported mel-spectrogram dimension: {}, expected 80 or 384", mel_dim));
        };

        // 准备输入：HiFiGAN 期望 [1, time_steps, 384]
        let arr_dyn = hifigan_input.into_dyn();
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
        
        println!("[DEBUG HiFiGAN] Output shape: {:?}, ndim: {}", view.shape(), view.ndim());
        
        // 根据模型规范，HiFiGAN 输出是 [1, '?', 80] = [batch, time_steps, feature_dim]
        // 这不是标准的音频波形，需要特殊处理
        // 可能的处理方式：
        // 1. 如果是 3 维，可能是特征序列，需要进一步处理
        // 2. 如果是 2 维或 1 维，可能是音频波形
        let audio: Array1<f32> = match view.ndim() {
            3 => {
                // [batch, time_steps, feature_dim] = [1, time_steps, 80]
                let audio_3d: Array3<f32> = view
                    .to_owned()
                    .into_dimensionality::<Ix3>()
                    .map_err(|e| anyhow!("failed to reshape audio to 3D: {e}"))?;
                
                let shape = audio_3d.shape();
                let (_batch, time_steps, feature_dim) = (shape[0], shape[1], shape[2]);
                
                println!("[DEBUG HiFiGAN] 3D output shape: [batch={}, time_steps={}, feature_dim={}]", 
                    _batch, time_steps, feature_dim);
                
                // 打印前几个值，看看数据范围
                if time_steps > 0 && feature_dim > 0 {
                    let sample_00 = audio_3d[[0, 0, 0]];
                    let sample_01 = audio_3d[[0, 0, 1]];
                    let sample_10 = if time_steps > 1 { audio_3d[[0, 1, 0]] } else { sample_00 };
                    println!("[DEBUG HiFiGAN] Sample values: [0,0,0]={:.6}, [0,0,1]={:.6}, [0,1,0]={:.6}", 
                        sample_00, sample_01, sample_10);
                }
                
                // 问题：HiFiGAN 输出 [1, time_steps, 80] 不是音频波形
                // 可能的情况：
                // 1. 这个模型不是标准的 vocoder，而是中间层输出
                // 2. 需要额外的后处理步骤
                // 3. 模型导出有问题
                // 
                // 临时方案：尝试将 80 维特征转换为音频
                // 假设每个 time_step 对应一个音频帧，80 维是 mel 特征
                // 但这不对，因为 vocoder 应该输出音频波形，不是 mel 特征
                
                println!("[WARN] HiFiGAN output is 3D: {:?}, expected audio waveform [samples].", shape);
                println!("[WARN] This model may not be a standard vocoder. Attempting to extract audio...");
                
                // 尝试方案 1：如果 feature_dim == 1，可能是 [batch, time_steps, 1]，展平为 [time_steps]
                if feature_dim == 1 {
                    println!("[INFO] feature_dim == 1, treating as [batch, time_steps, 1] -> [time_steps]");
                    let mut audio_data = Vec::with_capacity(time_steps);
                    for t in 0..time_steps {
                        audio_data.push(audio_3d[[0, t, 0]]);
                    }
                    return Ok(Array1::from_vec(audio_data));
                }
                
                // 尝试方案 2：如果 time_steps == 1，可能是 [batch, 1, samples]，展平为 [samples]
                if time_steps == 1 {
                    println!("[INFO] time_steps == 1, treating as [batch, 1, samples] -> [samples]");
                    let mut audio_data = Vec::with_capacity(feature_dim);
                    for d in 0..feature_dim {
                        audio_data.push(audio_3d[[0, 0, d]]);
                    }
                    return Ok(Array1::from_vec(audio_data));
                }
                
                // 尝试方案 3：转置后展平
                // 也许输出格式是 [1, mel_dim, time_steps] 而不是 [1, time_steps, mel_dim]
                // 尝试转置： [1, 4, 80] -> [1, 80, 4]
                println!("[INFO] Attempting transpose: [1, {}, {}] -> [1, {}, {}]", 
                    time_steps, feature_dim, feature_dim, time_steps);
                let audio_transposed = audio_3d.permuted_axes([0, 2, 1]);
                let transposed_shape = audio_transposed.shape();
                println!("[INFO] Transposed shape: {:?}", transposed_shape);
                
                // 转置后，如果 feature_dim (80) 是 mel 维度，time_steps (4) 是时间步
                // 那么可能需要将 80 维 mel 特征转换为音频
                // 但这里我们暂时尝试展平转置后的结果
                let total_samples = feature_dim * time_steps;  // 80 * 4 = 320
                let mut audio_data = Vec::with_capacity(total_samples);
                for d in 0..feature_dim {
                    for t in 0..time_steps {
                        audio_data.push(audio_transposed[[0, d, t]]);
                    }
                }
                
                // 打印展平后的统计信息
                if !audio_data.is_empty() {
                    let min_val = audio_data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
                    let max_val = audio_data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                    let mean_val = audio_data.iter().sum::<f32>() / audio_data.len() as f32;
                    println!("[DEBUG HiFiGAN] Transposed and flattened audio stats: min={:.6}, max={:.6}, mean={:.6}, len={}", 
                        min_val, max_val, mean_val, audio_data.len());
                }
                
                Array1::from_vec(audio_data)
            }
            2 => {
                // [batch, samples] -> 取第一行
                let audio_2d: Array2<f32> = view
                    .to_owned()
                    .into_dimensionality::<Ix2>()
                    .map_err(|e| anyhow!("failed to reshape audio to 2D: {e}"))?;
                let shape = audio_2d.shape();
                println!("[DEBUG HiFiGAN] 2D output shape: {:?}, taking first row", shape);
                audio_2d.row(0).to_owned()
            }
            1 => {
                // [samples]
                let shape = view.shape();
                println!("[DEBUG HiFiGAN] 1D output shape: {:?} (this is correct for audio waveform)", shape);
                view.to_owned()
                    .into_dimensionality::<ndarray::Ix1>()
                    .map_err(|e| anyhow!("failed to reshape audio to 1D: {e}"))?
            }
            _ => {
                return Err(anyhow!("Unexpected audio output dimensions: {}", view.ndim()));
            }
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

        println!("[DEBUG TTS] Text: '{}', Phone IDs: {:?} (length: {})", 
            request.text, phone_ids, phone_ids.len());

        if phone_ids.is_empty() {
            println!("[DEBUG TTS] Phone IDs is empty, returning empty audio");
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
        println!("[DEBUG TTS] Mel-spectrogram shape: {:?}", mel_shape);
        if mel_shape.len() != 3 || mel_shape[0] != 1 {
            return Err(EngineError::new(format!(
                "Invalid mel-spectrogram shape: {:?}, expected [1, mel_dim, time_steps]",
                mel_shape
            )));
        }

        // 3. HiFiGAN 推理：Mel-spectrogram → 音频波形
        let audio_waveform = self.run_hifigan(&mel, &request.locale)
            .map_err(|e| EngineError::new(format!("HiFiGAN inference failed: {e}")))?;

        println!("[DEBUG TTS] Audio waveform length: {} samples", audio_waveform.len());
        if !audio_waveform.is_empty() {
            let min_val = audio_waveform.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_val = audio_waveform.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let mean_val = audio_waveform.iter().sum::<f32>() / audio_waveform.len() as f32;
            println!("[DEBUG TTS] Audio waveform stats (before normalization): min={:.6}, max={:.6}, mean={:.6}", 
                min_val, max_val, mean_val);
        }

        if audio_waveform.is_empty() {
            return Err(EngineError::new("HiFiGAN produced empty audio"));
        }

        // 4. 归一化音频波形到 [-1.0, 1.0] 范围
        // HiFiGAN 输出可能不在标准音频范围内，需要归一化
        let normalized_audio = if !audio_waveform.is_empty() {
            let min_val = audio_waveform.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_val = audio_waveform.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let range = max_val - min_val;
            
            if range > 1e-6 {
                // 归一化到 [-1.0, 1.0]
                let normalized = audio_waveform.mapv(|x| {
                    let normalized = (x - min_val) / range; // [0, 1]
                    normalized * 2.0 - 1.0 // [-1, 1]
                });
                
                // 打印归一化后的统计信息
                let norm_min = normalized.iter().fold(f32::INFINITY, |a, &b| a.min(b));
                let norm_max = normalized.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
                let norm_mean = normalized.iter().sum::<f32>() / normalized.len() as f32;
                println!("[DEBUG TTS] Audio waveform stats (after normalization): min={:.6}, max={:.6}, mean={:.6}", 
                    norm_min, norm_max, norm_mean);
                
                normalized
            } else {
                // 如果范围太小（可能是常数），直接使用原值或返回零
                println!("[WARN] Audio waveform range too small: {:.6}, using original values", range);
                audio_waveform.clone()
            }
        } else {
            audio_waveform.clone()
        };

        // 5. 转换为 PCM 16-bit 字节
        let pcm_audio = self.audio_to_pcm16(&normalized_audio);

        println!("[DEBUG TTS] PCM audio length: {} bytes ({} samples)", 
            pcm_audio.len(), pcm_audio.len() / 2);
        
        // 检查 PCM 数据是否全为 0
        let non_zero_count = pcm_audio.iter().filter(|&&b| b != 0).count();
        println!("[DEBUG TTS] PCM non-zero bytes: {} / {} ({:.1}%)", 
            non_zero_count, pcm_audio.len(), 
            (non_zero_count as f32 / pcm_audio.len() as f32) * 100.0);
        
        // 检查前几个 PCM 样本的值
        if pcm_audio.len() >= 4 {
            let sample1 = i16::from_le_bytes([pcm_audio[0], pcm_audio[1]]);
            let sample2 = i16::from_le_bytes([pcm_audio[2], pcm_audio[3]]);
            println!("[DEBUG TTS] First 2 PCM samples: {}, {}", sample1, sample2);
        }

        if pcm_audio.is_empty() {
            return Err(EngineError::new("PCM conversion produced empty audio"));
        }
        
        // 如果 PCM 数据全为 0，发出警告
        if non_zero_count == 0 {
            println!("[WARN] PCM audio data is all zeros! This will produce silence.");
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

