use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::Arc;
use async_trait::async_trait;
use ort::session::Session;
use ort::{SessionBuilder, Environment};
use ort::tensor::OrtOwnedTensor;
use ort::value::Value;
use ndarray::{Array1, Array2, IxDyn, Ix2};
use std::ptr;
use ndarray::CowArray;

use crate::error::{EngineError, EngineResult};
use super::{TtsRequest, TtsStreamChunk, TtsStreaming};
use super::vits_zh_aishell3_tokenizer::VitsZhAishell3Tokenizer;

/// VITS TTS 引擎（使用 MMS TTS ONNX 模型）
/// 
/// 支持多语言，根据 locale 选择对应的模型
pub struct VitsTtsEngine {
    /// 英文模型会话
    session_en: Mutex<Session>,
    /// 中文模型会话（可选）
    session_zh: Option<Mutex<Session>>,
    /// 英文 tokenizer（MMS TTS 格式）
    tokenizer_en: VitsTokenizer,
    /// 中文 tokenizer（可选，可能是 MMS TTS 或 vits-zh-aishell3 格式）
    tokenizer_zh: Option<VitsTokenizer>,
    /// 中文 AISHELL3 tokenizer（可选，用于 vits-zh-aishell3 模型）
    tokenizer_zh_aishell3: Option<VitsZhAishell3Tokenizer>,
    /// 中文模型类型：true = vits-zh-aishell3, false = MMS TTS
    is_zh_aishell3: bool,
    sample_rate: u32,
    /// 模型根目录
    models_root: PathBuf,
}

/// VITS Tokenizer（字符级 tokenizer）
/// 
/// MMS TTS 使用字符级 tokenizer，vocab 只有 39 个字符
struct VitsTokenizer {
    char_to_id: std::collections::HashMap<char, i64>,
    pad_token_id: i64,
    unk_token_id: i64,
}

impl VitsTokenizer {
    /// 从模型目录加载 tokenizer
    /// 
    /// 从 tokenizer.json 中读取 vocab 映射
    fn from_model_dir(model_dir: &Path) -> Result<Self> {
        use serde_json::Value;
        
        let tokenizer_path = model_dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            return Err(anyhow!("tokenizer.json not found at {}", tokenizer_path.display()));
        }

        // 读取 tokenizer.json
        let tokenizer_data = std::fs::read_to_string(&tokenizer_path)
            .map_err(|e| anyhow!("failed to read tokenizer.json: {e}"))?;
        
        let json: Value = serde_json::from_str(&tokenizer_data)
            .map_err(|e| anyhow!("failed to parse tokenizer.json: {e}"))?;

