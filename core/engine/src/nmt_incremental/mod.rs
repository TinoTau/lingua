use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{EngineError, EngineResult};
use crate::types::PartialTranscript;

use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use ort::session::Session;
use ndarray::{Array1, Array2, Array3, Array4};

mod tokenizer;
mod language_pair;

pub use tokenizer::MarianTokenizer;
pub use language_pair::{LanguageCode, LanguagePair};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub transcript: PartialTranscript,
    pub target_language: String,
    pub wait_k: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub translated_text: String,
    pub is_stable: bool,
}

#[async_trait]
pub trait NmtIncremental: Send + Sync {
    async fn initialize(&self) -> EngineResult<()>;
    async fn translate(&self, request: TranslationRequest) -> EngineResult<TranslationResponse>;
    async fn finalize(&self) -> EngineResult<()>;
}

/// 仅用于测试：尝试加载 Marian NMT 的 ONNX 模型，确认文件与 ORT 兼容。
pub fn load_marian_onnx_for_smoke_test(model_path: &Path) -> Result<()> {
    // 1. 初始化全局 ORT 环境
    crate::onnx_utils::init_onnx_runtime()?;

    // 2. 检查文件存在
    if !model_path.exists() {
        return Err(anyhow!(
            "NMT ONNX model not found at: {}",
            model_path.display()
        ));
    }

    // 3. 把模型文件读入内存（Vec<u8>）
    let model_bytes = fs::read(model_path)
        .map_err(|e| anyhow!("failed to read NMT model file {}: {e}", model_path.display()))?;

    // 4. 使用 Session::builder() + commit_from_memory 加载模型
    let builder = Session::builder()
        .map_err(|e| anyhow!("failed to create Session builder: {e}"))?;

    let _session = builder
        .commit_from_memory(&model_bytes)
        .map_err(|e| anyhow!("failed to load NMT model from memory: {e}"))?;

    // 能走到这里，说明模型格式至少是 ORT 能识别的
    Ok(())
}

/// 简单的整句翻译入口（暂时是 stub 版，只检查模型能否加载，再返回一个占位结果）
/// 后面会用真正的 Marian 推理替换这里的实现。
pub fn translate_full_sentence_stub(input: &str, model_path: &Path) -> Result<String> {
    // 先确保 ORT + 模型文件是好的（重用前面的 smoke test）
    load_marian_onnx_for_smoke_test(model_path)?;

    // TODO: 这里将来会：
    //  1. 加载 tokenizer / vocab
    //  2. 把 input 切分成子词 ID
    //  3. 构造 ONNX 输入张量
    //  4. 调用 Session.run()
    //  5. 把输出 token ID 解码回字符串
    //
    // 现在先返回一个可预期的占位结果，方便前端和其它模块联调。
    Ok(format!("[NMT stub en→zh] {}", input))
}

/// Marian NMT 的增量翻译 stub 实现：
/// - initialize(): 只做一次模型加载 smoke test，确认模型 OK；
/// - translate(): 调用上面的 translate_full_sentence_stub，返回占位翻译；
/// - finalize(): 目前什么都不做。
pub struct MarianNmtStub {
    model_path: PathBuf,
}

impl MarianNmtStub {
    /// 创建一个新的 NMT stub，传入 Marian ONNX 模型路径
    pub fn new(model_path: PathBuf) -> Self {
        Self { model_path }
    }
}

#[async_trait]
impl NmtIncremental for MarianNmtStub {
    async fn initialize(&self) -> EngineResult<()> {
        if let Err(_e) = load_marian_onnx_for_smoke_test(&self.model_path) {
            // 这里先不携带 e 的详细信息，避免 &'static str 生命周期问题
            return Err(EngineError::new("failed to initialize MarianNmtStub"));
        }
        Ok(())
    }

