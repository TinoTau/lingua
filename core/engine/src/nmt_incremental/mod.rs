use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{EngineError, EngineResult};
use crate::types::PartialTranscript;

use anyhow::{Result, anyhow};
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;
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
        // ort 1.16.3 使用 from_array 需要 allocator 和 array
        use std::ptr;
        macro_rules! array_to_value {
            ($arr:expr, $ty:ty) => {{
                // 使用 null allocator（让 ORT 使用默认 allocator）
                Value::from_array(ptr::null_mut(), &$arr)
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
        // ort 1.16.3: 当使用手动构建的 inputs 时，outputs 是 Vec<Value>
        // encoder 只有一个输出 last_hidden_state，索引为 0
        let hidden_states_value = &outputs[0];

        // ort 1.16.3 使用 try_extract 返回 OrtOwnedTensor
        let hidden_states_tensor = hidden_states_value
            .try_extract::<f32>()
            .map_err(|e| anyhow!("failed to extract encoder hidden states: {e}"))?;

        // OrtOwnedTensor 需要手动提取 shape 和 data 来构建 Array3
        // 先转换为 ArrayBase，然后手动提取数据
        let tensor_array: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, _> = hidden_states_tensor.into();
        // 获取 shape 和 data
        let shape = tensor_array.shape();
        let data: Vec<f32> = tensor_array.iter().cloned().collect();
        // 手动构建 Array3
        let encoder_hidden_states = Array3::from_shape_vec(
            (shape[0], shape[1], shape[2]),
            data,
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
        let num_layers = 6usize;

        // 4. 创建 DecoderState 来存储 KV cache（作为 Value 黑盒）
        use ort::value::Value;
        
        struct DecoderState {
            // 每一层的 KV cache 都保存成 ort::Value，不做解码
            layer_kv: Vec<(Value, Value, Value, Value)>, // (dec_k, dec_v, enc_k, enc_v)
        }
        
        impl DecoderState {
            fn new_empty(batch: usize, num_heads: usize, enc_len: usize, head_dim: usize, layers: usize) -> Result<Self> {
                fn empty_kv(batch: usize, num_heads: usize, seq_len: usize, head_dim: usize) -> Result<Value> {
                    let actual_len = seq_len.max(1);
                    let arr = Array4::<f32>::zeros((batch, num_heads, actual_len, head_dim));
                    // ort 1.16.3 使用 from_array 需要 allocator 和动态维度的 array
                    // 使用 null allocator（让 ORT 使用默认 allocator）
                    // 需要转换为 CowRepr 类型
                    use ndarray::CowArray;
                    let arr_dyn = arr.into_dyn();
                    let cow_arr = CowArray::from(arr_dyn);
                    let value = Value::from_array(ptr::null_mut(), &cow_arr)
                        .map_err(|e| anyhow!("failed to create empty KV cache Value: {e}"))?;
                    Ok(value)
                }
                
                let mut layer_kv = Vec::new();
                for _ in 0..layers {
                    layer_kv.push((
                        empty_kv(batch, num_heads, 0, head_dim)?,      // dec_k
                        empty_kv(batch, num_heads, 0, head_dim)?,      // dec_v
                        empty_kv(batch, num_heads, enc_len, head_dim)?, // enc_k
                        empty_kv(batch, num_heads, enc_len, head_dim)?, // enc_v
                    ));
                }
                Ok(Self { layer_kv })
            }
        }
        
        let mut decoder_state = DecoderState::new_empty(batch_size, num_heads, encoder_seq_len, head_dim, num_layers)?;

        // 5. 增量解码循环
        // ort 1.16.3 使用 from_array 需要 allocator 和动态维度的 array
        use ndarray::CowArray;
        macro_rules! array_to_value {
            ($arr:expr, $t:ty) => {{
                // 转换为动态维度和 CowRepr 类型，使用 null allocator（让 ORT 使用默认 allocator）
                let arr_dyn = $arr.into_dyn();
                let cow_arr = CowArray::from(arr_dyn);
                Value::from_array(ptr::null_mut(), &cow_arr)
                    .map_err(|e| anyhow!("failed to convert array to Value: {e}"))
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

            // 准备所有输入（在循环中需要 clone，因为每次迭代都会使用）
            let encoder_attention_mask_value = array_to_value!(encoder_attention_mask.clone(), i64)?;
            let input_ids_value = array_to_value!(decoder_input, i64)?;
            let encoder_hidden_states_value = array_to_value!(encoder_hidden_states.clone(), f32)?;
            let use_cache_branch_value = array_to_value!(Array1::from_vec(vec![true]), bool)?;

            // 准备 past_key_values（黑盒方式：直接把 state.layer_kv 里的 Value 原样喂回去）
            use std::borrow::Cow;
            use ort::session::SessionInputValue;

            // 构建输入 - 使用 ort::inputs! 宏（ort 1.16.3 需要）
            // 先准备所有 past_key_values
            let mut past_kv_values = Vec::new();
            for (layer_idx, (dec_k, dec_v, enc_k, enc_v)) in decoder_state.layer_kv.iter().enumerate() {
                let i = layer_idx;
                
                // 提取数据并重新构建 Value（黑盒：只是搬运数据）
                // ort 1.16.3 使用 try_extract 返回 OrtOwnedTensor
                let dec_k_tensor = dec_k
                    .try_extract::<f32>()
                    .map_err(|e| anyhow!("failed to extract past_key_values.{i}.decoder.key: {e}"))?;
                let dec_v_tensor = dec_v
                    .try_extract::<f32>()
                    .map_err(|e| anyhow!("failed to extract past_key_values.{i}.decoder.value: {e}"))?;
                let enc_k_tensor = enc_k
                    .try_extract::<f32>()
                    .map_err(|e| anyhow!("failed to extract past_key_values.{i}.encoder.key: {e}"))?;
                let enc_v_tensor = enc_v
                    .try_extract::<f32>()
                    .map_err(|e| anyhow!("failed to extract past_key_values.{i}.encoder.value: {e}"))?;

                // OrtOwnedTensor 实现了 Into<ArrayBase>，先转换为 ArrayBase
                let dec_k_arr_dyn: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<ndarray::IxDyn>> = dec_k_tensor.into();
                let dec_k_arr = dec_k_arr_dyn
                    .into_dimensionality::<ndarray::Ix4>()
                    .map_err(|e| anyhow!("failed to convert dec_k to Array4: {e}"))?;
                let dec_v_arr_dyn: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<ndarray::IxDyn>> = dec_v_tensor.into();
                let dec_v_arr = dec_v_arr_dyn
                    .into_dimensionality::<ndarray::Ix4>()
                    .map_err(|e| anyhow!("failed to convert dec_v to Array4: {e}"))?;
                let enc_k_arr_dyn: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<ndarray::IxDyn>> = enc_k_tensor.into();
                let enc_k_arr = enc_k_arr_dyn
                    .into_dimensionality::<ndarray::Ix4>()
                    .map_err(|e| anyhow!("failed to convert enc_k to Array4: {e}"))?;
                let enc_v_arr_dyn: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<ndarray::IxDyn>> = enc_v_tensor.into();
                let enc_v_arr = enc_v_arr_dyn
                    .into_dimensionality::<ndarray::Ix4>()
                    .map_err(|e| anyhow!("failed to convert enc_v to Array4: {e}"))?;

                // 转换为动态维度和 CowRepr 类型
                use ndarray::CowArray;
                let dec_k_value = Value::from_array(ptr::null_mut(), &CowArray::from(dec_k_arr.into_dyn()))
                    .map_err(|e| anyhow!("failed to rebuild past_key_values.{i}.decoder.key: {e}"))?;
                let dec_v_value = Value::from_array(ptr::null_mut(), &CowArray::from(dec_v_arr.into_dyn()))
                    .map_err(|e| anyhow!("failed to rebuild past_key_values.{i}.decoder.value: {e}"))?;
                let enc_k_value = Value::from_array(ptr::null_mut(), &CowArray::from(enc_k_arr.into_dyn()))
                    .map_err(|e| anyhow!("failed to rebuild past_key_values.{i}.encoder.key: {e}"))?;
                let enc_v_value = Value::from_array(ptr::null_mut(), &CowArray::from(enc_v_arr.into_dyn()))
                    .map_err(|e| anyhow!("failed to rebuild past_key_values.{i}.encoder.value: {e}"))?;

                past_kv_values.push((i, dec_k_value, dec_v_value, enc_k_value, enc_v_value));
            }
            
            // 使用 ort::inputs! 宏构建输入（ort 1.16.3 需要）
            // 注意：ort::inputs! 宏需要静态字符串，所以我们需要手动构建
            // 但为了兼容性，我们仍然使用手动构建的 Vec
            let mut inputs_vec: Vec<(Cow<'_, str>, SessionInputValue<'_>)> = vec![
                ("encoder_attention_mask".into(), encoder_attention_mask_value.into()),
                ("input_ids".into(), input_ids_value.into()),
                ("encoder_hidden_states".into(), encoder_hidden_states_value.into()),
                ("use_cache_branch".into(), use_cache_branch_value.into()),
            ];
            
            for (i, dec_k, dec_v, enc_k, enc_v) in past_kv_values {
                inputs_vec.push((Cow::Owned(format!("past_key_values.{i}.decoder.key")), dec_k.into()));
                inputs_vec.push((Cow::Owned(format!("past_key_values.{i}.decoder.value")), dec_v.into()));
                inputs_vec.push((Cow::Owned(format!("past_key_values.{i}.encoder.key")), enc_k.into()));
                inputs_vec.push((Cow::Owned(format!("past_key_values.{i}.encoder.value")), enc_v.into()));
            }
            
            let inputs = inputs_vec;

            // 运行模型
            let mut decoder_session = self.decoder_session.lock().unwrap();
            let outputs = decoder_session.run(inputs)
                .map_err(|e| anyhow!("failed to run decoder model: {e}"))?;

            // 提取 logits
            // ort 1.16.3: 当使用手动构建的 inputs 时，outputs 是 Vec<Value>
            // 需要根据输出顺序访问，logits 是第一个输出（索引 0）
            let logits_value = &outputs[0];
            // ort 1.16.3 使用 try_extract 返回 OrtOwnedTensor
            let logits_tensor = logits_value
                .try_extract::<f32>()
                .map_err(|e| anyhow!("failed to extract logits: {e}"))?;

            // OrtOwnedTensor 实现了 Into<ArrayBase>，先转换为 ArrayBase
            // logits shape: [batch, seq_len, vocab_size]
            let logits_arr_dyn: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<ndarray::IxDyn>> = logits_tensor.into();
            let logits_arr = logits_arr_dyn
                .into_dimensionality::<ndarray::Ix3>()
                .map_err(|e| anyhow!("failed to convert logits to Array3: {e}"))?;
            
            // 取最后一个 token 的 logits
            let logits_shape = logits_arr.shape();
            let vocab_size = logits_shape[2];
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
            // 注意：由于 Value 不支持 Clone，我们需要提取数据并重新构建 Value
            // 但这是"黑盒"处理：我们只是搬运数据，不关心具体值
            for (layer_idx, kv) in decoder_state.layer_kv.iter_mut().enumerate() {
                let i = layer_idx;
                let dec_key_name = format!("present.{i}.decoder.key");
                let dec_val_name = format!("present.{i}.decoder.value");
                let enc_key_name = format!("present.{i}.encoder.key");
                let enc_val_name = format!("present.{i}.encoder.value");
                
                // 提取数据并重新构建 Value（黑盒：只是搬运数据）
                // ort 1.16.3: outputs 是 Vec<Value>，需要根据输出顺序访问
                // 输出顺序：logits (0), present.0.decoder.key (1), present.0.decoder.value (2), ...
                // 对于第 i 层：present.{i}.decoder.key 在索引 1 + i*4, present.{i}.decoder.value 在 1 + i*4 + 1, ...
                let output_idx_base = 1 + i * 4; // logits 是 0，所以从 1 开始
                // ort 1.16.3 使用 try_extract 返回 OrtOwnedTensor
                let dec_key_tensor = outputs[output_idx_base]
                    .try_extract::<f32>()
                    .map_err(|e| anyhow!("failed to extract present.{i}.decoder.key: {e}"))?;
                let dec_val_tensor = outputs[output_idx_base + 1]
                    .try_extract::<f32>()
                    .map_err(|e| anyhow!("failed to extract present.{i}.decoder.value: {e}"))?;
                let enc_key_tensor = outputs[output_idx_base + 2]
                    .try_extract::<f32>()
                    .map_err(|e| anyhow!("failed to extract present.{i}.encoder.key: {e}"))?;
                let enc_val_tensor = outputs[output_idx_base + 3]
                    .try_extract::<f32>()
                    .map_err(|e| anyhow!("failed to extract present.{i}.encoder.value: {e}"))?;

                // OrtOwnedTensor 实现了 Into<ArrayBase>，先转换为 ArrayBase
                let dec_key_arr_dyn: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<ndarray::IxDyn>> = dec_key_tensor.into();
                let dec_key_arr = dec_key_arr_dyn
                    .into_dimensionality::<ndarray::Ix4>()
                    .map_err(|e| anyhow!("failed to convert dec_key to Array4: {e}"))?;
                let dec_val_arr_dyn: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<ndarray::IxDyn>> = dec_val_tensor.into();
                let dec_val_arr = dec_val_arr_dyn
                    .into_dimensionality::<ndarray::Ix4>()
                    .map_err(|e| anyhow!("failed to convert dec_val to Array4: {e}"))?;
                let enc_key_arr_dyn: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<ndarray::IxDyn>> = enc_key_tensor.into();
                let enc_key_arr = enc_key_arr_dyn
                    .into_dimensionality::<ndarray::Ix4>()
                    .map_err(|e| anyhow!("failed to convert enc_key to Array4: {e}"))?;
                let enc_val_arr_dyn: ndarray::ArrayBase<ndarray::OwnedRepr<f32>, ndarray::Dim<ndarray::IxDyn>> = enc_val_tensor.into();
                let enc_val_arr = enc_val_arr_dyn
                    .into_dimensionality::<ndarray::Ix4>()
                    .map_err(|e| anyhow!("failed to convert enc_val to Array4: {e}"))?;

                // 转换为动态维度和 CowRepr 类型
                use ndarray::CowArray;
                kv.0 = Value::from_array(ptr::null_mut(), &CowArray::from(dec_key_arr.into_dyn()))
                    .map_err(|e| anyhow!("failed to rebuild present.{i}.decoder.key: {e}"))?;
                kv.1 = Value::from_array(ptr::null_mut(), &CowArray::from(dec_val_arr.into_dyn()))
                    .map_err(|e| anyhow!("failed to rebuild present.{i}.decoder.value: {e}"))?;
                kv.2 = Value::from_array(ptr::null_mut(), &CowArray::from(enc_key_arr.into_dyn()))
                    .map_err(|e| anyhow!("failed to rebuild present.{i}.encoder.key: {e}"))?;
                kv.3 = Value::from_array(ptr::null_mut(), &CowArray::from(enc_val_arr.into_dyn()))
                    .map_err(|e| anyhow!("failed to rebuild present.{i}.encoder.value: {e}"))?;
            }
        }

        println!("Generated IDs: {:?} (length: {})", generated_ids, generated_ids.len());

        // 6. 使用 tokenizer 解码
        let translated_text = self.tokenizer.decode(&generated_ids);
        println!("Translated text: '{}'", translated_text);

        Ok(translated_text)
    }
}