        // 提取 vocab（从 model.vocab）
        let vocab = json
            .get("model")
            .and_then(|m| m.get("vocab"))
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("vocab not found in tokenizer.json"))?;

        // 构建 char -> id 映射
        let mut char_to_id = std::collections::HashMap::new();
        for (token, id_value) in vocab {
            let id = id_value
                .as_u64()
                .ok_or_else(|| anyhow!("invalid token ID in vocab: {}", id_value))?
                as i64;
            
            // token 可能是单个字符或特殊 token（如 "<unk>"）
            if token.len() == 1 {
                char_to_id.insert(token.chars().next().unwrap(), id);
            } else if token == "<unk>" {
                // 特殊处理 <unk>
                char_to_id.insert('\0', id); // 使用 '\0' 作为未知字符的占位符
            }
        }

        // 获取 pad_token_id（通常是 "k" = 0）
        let pad_token_id = *char_to_id.get(&'k').unwrap_or(&0);
        
        // 获取 unk_token_id（通常是 "<unk>" = 38）
        let unk_token_id = *char_to_id.get(&'\0').unwrap_or(&38);

        Ok(Self {
            char_to_id,
            pad_token_id,
            unk_token_id,
        })
    }

    /// 编码文本为 token IDs 和 attention mask
    /// 
    /// 根据 MMS TTS 的 tokenizer 逻辑（参考 Python transformers 的 VitsTokenizer）：
    /// 1. 文本转小写（normalize = true）
    /// 2. 过滤不在 vocab 中的字符（根据 normalizer 的正则表达式）
    /// 3. 每个字符映射到对应的 ID
    /// 4. 在每个字符之间插入空白 token（add_blank = true，使用 pad_token_id = 0）
    /// 5. 标点符号和空格被过滤掉
    /// 
    /// 返回：(input_ids, attention_mask)
    /// - input_ids: [1, seq_len] (int64)
    /// - attention_mask: [1, seq_len] (int64)
    fn encode(&self, text: &str) -> Result<(Vec<i64>, Vec<i64>)> {
        // 1. 转小写（根据 tokenizer_config.json: normalize = true）
        let text_lower = text.to_lowercase();
        
        // 2. 字符级编码，并在每个字符之间插入空白 token
        // 根据 Python 输出，模式是：每个字符前后都有空白 token (0)
        // 例如 "Hello" -> [0, 6, 0, 7, 0, 21, 0, 21, 0, 22, 0]
        let mut ids = Vec::new();
        
        // 先添加开头的空白 token
        ids.push(self.pad_token_id); // pad_token_id = 0 = "k"
        
        for ch in text_lower.chars() {
            // 检查是否是标点符号（根据 normalizer，标点符号会被过滤）
            // 常见的标点符号：. , ! ? ; : - ' " ( ) [ ] { } 等
            if ch.is_ascii_punctuation() {
                continue;
            }
            
            // 查找字符对应的 ID（包括空格，空格在 vocab 中 ID 是 19）
            if let Some(&id) = self.char_to_id.get(&ch) {
                // 添加字符 ID
                ids.push(id);
                // 在每个字符后添加空白 token
                ids.push(self.pad_token_id); // pad_token_id = 0 = "k"
            } else {
                // 未知字符：跳过（根据 normalizer，不在 vocab 中的字符会被过滤）
                // 注意：空格（' '）在 vocab 中，ID 是 19，不应该被跳过
                continue;
            }
        }

        // 3. 如果只有开头的空白 token，至少添加一个字符
        if ids.len() <= 1 {
            // 如果完全为空，添加一个 pad token
            if ids.is_empty() {
                ids.push(self.pad_token_id);
            }
        }

        // 4. 生成 attention_mask（所有 token 都是有效的，设为 1）
        let attention_mask = vec![1i64; ids.len()];

        Ok((ids, attention_mask))
    }
}