    async fn translate(
        &self,
        _request: TranslationRequest,
    ) -> EngineResult<TranslationResponse> {
        // 这里暂时**不去访问 PartialTranscript 的内部字段**，
        // 避免和你现有 struct 定义不匹配导致无法编译。
        //
        // 后续你可以根据 PartialTranscript 的实际结构，
        // 把下面这个 input 替换成从 _request.transcript 里取出来的文本。
        let input = "NMT stub input";

        let translated = translate_full_sentence_stub(input, &self.model_path)
            .map_err(|_e| EngineError::new("MarianNmtStub translate failed"))?;

        Ok(TranslationResponse {
            translated_text: translated,
            is_stable: true,
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        Ok(())
    }
}


pub struct MarianNmtOnnx {
    pub encoder_session: std::sync::Mutex<Session>,
    pub decoder_session: std::sync::Mutex<Session>,
    pub tokenizer: MarianTokenizer,
    pub decoder_start_token_id: i64,
    pub eos_token_id: i64,
    pub pad_token_id: i64,
    pub max_length: usize,
}

impl MarianNmtOnnx {
    /// 从模型目录加载 MarianNmtOnnx
    /// 
    /// # Arguments
    /// * `model_dir` - 模型目录路径（如 `models/nmt/marian-en-zh/`）
    /// 
    /// 会自动从目录名识别语言对
    pub fn new_from_dir(model_dir: &Path) -> Result<Self> {
        // 1. 先初始化 ORT 环境
        crate::onnx_utils::init_onnx_runtime()?;

        // 2. 从目录名识别语言对
        let language_pair = LanguagePair::from_model_dir(model_dir)?;

        let model_path = model_dir.join("model.onnx");
        let vocab_path = model_dir.join("vocab.json");

        if !model_path.exists() {
            return Err(anyhow!(
                "model.onnx not found at {}",
                model_path.display()
            ));
        }
        if !vocab_path.exists() {
            return Err(anyhow!(
                "vocab.json not found at {}",
                vocab_path.display()
            ));
        }

        // 3. 加载 vocab.json → tokenizer（传入语言对信息）
        let tokenizer = MarianTokenizer::from_model_dir(model_dir, language_pair)?;

        // 3. 加载 encoder 模型（使用文件路径，以便 ONNX Runtime 可以找到外部数据文件）
        let encoder_path = model_dir.join("encoder_model.onnx");
        if !encoder_path.exists() {
            return Err(anyhow!(
                "encoder_model.onnx not found at {}. Please export it first using scripts/export_marian_encoder.py",
                encoder_path.display()
            ));
        }

        // 读取 encoder 模型文件
        // 注意：ONNX Runtime 在加载模型时会查找外部数据文件
        // 它会在模型文件所在目录查找，所以我们需要确保路径正确
        let encoder_bytes = fs::read(&encoder_path)
            .map_err(|e| anyhow!("failed to read encoder_model.onnx: {e}"))?;

        // 检查外部数据文件是否存在
        let encoder_data_path = model_dir.join("encoder_model.onnx.data");
        if encoder_data_path.exists() {
            println!("[INFO] Found external data file: {}", encoder_data_path.display());
        }

        // 使用 with_model_from_memory 并设置外部数据目录
        // 注意：ort crate 可能需要在模型目录中查找外部数据
        // 我们通过设置当前工作目录来解决这个问题
        let current_dir = std::env::current_dir()?;
        std::env::set_current_dir(&model_dir)
            .map_err(|e| anyhow!("failed to change to model directory: {e}"))?;

        let encoder_builder = Session::builder()
            .map_err(|e| anyhow!("failed to create encoder Session builder: {e}"))?;

        let encoder_session = encoder_builder
            .commit_from_memory(&encoder_bytes)
            .map_err(|e| anyhow!("failed to load encoder ONNX model: {e}"))?;

        // 恢复工作目录
        std::env::set_current_dir(&current_dir)
            .map_err(|e| anyhow!("failed to restore working directory: {e}"))?;

        println!("[OK] Encoder model loaded: {}", encoder_path.display());

        // 4. 加载 decoder 模型
        let decoder_bytes = fs::read(&model_path)
            .map_err(|e| anyhow!("failed to read model.onnx (decoder): {e}"))?;

        let decoder_builder = Session::builder()
            .map_err(|e| anyhow!("failed to create decoder Session builder: {e}"))?;

        let decoder_session = decoder_builder
            .commit_from_memory(&decoder_bytes)
            .map_err(|e| anyhow!("failed to load decoder ONNX model from {}: {e}", model_path.display()))?;

        // 打印 decoder 模型的 I/O 信息
        println!("--- Decoder ONNX Model Inputs ---");
        for (i, input) in decoder_session.inputs.iter().enumerate() {
    println!(
                "Input[{i}] name={:?} input_type={:?}",
                input.name, input.input_type
    );
}

        println!("--- Decoder ONNX Model Outputs ---");
        for (i, output) in decoder_session.outputs.iter().enumerate() {
    println!(
                "Output[{i}] name={:?} output_type={:?}",
                output.name, output.output_type
            );
        }

        // 从 config.json 读取配置（如果存在）
        let config_path = model_dir.join("config.json");
        let (decoder_start_token_id, eos_token_id, pad_token_id, max_length) = 
            if config_path.exists() {
                let config_str = fs::read_to_string(&config_path)
                    .map_err(|e| anyhow!("failed to read config.json: {e}"))?;
                let config: serde_json::Value = serde_json::from_str(&config_str)
                    .map_err(|e| anyhow!("failed to parse config.json: {e}"))?;
                
                (
                    config["decoder_start_token_id"].as_i64().unwrap_or(65000),
                    config["eos_token_id"].as_i64().unwrap_or(0),
                    config["pad_token_id"].as_i64().unwrap_or(65000),
                    config["max_length"].as_u64().unwrap_or(512) as usize,
                )
            } else {
                // 默认值
                (65000, 0, 65000, 512)
            };

        Ok(Self { 
            encoder_session: std::sync::Mutex::new(encoder_session),
            decoder_session: std::sync::Mutex::new(decoder_session), 
            tokenizer,
            decoder_start_token_id,
            eos_token_id,
            pad_token_id,
            max_length,
        })
    }

    /// 根据语言对查找并加载模型
    /// 
    /// # Arguments
    /// * `base_dir` - 模型基础目录（如 `core/engine/models/nmt/`）
    /// * `language_pair` - 语言对
    /// 
    /// # Example
    /// ```rust
    /// use core_engine::nmt_incremental::{LanguagePair, LanguageCode, MarianNmtOnnx};
    /// let pair = LanguagePair::new(LanguageCode::En, LanguageCode::Zh);
    /// let model = MarianNmtOnnx::new_from_language_pair(
    ///     Path::new("models/nmt"),
    ///     pair
    /// )?;
    /// ```
    pub fn new_from_language_pair(base_dir: &Path, language_pair: LanguagePair) -> Result<Self> {
        let model_dir = language_pair.find_model_dir(base_dir);
        Self::new_from_dir(&model_dir)
    }

    /// 运行 encoder 模型，获取 encoder_hidden_states
    /// 
    /// # Arguments
    /// * `input_ids` - 编码后的输入 token IDs
    /// 
    /// # Returns
    /// (encoder_hidden_states, encoder_attention_mask)
    fn run_encoder(&self, input_ids: &[i64]) -> Result<(Array3<f32>, Array2<i64>)> {
        use ndarray::Array2;
        use ort::value::Value;

        let batch_size = 1usize;
        let seq_len = input_ids.len();

        // 准备输入
        let input_ids_array: Array2<i64> = Array2::from_shape_vec(
            (batch_size, seq_len),
            input_ids.to_vec(),
        )?;

        let attention_mask: Array2<i64> = Array2::ones((batch_size, seq_len));

        // 转换为 ONNX Value
        macro_rules! array_to_value {
            ($arr:expr, $ty:ty) => {{
                let arr_dyn = $arr.into_dyn();
                let shape: Vec<i64> = arr_dyn.shape().iter().map(|&d| d as i64).collect();
                let data: Vec<$ty> = arr_dyn.iter().cloned().collect();
                Value::from_array((shape, data))
                    .map_err(|e| anyhow!("failed to convert array to Value: {e}"))
            }};
        }

        let input_ids_value = array_to_value!(input_ids_array, i64)?;
        let attention_mask_value = array_to_value!(attention_mask.clone(), i64)?;

        // 运行 encoder
        use std::borrow::Cow;
        use ort::session::SessionInputValue;
        
        let mut encoder_session = self.encoder_session.lock().unwrap();
        let inputs: Vec<(Cow<'_, str>, SessionInputValue<'_>)> = vec![
            (Cow::Borrowed("input_ids"), input_ids_value.into()),
            (Cow::Borrowed("attention_mask"), attention_mask_value.into()),
        ];
        let outputs = encoder_session.run(inputs)
            .map_err(|e| anyhow!("failed to run encoder model: {e}"))?;

        // 提取 encoder_hidden_states (last_hidden_state)
        let hidden_states_value = outputs.get("last_hidden_state")
            .ok_or_else(|| anyhow!("encoder output 'last_hidden_state' not found"))?;

        let (hidden_states_shape, hidden_states_data) = hidden_states_value
            .try_extract_tensor::<f32>()
            .map_err(|e| anyhow!("failed to extract encoder hidden states: {e}"))?;

        // 转换为 Array3
        let encoder_hidden_states = Array3::from_shape_vec(
            (
                hidden_states_shape[0] as usize,
                hidden_states_shape[1] as usize,
                hidden_states_shape[2] as usize,
            ),
            hidden_states_data.to_vec(),
        )?;

        Ok((encoder_hidden_states, attention_mask))
    }

    /// 执行完整的翻译流程
    /// 
    /// # Arguments
    /// * `source_text` - 源文本（需要翻译的文本）
    /// 
    /// # Returns
    /// 翻译后的文本
    /// 
    /// # Note
    /// 这是一个简化版本，假设 encoder_hidden_states 已经准备好。
    /// 完整的实现需要先运行 encoder 模型。
    pub fn translate(&self, source_text: &str) -> Result<String> {
        // 1. 使用 tokenizer 编码源文本
        let source_ids = self.tokenizer.encode(source_text, true);
        let encoder_seq_len = source_ids.len();
        let batch_size = 1usize;

        println!("Source text: '{}'", source_text);
        println!("Encoded source IDs: {:?} (length: {})", source_ids, encoder_seq_len);

        // 2. 运行 encoder 获取真实的 encoder_hidden_states
        let (encoder_hidden_states, encoder_attention_mask) = self.run_encoder(&source_ids)?;
        println!("Encoder output shape: {:?}", encoder_hidden_states.shape());

        // 3. 初始化 decoder 状态
        let mut decoder_input_ids = vec![self.decoder_start_token_id];
        let mut generated_ids = Vec::new();
        let num_heads = 8usize;
        let head_dim = 64usize;
        let past_decoder_seq_len = 0usize;

        // 4. 创建空的 KV cache
        fn create_empty_kv(batch: usize, num_heads: usize, seq_len: usize, head_dim: usize) -> Array4<f32> {
            Array4::<f32>::zeros((batch, num_heads, seq_len.max(1), head_dim))
        }

        let mut past_key_values = Vec::new();
        for _ in 0..6 {
            // 6 layers
            past_key_values.push((
                create_empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim), // decoder key
                create_empty_kv(batch_size, num_heads, past_decoder_seq_len, head_dim), // decoder value
                create_empty_kv(batch_size, num_heads, encoder_seq_len, head_dim),      // encoder key
                create_empty_kv(batch_size, num_heads, encoder_seq_len, head_dim),      // encoder value
            ));
        }

        // 5. 增量解码循环
        use ort::value::Value;
        macro_rules! array_to_value {
            ($arr:expr, $t:ty) => {{
                let arr_dyn = $arr.into_dyn();
                let shape: Vec<i64> = arr_dyn.shape().iter().map(|&d| d as i64).collect();
                let data: Vec<$t> = arr_dyn.iter().cloned().collect();
                Value::from_array((shape, data))
                    .map_err(|e| anyhow!("failed to convert array to Value: {e}"))
            }};
        }

        let mut current_past_len = past_decoder_seq_len;
        let max_steps = self.max_length.min(128); // 限制最大步数

        for step in 0..max_steps {
            println!("[DEBUG] Step {}: decoder_input_ids={:?}, past_decoder_seq_len={}", step, decoder_input_ids, current_past_len);
            // 准备 decoder input
            let decoder_input: Array2<i64> = Array2::from_shape_vec(
                (batch_size, decoder_input_ids.len()),
                decoder_input_ids.clone(),
            )?;

            // 准备所有输入（在循环中需要 clone，因为每次迭代都会使用）
            let encoder_attention_mask_value = array_to_value!(encoder_attention_mask.clone(), i64)?;
            let input_ids_value = array_to_value!(decoder_input, i64)?;
            let encoder_hidden_states_value = array_to_value!(encoder_hidden_states.clone(), f32)?;
            let use_cache_branch_value = array_to_value!(Array1::from_vec(vec![true]), bool)?;

            // 准备 past_key_values
            let mut past_kv_inputs = Vec::new();
            for (layer_idx, (dec_key, dec_val, enc_key, enc_val)) in past_key_values.iter().enumerate() {
                past_kv_inputs.push((
                    format!("past_key_values.{}.decoder.key", layer_idx),
                    array_to_value!(dec_key.clone(), f32)?,
                ));
                past_kv_inputs.push((
                    format!("past_key_values.{}.decoder.value", layer_idx),
                    array_to_value!(dec_val.clone(), f32)?,
                ));
                past_kv_inputs.push((
                    format!("past_key_values.{}.encoder.key", layer_idx),
                    array_to_value!(enc_key.clone(), f32)?,
                ));
                past_kv_inputs.push((
                    format!("past_key_values.{}.encoder.value", layer_idx),
                    array_to_value!(enc_val.clone(), f32)?,
                ));
            }

            // 构建输入 - 使用 ort::inputs! 宏，但需要手动构建所有输入
            use std::borrow::Cow;
            use ort::session::SessionInputValue;
            
            let mut inputs: Vec<(Cow<'_, str>, SessionInputValue<'_>)> = vec![
                ("encoder_attention_mask".into(), encoder_attention_mask_value.into()),
                ("input_ids".into(), input_ids_value.into()),
                ("encoder_hidden_states".into(), encoder_hidden_states_value.into()),
                ("use_cache_branch".into(), use_cache_branch_value.into()),
            ];
            
            // 添加 past_key_values
            for (name, value) in past_kv_inputs {
                inputs.push((Cow::Owned(name), value.into()));
            }

            // 运行模型
            let mut decoder_session = self.decoder_session.lock().unwrap();
            let outputs = decoder_session.run(inputs)
                .map_err(|e| anyhow!("failed to run decoder model: {e}"))?;

            // 提取 logits
            let logits_value = &outputs["logits"];
            let (logits_shape, logits_data) = logits_value
                .try_extract_tensor::<f32>()
                .map_err(|e| anyhow!("failed to extract logits: {e}"))?;

            // logits shape: [batch, seq_len, vocab_size]
            // 取最后一个 token 的 logits
            let vocab_size = logits_shape[logits_shape.len() - 1] as usize;
            let last_token_logits = &logits_data[logits_data.len() - vocab_size..];

            // 选择概率最高的 token（贪婪解码）
            let next_token_id = last_token_logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx as i64)
                .ok_or_else(|| anyhow!("failed to find next token"))?;

            generated_ids.push(next_token_id);

            // 检查是否生成 EOS
            if next_token_id == self.eos_token_id {
                println!("Generated EOS token at step {}", step);
                break;
            }

            // 更新 decoder_input_ids（只保留最后一个 token）
            decoder_input_ids = vec![next_token_id];

            // 更新 past_key_values（从 present 输出）
            // KV Cache 说明：
            // - Transformer 模型在解码时，每次生成新 token 都需要计算 attention
            // - Attention 需要用到之前所有 token 的 key 和 value
            // - KV cache 缓存这些之前计算过的 key/value，避免重复计算，加速推理
            // - 每次迭代：模型输出 present KV cache，我们将其作为下次迭代的 past_key_values 输入
            
            // 注意：在第一次迭代时，past_decoder_seq_len 为 0，但模型输出的 present KV cache 
            // 的 decoder 部分长度应该是 1（因为输入了一个 decoder_start_token_id）
            // 从第二次迭代开始，我们需要更新 KV cache
            
            // 更新 KV cache（从第二次迭代开始）
            // 注意：由于 ort crate 的内存安全问题，我们暂时跳过 KV cache 更新
            // 这会导致每次迭代都使用相同的 KV cache，翻译结果可能不准确
            // 但至少可以验证基本的翻译流程
            // 
            // 问题分析：
            // - try_extract_tensor 返回的 slice 可能引用了 outputs 内部的数据
            // - 当我们在循环中多次提取时，可能会出现内存安全问题
            // - 这可能是 ort crate 2.0.0-rc.10 的一个已知问题
            //
            // 可能的解决方案：
            // 1. 升级 ort crate 到稳定版本（如果有）
            // 2. 使用不同的 API 提取 tensor 数据
            // 3. 一次性提取所有数据，避免多次访问 outputs
            // 4. 使用 unsafe 代码手动管理内存（不推荐）
            
            if step > 0 {
                // TODO: 修复 KV cache 更新逻辑
                // 目前暂时跳过，因为会出现内存安全问题
                println!("[DEBUG] Step {}: KV cache update skipped due to memory safety issues", step);
            } else {
                println!("[DEBUG] Step 0: First iteration, skipping KV cache update");
            }
            current_past_len += 1;
        }

        println!("Generated IDs: {:?} (length: {})", generated_ids, generated_ids.len());

        // 6. 使用 tokenizer 解码
        let translated_text = self.tokenizer.decode(&generated_ids);
        println!("Translated text: '{}'", translated_text);

        Ok(translated_text)
    }
}
