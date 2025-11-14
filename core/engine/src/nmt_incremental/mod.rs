use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{EngineError, EngineResult};
use crate::types::PartialTranscript;

use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;
use ort::session::Session;
use ort::tensor::OrtOwnedTensor;
use ndarray::{Array1, Array2, Array3, Array4, IxDyn, Ix3};

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

    // 4. 使用 SessionBuilder 加载模型（文件模式）
    use ort::{SessionBuilder, Environment};
    use std::sync::Arc;
    let env = Arc::new(
        Environment::builder()
            .with_name("marian_nmt_test")
            .build()?
    );
    let _session = SessionBuilder::new(&env)
        .map_err(|e| anyhow!("failed to create Session builder: {e}"))?
        .with_model_from_file(model_path)
        .map_err(|e| anyhow!("failed to load NMT model: {e}"))?;

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

        // 3. 加载 encoder 模型（使用文件模式，避免 InMemorySession 生命周期问题）
        let encoder_path = model_dir.join("encoder_model.onnx");
        if !encoder_path.exists() {
            return Err(anyhow!("encoder model not found: {}", encoder_path.display()));
        }

        // ort 1.16.3: 使用文件模式创建 session（根据指南）
        // 使用 Environment::builder() 创建 Environment
        use ort::{SessionBuilder, Environment};
        use std::sync::Arc;
        let env = Arc::new(
            Environment::builder()
                .with_name("marian_nmt")
                .build()?
        );
        
        let encoder_session = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create encoder Session builder: {e}"))?
            .with_model_from_file(&encoder_path)
            .map_err(|e| anyhow!("failed to load encoder ONNX model from {}: {e}", encoder_path.display()))?;

        println!("[OK] Encoder model loaded: {}", encoder_path.display());

        // 4. 加载 decoder 模型（使用文件模式）
        if !model_path.exists() {
            return Err(anyhow!("decoder model not found: {}", model_path.display()));
        }

        let decoder_session = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create decoder Session builder: {e}"))?
            .with_model_from_file(&model_path)
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
        // ort 1.16.3 使用 from_array 需要 allocator 和 array（需要 CowRepr 类型）
        use std::ptr;
        use ndarray::CowArray;
        macro_rules! array_to_value {
            ($arr:expr, $ty:ty) => {{
                // 转换为动态维度和 CowRepr 类型，使用 null allocator（让 ORT 使用默认 allocator）
                // 注意：需要确保数据是 owned 的，这样 Value 可以拥有数据的所有权
                let arr_dyn = $arr.into_dyn();
                let arr_owned = arr_dyn.to_owned();
                let cow_arr = CowArray::from(arr_owned);
                // Value::from_array 会复制数据，但返回的 Value 仍然有生命周期限制
                // 使用 transmute 转换为 'static 生命周期（因为数据已经被复制到 ORT 内部）
                let value = Value::from_array(ptr::null_mut(), &cow_arr)
                    .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
                Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
            }};
        }

        let input_ids_value: ort::Value<'static> = array_to_value!(input_ids_array, i64)?;
        let attention_mask_value: ort::Value<'static> = array_to_value!(attention_mask.clone(), i64)?;

        // 运行 encoder
        // ort 1.16.3: session.run() 接受 Vec<Value>，按输入顺序排列
        // 注意：需要按照模型定义的输入顺序传递
        let encoder_session = self.encoder_session.lock().unwrap();
        let inputs = vec![input_ids_value, attention_mask_value];
        let outputs: Vec<ort::Value> = encoder_session.run(inputs)
            .map_err(|e| anyhow!("failed to run encoder model: {e}"))?;

        // 提取 encoder_hidden_states (last_hidden_state)
        // ort 1.16.3: 当使用 HashMap 构建的 inputs 时，outputs 是 Vec<Value>，按输出顺序排列
        // encoder 只有一个输出 last_hidden_state，索引为 0
        let hidden_states_value = &outputs[0];

        // ort 1.16.3: 使用 try_extract() + view() + into_dimensionality()
        let tensor: OrtOwnedTensor<f32, IxDyn> = hidden_states_value
            .try_extract::<f32>()
            .map_err(|e| anyhow!("failed to extract encoder hidden states: {e}"))?;
        let view = tensor.view();
        let encoder_hidden_states: Array3<f32> = view
            .to_owned()
            .into_dimensionality::<Ix3>()
            .map_err(|e| anyhow!("failed to convert to Array3: {e}"))?;

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
        let _num_heads = 8usize; // 用于创建空的 KV cache
        // 4. KV cache 状态：使用 Option<Vec<Value>> 存储，None 表示第一步（不传 KV cache）
        // 根据 recommended_kv_solution.md：第一步不传 KV cache，让模型内部初始化
        use ort::value::Value;
        let num_layers = 6usize; // 从模型结构推断
        let mut kv_cache: Option<Vec<Value<'static>>> = None;

        // 5. 增量解码循环
        // ort 1.16.3 使用 from_array 需要 allocator 和动态维度的 array
        macro_rules! array_to_value {
            ($arr:expr, $t:ty) => {{
                // 转换为动态维度和 CowRepr 类型，使用 null allocator（让 ORT 使用默认 allocator）
                // 注意：需要确保数据是 owned 的，这样 Value 可以拥有数据的所有权
                let arr_dyn = $arr.into_dyn();
                let arr_owned = arr_dyn.to_owned();
                let cow_arr = CowArray::from(arr_owned);
                // Value::from_array 会复制数据，但返回的 Value 仍然有生命周期限制
                // 使用 transmute 转换为 'static 生命周期（因为数据已经被复制到 ORT 内部）
                let value = Value::from_array(ptr::null_mut(), &cow_arr)
                    .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
                Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
            }};
        }

        let max_steps = self.max_length.min(128); // 限制最大步数

        for step in 0..max_steps {
            println!("[DEBUG] Step {}: decoder_input_ids={:?}", step, decoder_input_ids);
            // 准备 decoder input
            let decoder_input: Array2<i64> = Array2::from_shape_vec(
                (batch_size, decoder_input_ids.len()),
                decoder_input_ids.clone(),
            )?;

            // 准备基本输入
            let encoder_attention_mask_value = array_to_value!(encoder_attention_mask.clone(), i64)?;
            let input_ids_value = array_to_value!(decoder_input, i64)?;
            let encoder_hidden_states_value = array_to_value!(encoder_hidden_states.clone(), f32)?;
            
            // 根据 recommended_kv_solution.md：
            // Step 0: use_cache_branch = false，不传 KV cache（但实际上模型要求所有输入都存在）
            // Step N: use_cache_branch = true，传 KV cache
            // 注意：模型定义中 use_cache_branch 是 Bool 类型
            let is_first_step = kv_cache.is_none();
            let use_cache_branch_value = if is_first_step {
                array_to_value!(Array1::from_vec(vec![false]), bool)? // Step 0
            } else {
                array_to_value!(Array1::from_vec(vec![true]), bool)? // Step N
            };

            // 运行模型
            // ort 1.16.3: session.run() 接受 Vec<Value>，按输入顺序排列
            // 根据模型定义，输入顺序是：
            // encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.*, use_cache_branch
            let decoder_session = self.decoder_session.lock().unwrap();
            let mut inputs: Vec<ort::Value> = vec![
                encoder_attention_mask_value,
                input_ids_value,
                encoder_hidden_states_value,
            ];
            
            // 添加 past_key_values（所有输入都是必需的，包括第一步）
            // 根据 recommended_kv_solution.md：第一步不传 KV cache，但模型要求所有输入都存在
            // 所以我们需要在第一步也传递空的 KV cache，但使用 use_cache_branch = 0.0 来告诉模型忽略它们
            // 但实际上，根据文档，第一步应该不传 KV cache。但模型定义要求所有输入都存在。
            // 让我们尝试在第一步传递空的 KV cache，看看是否能工作。
            use ort::tensor::OrtOwnedTensor;
            use ndarray::{IxDyn, CowArray, Array4};
            
            if let Some(prev_kv) = &kv_cache {
                // Step N: 使用上一步的 KV cache
                for kv_value in prev_kv {
                    let kv_tensor: OrtOwnedTensor<f32, IxDyn> = kv_value.try_extract::<f32>()
                        .map_err(|e| anyhow!("failed to extract KV cache: {e}"))?;
                    let kv_view = kv_tensor.view();
                    let kv_arr = kv_view.to_owned().into_dyn();
                    let kv_cow = CowArray::from(kv_arr);
                    let kv_new = Value::from_array(ptr::null_mut(), &kv_cow)
                        .map_err(|e| anyhow!("failed to rebuild KV cache: {e}"))?;
                    inputs.push(unsafe { std::mem::transmute::<Value<'_>, Value<'static>>(kv_new) });
                }
            } else {
                // Step 0: 模型要求所有输入都存在，但根据 recommended_kv_solution.md，第一步应该不传 KV cache
                // 这里我们创建一个空的 KV cache，但使用 use_cache_branch = false 来告诉模型忽略它们
                // 创建空的 KV cache：每层 4 个值（dec_k, dec_v, enc_k, enc_v），每个都是空的 4D tensor
                // 形状：[batch=1, num_heads=8, seq_len=1, head_dim=64]（使用 1 而不是 0，因为 ONNX 不允许 0 维度）
                let batch = 1;
                let num_heads = 8;
                let head_dim = 64;
                let empty_seq_len = 1; // 使用 1 而不是 0，因为 ONNX 不允许 0 维度
                
                for _layer in 0..num_layers {
                    // 为每层创建 4 个空的 KV cache：dec_k, dec_v, enc_k, enc_v
                    for _ in 0..4 {
                        let empty_arr = Array4::<f32>::zeros((batch, num_heads, empty_seq_len, head_dim));
                        let arr_dyn = empty_arr.into_dyn();
                        let arr_owned = arr_dyn.to_owned();
                        let cow_arr = CowArray::from(arr_owned);
                        let kv_value = Value::from_array(ptr::null_mut(), &cow_arr)
                            .map_err(|e| anyhow!("failed to create empty KV cache: {e}"))?;
                        inputs.push(unsafe { std::mem::transmute::<Value<'_>, Value<'static>>(kv_value) });
                    }
                }
            }
            
            // use_cache_branch 是最后一个输入
            inputs.push(use_cache_branch_value);
            
            let outputs: Vec<ort::Value> = decoder_session.run(inputs)
                .map_err(|e| anyhow!("failed to run decoder model: {e}"))?;

            // 提取 logits
            // ort 1.16.3: 当使用 HashMap 构建的 inputs 时，outputs 是 Vec<Value>，按输出顺序排列
            // logits 是第一个输出（索引 0）
            let logits_value = &outputs[0];
            // ort 1.16.3: 使用 try_extract() + view() + into_dimensionality()
            // logits shape: [batch, seq_len, vocab_size]
            let tensor: OrtOwnedTensor<f32, IxDyn> = logits_value
                .try_extract::<f32>()
                .map_err(|e| anyhow!("failed to extract logits: {e}"))?;
            let view = tensor.view();
            let logits_arr: Array3<f32> = view
                .to_owned()
                .into_dimensionality::<Ix3>()
                .map_err(|e| anyhow!("failed to convert logits to Array3: {e}"))?;
            
            // 取最后一个 token 的 logits
            let logits_shape = logits_arr.shape();
            let _vocab_size = logits_shape[2];
            let seq_len = logits_shape[1];
            // 获取最后一个 token 的 logits (最后一个 seq_len 维度)
            let last_token_logits = logits_arr.slice(ndarray::s![0, seq_len - 1, ..]);

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

            // 更新 KV cache：从 outputs 中提取 present.* Value 并重新构建
            // 根据 recommended_kv_solution.md：KV cache 作为黑盒 Value 存储
            // 输出顺序：logits (0), present.0.decoder.key (1), present.0.decoder.value (2), ...
            // 对于第 i 层：present.{i}.decoder.key 在索引 1 + i*4, present.{i}.decoder.value 在 1 + i*4 + 1, ...
            // 注意：在 ort 1.16.3 中，outputs 是 Vec<Value<'static>>，所以我们可以直接使用
            // 但为了确保数据被正确复制，我们仍然需要提取并重新构建
            let mut new_kv_cache = Vec::new();
            
            for layer in 0..num_layers {
                let output_idx_base = 1 + layer * 4; // logits 是 0，所以从 1 开始
                
                // 提取每层的 4 个 KV cache 值：dec_k, dec_v, enc_k, enc_v
                // 注意：outputs 中的 Value 可能不能直接 clone，所以我们需要提取数据并重新构建
                // 根据 recommended_kv_solution.md：KV cache 作为黑盒 Value 处理
                use ort::tensor::OrtOwnedTensor;
                use ndarray::{IxDyn, CowArray};
                
                for offset in 0..4 {
                    let kv_output = &outputs[output_idx_base + offset];
                    // 提取 tensor 数据并重新构建 Value
                    // 注意：即使 use_cache_branch = false，模型仍然会输出 present.*，但可能某些值是空的
                    let kv_tensor: OrtOwnedTensor<f32, IxDyn> = kv_output
                        .try_extract::<f32>()
                        .map_err(|e| anyhow!("failed to extract present.{layer} KV cache (offset {}): {e}. This may happen if the tensor data pointer is null.", offset))?;
                    let kv_view = kv_tensor.view();
                    let kv_arr = kv_view.to_owned().into_dyn();
                    let kv_cow = CowArray::from(kv_arr);
                    let kv_value = Value::from_array(ptr::null_mut(), &kv_cow)
                        .map_err(|e| anyhow!("failed to rebuild present.{layer} KV cache: {e}"))?;
                    new_kv_cache.push(unsafe { std::mem::transmute::<Value<'_>, Value<'static>>(kv_value) });
                }
            }
            
            kv_cache = Some(new_kv_cache);
        }

        println!("Generated IDs: {:?} (length: {})", generated_ids, generated_ids.len());

        // 6. 使用 tokenizer 解码
        let translated_text = self.tokenizer.decode(&generated_ids);
        println!("Translated text: '{}'", translated_text);

        Ok(translated_text)
    }
}