impl VitsTtsEngine {
    /// 从模型根目录加载 VITS TTS 引擎（支持多语言）
    /// 
    /// 模型目录结构：
    /// ```
    /// models_root/
    ///   ├── mms-tts-eng/       # 英文模型（必需）
    ///   └── vits-zh-aishell3/  # 中文模型（可选）
    /// ```
    pub fn new_from_models_root(models_root: &Path) -> Result<Self> {
        // 初始化 ONNX Runtime
        crate::onnx_utils::init_onnx_runtime()?;

        let models_root = models_root.to_path_buf();

        // 创建 ONNX Runtime 环境
        let env = Arc::new(
            Environment::builder()
                .with_name("vits_tts")
                .build()?
        );

        // 1. 加载英文模型（必需）
        let model_dir_en = models_root.join("mms-tts-eng");
        let tokenizer_en = VitsTokenizer::from_model_dir(&model_dir_en)?;
        let onnx_dir_en = model_dir_en.join("onnx");
        let model_path_en = if onnx_dir_en.join("model.onnx").exists() {
            onnx_dir_en.join("model.onnx")
        } else if onnx_dir_en.join("model_fp16.onnx").exists() {
            onnx_dir_en.join("model_fp16.onnx")
        } else {
            return Err(anyhow!("No ONNX model found in {}", onnx_dir_en.display()));
        };
        let session_en = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create VITS Session builder: {e}"))?
            .with_model_from_file(&model_path_en)
            .map_err(|e| anyhow!("failed to load English VITS model: {e}"))?;

        // 2. 尝试加载中文模型（可选）
        // 优先尝试 vits-zh-aishell3，如果没有则尝试 mms-tts-zh-Hans
        let model_dir_zh = if models_root.join("vits-zh-aishell3").exists() {
            models_root.join("vits-zh-aishell3")
        } else {
            models_root.join("mms-tts-zh-Hans")
        };
        
        // 检测模型类型：vits-zh-aishell3 有 tokens.txt 和 lexicon.txt
        let is_zh_aishell3 = model_dir_zh.exists() 
            && model_dir_zh.join("tokens.txt").exists() 
            && model_dir_zh.join("lexicon.txt").exists();
        
        let (session_zh, tokenizer_zh, tokenizer_zh_aishell3) = if model_dir_zh.exists() {
            if is_zh_aishell3 {
                // 加载 vits-zh-aishell3 模型
                match Self::load_aishell3_model(&env, &model_dir_zh) {
                    Ok((session, tokenizer)) => {
                        (Some(Mutex::new(session)), None, Some(tokenizer))
                    }
                    Err(e) => {
                        eprintln!("[WARN] Failed to load vits-zh-aishell3 model: {}. Chinese TTS will be unavailable.", e);
                        (None, None, None)
                    }
                }
            } else {
                // 加载 MMS TTS 中文模型
                match Self::load_model_for_locale(&env, &model_dir_zh) {
                    Ok((session, tokenizer)) => {
                        (Some(Mutex::new(session)), Some(tokenizer), None)
                    }
                    Err(e) => {
                        eprintln!("[WARN] Failed to load Chinese model: {}. Chinese TTS will be unavailable.", e);
                        (None, None, None)
                    }
                }
            }
        } else {
            (None, None, None)
        };

        // MMS TTS 的采样率是 16000 Hz
        // vits-zh-aishell3 的采样率通常是 22050 Hz 或 24000 Hz
        // 根据模型类型设置采样率
        let sample_rate = if is_zh_aishell3 {
            22050u32  // vits-zh-aishell3 通常使用 22050 Hz
        } else {
            16000u32  // MMS TTS 使用 16000 Hz
        };

        Ok(Self {
            session_en: Mutex::new(session_en),
            session_zh,
            tokenizer_en,
            tokenizer_zh,
            tokenizer_zh_aishell3,
            is_zh_aishell3,
            sample_rate,
            models_root,
        })
    }

    /// 加载 vits-zh-aishell3 模型
    fn load_aishell3_model(env: &Arc<Environment>, model_dir: &Path) -> Result<(Session, VitsZhAishell3Tokenizer)> {
        let tokenizer = VitsZhAishell3Tokenizer::from_model_dir(model_dir)?;
        
        // 尝试多种可能的 ONNX 模型路径
        let model_path = {
            if model_dir.join("vits-aishell3.onnx").exists() {
                model_dir.join("vits-aishell3.onnx")
            } else if model_dir.join("vits-aishell3.int8.onnx").exists() {
                model_dir.join("vits-aishell3.int8.onnx")
            } else {
                return Err(anyhow!("No ONNX model found in {}", model_dir.display()));
            }
        };
        
        let session = SessionBuilder::new(env)
            .map_err(|e| anyhow!("failed to create VITS Session builder: {e}"))?
            .with_model_from_file(&model_path)
            .map_err(|e| anyhow!("failed to load vits-zh-aishell3 model: {e}"))?;
        Ok((session, tokenizer))
    }
    
    /// 加载指定语言模型的辅助函数（MMS TTS 格式）
    fn load_model_for_locale(env: &Arc<Environment>, model_dir: &Path) -> Result<(Session, VitsTokenizer)> {
        let tokenizer = VitsTokenizer::from_model_dir(model_dir)?;
        
        // 尝试多种可能的 ONNX 模型路径
        let model_path = {
            // 1. 检查 onnx/ 子目录（MMS TTS 格式）
            let onnx_dir = model_dir.join("onnx");
            if onnx_dir.join("model.onnx").exists() {
                onnx_dir.join("model.onnx")
            } else if onnx_dir.join("model_fp16.onnx").exists() {
                onnx_dir.join("model_fp16.onnx")
            }
            // 2. 检查模型目录根目录（vits-zh-aishell3 格式）
            else if model_dir.join("vits-aishell3.onnx").exists() {
                model_dir.join("vits-aishell3.onnx")
            } else if model_dir.join("vits-aishell3.int8.onnx").exists() {
                model_dir.join("vits-aishell3.int8.onnx")
            } else {
                return Err(anyhow!("No ONNX model found in {} or {}/onnx", 
                    model_dir.display(), model_dir.display()));
            }
        };
        
        let session = SessionBuilder::new(env)
            .map_err(|e| anyhow!("failed to create VITS Session builder: {e}"))?
            .with_model_from_file(&model_path)
            .map_err(|e| anyhow!("failed to load VITS model: {e}"))?;
        Ok((session, tokenizer))
    }

    /// 从单个模型目录加载 VITS TTS 引擎（仅英文，向后兼容）
    pub fn new_from_dir(model_dir: &Path) -> Result<Self> {
        let models_root = model_dir.parent()
            .ok_or_else(|| anyhow!("model_dir has no parent"))?;
        Self::new_from_models_root(models_root)
    }

    /// 运行 VITS 推理：文本 → 音频波形
    /// 
    /// 根据 locale 选择对应的模型和 tokenizer
    fn run_inference(&self, text: &str, locale: &str) -> Result<Array1<f32>> {
        // 根据 locale 选择模型类型
        match locale {
            "zh" | "zh-CN" | "zh-TW" | "cmn" => {
                if self.is_zh_aishell3 {
                    // 使用 vits-zh-aishell3 格式
                    if let (Some(ref session_zh), Some(ref tokenizer_zh_aishell3)) = (&self.session_zh, &self.tokenizer_zh_aishell3) {
                        return self.run_inference_aishell3(session_zh, tokenizer_zh_aishell3, text);
                    } else {
                        return Err(anyhow!("Chinese AISHELL3 model not available."));
                    }
                } else {
                    // 使用 MMS TTS 格式
                    if let (Some(ref session_zh), Some(ref tokenizer_zh)) = (&self.session_zh, &self.tokenizer_zh) {
                        return self.run_inference_mms(session_zh, tokenizer_zh, text);
                    } else {
                        return Err(anyhow!("Chinese model not available. Please download Chinese model."));
                    }
                }
            }
            _ => {
                // 默认使用英文模型（MMS TTS 格式）
                return self.run_inference_mms(&self.session_en, &self.tokenizer_en, text);
            }
        }
    }
    
    /// 运行 MMS TTS 格式的推理
    fn run_inference_mms(&self, session: &Mutex<Session>, tokenizer: &VitsTokenizer, text: &str) -> Result<Array1<f32>> {
        // 1. 编码文本
        let (input_ids, attention_mask) = tokenizer.encode(text)?;
        
        if input_ids.is_empty() {
            return Err(anyhow!("encoded input_ids is empty"));
        }
        
        // 调试输出：打印编码结果的前50个值
        println!("[DEBUG VITS] Text: '{}'", text);
        println!("[DEBUG VITS] Encoded input_ids length: {}", input_ids.len());
        println!("[DEBUG VITS] First 50 input_ids: {:?}", &input_ids[..input_ids.len().min(50)]);

        let batch_size = 1usize;
        let seq_len = input_ids.len();

        // 2. 准备输入张量
        // input_ids: [1, seq_len] (int64)
        let input_ids_array: Array2<i64> = Array2::from_shape_vec(
            (batch_size, seq_len),
            input_ids,
        )?;

        // attention_mask: [1, seq_len] (int64)
        let attention_mask_array: Array2<i64> = Array2::from_shape_vec(
            (batch_size, seq_len),
            attention_mask,
        )?;

        // 3. 转换为 ONNX Value
        let arr_dyn_ids = input_ids_array.into_dyn();
        let arr_owned_ids = arr_dyn_ids.to_owned();
        let cow_arr_ids = CowArray::from(arr_owned_ids);
        let input_ids_value = Value::from_array(ptr::null_mut(), &cow_arr_ids)
            .map_err(|e| anyhow!("failed to convert input_ids to Value: {:?}", e))?;
        let input_ids_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(input_ids_value)
        };

        let arr_dyn_mask = attention_mask_array.into_dyn();
        let arr_owned_mask = arr_dyn_mask.to_owned();
        let cow_arr_mask = CowArray::from(arr_owned_mask);
        let attention_mask_value = Value::from_array(ptr::null_mut(), &cow_arr_mask)
            .map_err(|e| anyhow!("failed to convert attention_mask to Value: {:?}", e))?;
        let attention_mask_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(attention_mask_value)
        };

        // 4. 运行推理
        // 注意：输入顺序必须与模型定义的输入顺序一致
        // 根据 Python 测试，顺序是：input_ids, attention_mask
        let session_guard = session.lock().unwrap();
        let inputs = vec![input_ids_value, attention_mask_value];
        let outputs: Vec<Value> = session_guard.run(inputs)
            .map_err(|e| anyhow!("failed to run VITS model: {e}"))?;

        // 5. 提取 waveform 输出（第一个输出）
        let waveform_value = outputs.get(0)
            .ok_or_else(|| anyhow!("VITS output is empty"))?;

        // 6. 转换为 ndarray
        let tensor: OrtOwnedTensor<f32, IxDyn> = waveform_value.try_extract()
            .map_err(|e| anyhow!("failed to extract waveform tensor: {e}"))?;
        let view = tensor.view();

        // waveform 输出应该是 [1, n_samples] 或 [n_samples]
        let audio: Array1<f32> = match view.ndim() {
            2 => {
                // [batch, samples] -> 取第一行
                let audio_2d: Array2<f32> = view
                    .to_owned()
                    .into_dimensionality::<Ix2>()
                    .map_err(|e| anyhow!("failed to reshape waveform to 2D: {e}"))?;
                audio_2d.row(0).to_owned()
            }
            1 => {
                // [samples]
                view.to_owned()
                    .into_dimensionality::<ndarray::Ix1>()
                    .map_err(|e| anyhow!("failed to reshape waveform to 1D: {e}"))?
            }
            _ => {
                return Err(anyhow!("Unexpected waveform output dimensions: {}", view.ndim()));
            }
        };

        // 调试输出：打印音频统计信息
        let min_val = audio.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = audio.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let mean_val = audio.iter().sum::<f32>() / audio.len() as f32;
        println!("[DEBUG VITS] Audio waveform: length={}, min={:.6}, max={:.6}, mean={:.6}", 
            audio.len(), min_val, max_val, mean_val);
        println!("[DEBUG VITS] Expected duration @ 16kHz: {:.2} seconds", audio.len() as f32 / 16000.0);
        
        Ok(audio)
    }
    
    /// 运行 vits-zh-aishell3 格式的推理
    fn run_inference_aishell3(&self, session: &Mutex<Session>, tokenizer: &VitsZhAishell3Tokenizer, text: &str) -> Result<Array1<f32>> {
        // 1. 编码文本
        let (token_ids, _seq_len_from_tokenizer) = tokenizer.encode(text)?;
        
        if token_ids.is_empty() {
            return Err(anyhow!("encoded token_ids is empty"));
        }
        
        // 调试输出
        println!("[DEBUG VITS AISHELL3] Text: '{}'", text);
        println!("[DEBUG VITS AISHELL3] Encoded token_ids length: {}", token_ids.len());
        println!("[DEBUG VITS AISHELL3] First 50 token_ids: {:?}", &token_ids[..token_ids.len().min(50)]);
        
        let batch_size = 1usize;
        let seq_len_usize = token_ids.len();
        
        // 2. 准备输入张量
        // x: [N, L] (int64) - token IDs
        let x_array: Array2<i64> = Array2::from_shape_vec(
            (batch_size, seq_len_usize),
            token_ids,
        )?;
        
        // x_length: [N] (int64) - 序列长度
        let x_length_array: Array1<i64> = Array1::from_vec(vec![seq_len_usize as i64]);
        
        // noise_scale: [1] (float) - 控制音调变化
        // 减小值可以降低声音尖锐度，默认 0.667
        // 测试3的参数：0.5（语速合理但发音仍不清楚）
        let noise_scale_array: Array1<f32> = Array1::from_vec(vec![0.5f32]);
        
        // length_scale: [1] (float) - 控制语速
        // >1.0 变慢，<1.0 变快，默认 1.0
        // 测试3的参数：2.0（语速合理）
        let length_scale_array: Array1<f32> = Array1::from_vec(vec![2.0f32]);
        
        // noise_scale_w: [1] (float) - 控制音调变化（另一个维度）
        // 减小值可以降低声音尖锐度，默认 0.8
        // 测试3的参数：0.6（语速合理但发音仍不清楚）
        let noise_scale_w_array: Array1<f32> = Array1::from_vec(vec![0.6f32]);
        
        // sid: [1] (int64) - 说话人 ID，默认 0
        // 尝试不同的说话人可能改善音质，但需要知道模型支持的说话人数量
        // 暂时保持 0，如果音质仍不好，可以尝试 1, 2, 3 等
        let sid_array: Array1<i64> = Array1::from_vec(vec![0i64]);
        
        // 3. 转换为 ONNX Value
        let arr_dyn_x = x_array.into_dyn();
        let arr_owned_x = arr_dyn_x.to_owned();
        let cow_arr_x = CowArray::from(arr_owned_x);
        let x_value = Value::from_array(ptr::null_mut(), &cow_arr_x)
            .map_err(|e| anyhow!("failed to convert x to Value: {:?}", e))?;
        let x_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(x_value)
        };
        
        let arr_dyn_x_length = x_length_array.into_dyn();
        let arr_owned_x_length = arr_dyn_x_length.to_owned();
        let cow_arr_x_length = CowArray::from(arr_owned_x_length);
        let x_length_value = Value::from_array(ptr::null_mut(), &cow_arr_x_length)
            .map_err(|e| anyhow!("failed to convert x_length to Value: {:?}", e))?;
        let x_length_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(x_length_value)
        };
        
        let arr_dyn_noise_scale = noise_scale_array.into_dyn();
        let arr_owned_noise_scale = arr_dyn_noise_scale.to_owned();
        let cow_arr_noise_scale = CowArray::from(arr_owned_noise_scale);
        let noise_scale_value = Value::from_array(ptr::null_mut(), &cow_arr_noise_scale)
            .map_err(|e| anyhow!("failed to convert noise_scale to Value: {:?}", e))?;
        let noise_scale_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(noise_scale_value)
        };
        
        let arr_dyn_length_scale = length_scale_array.into_dyn();
        let arr_owned_length_scale = arr_dyn_length_scale.to_owned();
        let cow_arr_length_scale = CowArray::from(arr_owned_length_scale);
        let length_scale_value = Value::from_array(ptr::null_mut(), &cow_arr_length_scale)
            .map_err(|e| anyhow!("failed to convert length_scale to Value: {:?}", e))?;
        let length_scale_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(length_scale_value)
        };
        
        let arr_dyn_noise_scale_w = noise_scale_w_array.into_dyn();
        let arr_owned_noise_scale_w = arr_dyn_noise_scale_w.to_owned();
        let cow_arr_noise_scale_w = CowArray::from(arr_owned_noise_scale_w);
        let noise_scale_w_value = Value::from_array(ptr::null_mut(), &cow_arr_noise_scale_w)
            .map_err(|e| anyhow!("failed to convert noise_scale_w to Value: {:?}", e))?;
        let noise_scale_w_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(noise_scale_w_value)
        };
        
        let arr_dyn_sid = sid_array.into_dyn();
        let arr_owned_sid = arr_dyn_sid.to_owned();
        let cow_arr_sid = CowArray::from(arr_owned_sid);
        let sid_value = Value::from_array(ptr::null_mut(), &cow_arr_sid)
            .map_err(|e| anyhow!("failed to convert sid to Value: {:?}", e))?;
        let sid_value: Value<'static> = unsafe {
            std::mem::transmute::<Value, Value<'static>>(sid_value)
        };
        
        // 4. 运行推理
        // 输入顺序：x, x_length, noise_scale, length_scale, noise_scale_w, sid
        let session_guard = session.lock().unwrap();
        let inputs = vec![x_value, x_length_value, noise_scale_value, length_scale_value, noise_scale_w_value, sid_value];
        let outputs: Vec<Value> = session_guard.run(inputs)
            .map_err(|e| anyhow!("failed to run vits-zh-aishell3 model: {e}"))?;
        
        // 5. 提取 waveform 输出（第一个输出）
        let waveform_value = outputs.get(0)
            .ok_or_else(|| anyhow!("vits-zh-aishell3 output is empty"))?;
        
        // 6. 转换为 ndarray
        let tensor: OrtOwnedTensor<f32, IxDyn> = waveform_value.try_extract()
            .map_err(|e| anyhow!("failed to extract waveform tensor: {e}"))?;
        let view = tensor.view();
        
        // waveform 输出是 [N, 1, L]，需要 squeeze 成 [L]
        let audio: Array1<f32> = match view.ndim() {
            3 => {
                // [batch, 1, samples] -> 取第一行，然后 squeeze
                let audio_3d: ndarray::Array3<f32> = view
                    .to_owned()
                    .into_dimensionality::<ndarray::Ix3>()
                    .map_err(|e| anyhow!("failed to reshape waveform to 3D: {e}"))?;
                audio_3d.slice(ndarray::s![0, 0, ..]).to_owned()
            }
            2 => {
                // [batch, samples] -> 取第一行
                let audio_2d: Array2<f32> = view
                    .to_owned()
                    .into_dimensionality::<Ix2>()
                    .map_err(|e| anyhow!("failed to reshape waveform to 2D: {e}"))?;
                audio_2d.row(0).to_owned()
            }
            1 => {
                // [samples]
                view.to_owned()
                    .into_dimensionality::<ndarray::Ix1>()
                    .map_err(|e| anyhow!("failed to reshape waveform to 1D: {e}"))?
            }
            _ => {
                return Err(anyhow!("Unexpected waveform output dimensions: {}", view.ndim()));
            }
        };
        
        // 调试输出：打印音频统计信息
        let min_val = audio.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = audio.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let mean_val = audio.iter().sum::<f32>() / audio.len() as f32;
        println!("[DEBUG VITS AISHELL3] Audio waveform: length={}, min={:.6}, max={:.6}, mean={:.6}", 
            audio.len(), min_val, max_val, mean_val);
        println!("[DEBUG VITS AISHELL3] Expected duration @ 22.05kHz: {:.2} seconds", audio.len() as f32 / 22050.0);
        
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
}

#[async_trait]
impl TtsStreaming for VitsTtsEngine {
    async fn synthesize(&self, request: TtsRequest) -> EngineResult<TtsStreamChunk> {
        // 1. 运行推理生成音频波形（根据 locale 选择模型）
        let audio_waveform = self.run_inference(&request.text, &request.locale)
            .map_err(|e| EngineError::new(format!("VITS inference failed: {e}")))?;

        if audio_waveform.is_empty() {
            return Err(EngineError::new("VITS produced empty audio"));
        }

        // 2. 转换为 PCM 16-bit 字节
        let pcm_audio = self.audio_to_pcm16(&audio_waveform);

        if pcm_audio.is_empty() {
            return Err(EngineError::new("PCM conversion produced empty audio"));
        }

        // 3. 创建 chunk（当前实现：一次性返回完整音频）
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
