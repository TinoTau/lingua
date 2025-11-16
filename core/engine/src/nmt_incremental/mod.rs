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
use ort::value::Value;
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


/// 单句翻译时 Decoder 的状态
struct DecoderState {
    /// 当前 decoder 的 input_ids（最后一个 token 是本步要解码的）
    pub input_ids: Vec<i64>,
    /// 已经生成的 token IDs（不包括起始的 decoder_start_token_id）
    pub generated_ids: Vec<i64>,
    /// 上一步返回的 KV cache（present.*）
    /// - 每一层有 4 个 Value：decoder.key, decoder.value, encoder.key, encoder.value
    /// - `None` 代表第一步（没有历史 KV）
    pub kv_cache: Option<Vec<[Value<'static>; 4]>>,
    /// 控制 `use_cache_branch` 输入
    pub use_cache_branch: bool,
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
    // 模型常量
    const NUM_LAYERS: usize = 6;
    const NUM_HEADS: usize = 8;
    const HEAD_DIM: usize = 64;

    /// 构造第一步用的零张量 KV 值
    /// 
    /// # Arguments
    /// * `encoder_seq_len` - encoder 序列长度
    /// 
    /// # Returns
    /// 返回一个包含所有层的 KV cache 占位符，每层有 4 个 Value：dec_k, dec_v, enc_k, enc_v
    fn build_initial_kv_values(
        &self,
        encoder_seq_len: usize,
    ) -> anyhow::Result<Vec<[Value<'static>; 4]>> {
        use ndarray::Array4;
        use std::ptr;
        use ndarray::CowArray;

        let batch = 1usize;
        let dec_len = 1usize;           // decoder "历史长度"占位为 1
        let enc_len = encoder_seq_len;  // encoder 长度与真实输入一致

        let zeros_dec =
            Array4::<f32>::zeros((batch, Self::NUM_HEADS, dec_len, Self::HEAD_DIM));
        let zeros_enc =
            Array4::<f32>::zeros((batch, Self::NUM_HEADS, enc_len, Self::HEAD_DIM));

        // 使用与 decoder_step 中相同的 array_to_value 宏
        macro_rules! array_to_value {
            ($arr:expr) => {{
                let arr_dyn = $arr.into_dyn();
                let arr_owned = arr_dyn.to_owned();
                let cow_arr = CowArray::from(arr_owned);
                let value = Value::from_array(ptr::null_mut(), &cow_arr)
                    .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
                Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
            }};
        }

        let mut result = Vec::with_capacity(Self::NUM_LAYERS);

        for _ in 0..Self::NUM_LAYERS {
            // 每层有 4 个 KV：dec_k, dec_v, enc_k, enc_v
            let dec_k = array_to_value!(zeros_dec.clone())?;
            let dec_v = array_to_value!(zeros_dec.clone())?;
            let enc_k = array_to_value!(zeros_enc.clone())?;
            let enc_v = array_to_value!(zeros_enc.clone())?;
            result.push([dec_k, dec_v, enc_k, enc_v]);
        }

        Ok(result)
    }

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

    /// 执行 decoder 的单次步进
    ///
    /// - 输入：
    ///   - encoder_hidden_states: [1, encoder_seq_len, hidden_dim]
    ///   - encoder_attention_mask: [1, encoder_seq_len]
    ///   - state: 包含当前 decoder_input_ids / 上一步 KV cache
    /// - 输出：
    ///   - (logits_last_step, next_state)
    fn decoder_step(
        &self,
        encoder_hidden_states: &Array3<f32>,
        encoder_attention_mask: &Array2<i64>,
        mut state: DecoderState,
    ) -> anyhow::Result<(Array1<f32>, DecoderState)> {
        use std::ptr;
        use ndarray::CowArray;
        use ort::tensor::OrtOwnedTensor;

        // 打印调试信息
        println!(
            "[decoder_step] step input_ids_len={}, use_cache_branch={}, has_kv_cache={}",
            state.input_ids.len(),
            state.use_cache_branch,
            state.kv_cache.is_some(),
        );

        // 1. 准备 decoder input_ids: [1, cur_len]
        let batch_size = 1usize;
        let cur_len = state.input_ids.len();
        let decoder_input_ids = Array2::<i64>::from_shape_vec(
            (batch_size, cur_len),
            state.input_ids.clone(),
        )?;
        
        println!(
            "[decoder_step] input_ids shape: {:?}",
            decoder_input_ids.shape()
        );

        // 2. use_cache_branch: [1]，类型是 Bool（根据模型输入定义）
        let use_cache_array = Array1::<bool>::from_vec(vec![state.use_cache_branch]);

        // 3. 转换为 Value
        macro_rules! array_to_value {
            ($arr:expr, $ty:ty) => {{
                let arr_dyn = $arr.into_dyn();
                let arr_owned = arr_dyn.to_owned();
                let cow_arr = CowArray::from(arr_owned);
                let value = Value::from_array(ptr::null_mut(), &cow_arr)
                    .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
                Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
            }};
        }

        let input_ids_value = array_to_value!(decoder_input_ids, i64)?;
        let encoder_states_value = array_to_value!(encoder_hidden_states.clone(), f32)?;
        let encoder_mask_value = array_to_value!(encoder_attention_mask.clone(), i64)?;
        let use_cache_value = array_to_value!(use_cache_array, bool)?;

        // 4. 组织输入顺序（严格按照模型 I/O 顺序）
        // 输入顺序：encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.*, use_cache_branch
        let mut input_values: Vec<Value<'static>> = Vec::new();

        // 1. encoder_attention_mask
        input_values.push(encoder_mask_value);
        // 2. input_ids
        input_values.push(input_ids_value);
        // 3. encoder_hidden_states
        input_values.push(encoder_states_value);

        // 4. KV cache：准备输入 KV cache
        // 由于模型要求所有输入都存在，即使 use_cache_branch=false 也需要传入 KV
        let encoder_seq_len = encoder_hidden_states.shape()[1];
        // 保存旧的 KV cache（用于后续提取 encoder KV cache）
        let old_kv_for_encoder = if state.use_cache_branch && state.kv_cache.is_some() {
            // 正常模式：需要保存旧的 KV cache 以便提取 encoder KV cache
            // 注意：我们不能 clone Value，所以需要先保存，然后在构建输入时移动
            Some(state.kv_cache.as_ref().unwrap().clone())  // 这里会失败，因为 Value 不支持 Clone
        } else {
            None
        };
        
        let kv_to_use: Vec<[Value<'static>; 4]> = if state.use_cache_branch && state.kv_cache.is_some() {
            // 正常模式：使用历史 KV
            state.kv_cache.take().unwrap()
        } else {
            // 第一步或 Workaround 模式：使用占位 KV
            self.build_initial_kv_values(encoder_seq_len)?
        };

        // 无论第几步，都向模型提供完整的 KV 输入
        // 注意：我们需要在移动之前保存 encoder KV cache 的引用
        // 但由于 Value 不支持 Clone，我们需要使用不同的方法
        // 实际上，我们需要在处理 present.* 输出时，从旧的 KV cache 中获取 encoder KV
        // 所以我们需要在移动之前，先提取 encoder KV cache
        let saved_encoder_kv: Option<Vec<(Value<'static>, Value<'static>)>> = if state.use_cache_branch {
            // 从 kv_to_use 中提取 encoder KV cache（在移动之前）
            // 但是，由于 Value 不支持 Clone，我们不能这样做
            // 我们需要使用不同的方法：在处理 present.* 输出时，从旧的 KV cache 中获取
            None  // 暂时为 None，稍后从 kv_to_use 中提取
        } else {
            None
        };
        
        for kv_layer in kv_to_use {
            let [dec_k, dec_v, enc_k, enc_v] = kv_layer;
            input_values.push(dec_k);
            input_values.push(dec_v);
            input_values.push(enc_k);
            input_values.push(enc_v);
        }

        // 5. use_cache_branch
        input_values.push(use_cache_value);

        // 5. 调用 session.run
        let decoder_session = self.decoder_session.lock().unwrap();
        let outputs: Vec<Value<'static>> = decoder_session.run(input_values)
            .map_err(|e| anyhow!("failed to run decoder model: {e}"))?;

        // 6. 从输出中提取 logits + 新 KV
        // logits 是唯一需要转回 ndarray 的
        let mut iter = outputs.into_iter();
        let logits_value = iter.next().expect("missing logits output");

        let logits_tensor: OrtOwnedTensor<f32, IxDyn> = logits_value
            .try_extract::<f32>()
            .map_err(|e| anyhow!("failed to extract logits: {e}"))?;
        let logits_view = logits_tensor.view();
        let logits_array = logits_view.to_owned(); // shape: [1, cur_len, vocab_size]

        // 取最后一个 step 的 logits: [vocab_size]
        let shape = logits_array.shape();
        let seq_len = shape[1];
        // 使用 slice 获取最后一个 token 的 logits，然后转换为 Array1
        let last_step_logits = logits_array
            .slice(ndarray::s![0, seq_len - 1, ..])
            .to_owned(); // 已经是 Array1<f32>

        // KV cache：处理 present.* 输出
        // 关键发现：当 use_cache_branch=true 时，present.*.encoder.* 的第一个维度是 0
        // 我们不能使用这些空的 encoder KV cache，应该保持使用初始的 encoder KV cache
        if state.use_cache_branch {
            // 正常模式（第二步及以后）：只提取 decoder KV cache，保持 encoder KV cache 不变
            // 从 kv_to_use 中提取 encoder KV cache（保持不变）
            let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
            for (layer_idx, old_kv_layer) in kv_to_use.into_iter().enumerate() {
                let dec_k = iter.next().expect("missing present.*.decoder.key");
                let dec_v = iter.next().expect("missing present.*.decoder.value");
                iter.next(); // 跳过 present.*.encoder.key（use_cache_branch=true 时形状为 (0, 8, 1, 64)，不可用）
                iter.next(); // 跳过 present.*.encoder.value（use_cache_branch=true 时形状为 (0, 8, 1, 64)，不可用）
                
                // 从旧的 KV cache 中获取 encoder KV cache（保持不变）
                let [old_dec_k, old_dec_v, old_enc_k, old_enc_v] = old_kv_layer;
                
                // 只更新 decoder KV cache，保持 encoder KV cache 不变
                next_kv.push([dec_k, dec_v, old_enc_k, old_enc_v]);
            }
            state.kv_cache = Some(next_kv);
            state.use_cache_branch = true;  // 保持启用状态
        } else {
            // 第一步（use_cache_branch=false）：提取所有 KV cache
            // 这一步的 present.*.encoder.* 是正常的，可以全部提取
            let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
            for _layer in 0..Self::NUM_LAYERS {
                let dec_k = iter.next().expect("missing present.*.decoder.key");
                let dec_v = iter.next().expect("missing present.*.decoder.value");
                let enc_k = iter.next().expect("missing present.*.encoder.key");
                let enc_v = iter.next().expect("missing present.*.encoder.value");
                next_kv.push([dec_k, dec_v, enc_k, enc_v]);
            }
            state.kv_cache = Some(next_kv);
            state.use_cache_branch = true;  // 下一步启用 KV cache
        }

        // 返回 state（保持 generated_ids 不变，因为我们在 translate() 中管理它）
        Ok((last_step_logits, state))
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
        println!("Source text: '{}'", source_text);
        println!("Encoded source IDs: {:?} (length: {})", source_ids, source_ids.len());

        // 2. 运行 encoder 获取真实的 encoder_hidden_states
        let (encoder_hidden_states, encoder_attention_mask) = self.run_encoder(&source_ids)?;
        println!("Encoder output shape: {:?}", encoder_hidden_states.shape());

        // 3. 初始化 DecoderState
        // 第一步：不使用 KV cache，input_ids 只包含 BOS token
        let mut state = DecoderState {
            input_ids: vec![self.decoder_start_token_id],
            generated_ids: vec![self.decoder_start_token_id],  // 一开始就包含 BOS
            kv_cache: None,
            use_cache_branch: false,  // 第一步：禁用 KV 分支
        };

        // 4. 进入解码循环
        let max_steps = self.max_length.min(128); // 限制最大步数

        for step in 0..max_steps {
            // 准备当前步骤的 state
            // 关键：如果使用 KV cache，input_ids 应该只包含新 token（单个 token）
            // 如果不使用 KV cache（workaround），input_ids 包含完整历史序列
            let current_state = if state.use_cache_branch && state.kv_cache.is_some() {
                // 正常模式（使用 KV cache）：只输入新 token
                // 注意：这里应该使用上一步生成的最后一个 token
                let last_token = state.generated_ids.last().copied().unwrap_or(self.decoder_start_token_id);
                DecoderState {
                    input_ids: vec![last_token],  // 关键：只包含新 token
                    generated_ids: state.generated_ids.clone(),
                    kv_cache: state.kv_cache.take(),  // 使用历史 KV cache
                    use_cache_branch: true,  // 启用 KV 分支
                }
            } else {
                // Workaround 模式（不使用 KV cache）：使用完整历史序列
                let current_generated_ids = state.generated_ids.clone();
                DecoderState {
                    input_ids: current_generated_ids.clone(),  // 使用完整历史序列
                    generated_ids: current_generated_ids.clone(),
                    kv_cache: None,           // 不携带历史 KV
                    use_cache_branch: false,  // 禁用 KV 分支
                }
            };
            
            println!("[DEBUG] Step {}: decoder_input_ids={:?} (length: {}), use_cache_branch={}, has_kv_cache={}", 
                step, current_state.input_ids, current_state.input_ids.len(), 
                current_state.use_cache_branch, current_state.kv_cache.is_some());
            
            let (logits, next_state) = self.decoder_step(
                &encoder_hidden_states,
                &encoder_attention_mask,
                current_state,
            )?;

            // decoder_step 返回的 logits 已经是最后一个位置的 Array1<f32>
            // 所以直接使用即可
            
            // 选择概率最高的 token（贪婪解码）
            let next_token_id = logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx as i64)
                .ok_or_else(|| anyhow!("failed to find next token"))?;

            // 检查是否生成 EOS
            if next_token_id == self.eos_token_id {
                println!("Generated EOS token at step {}", step);
                break;
            }

            // 更新 state：添加新 token，并保存 KV cache（如果返回了）
            state.generated_ids.push(next_token_id);
            state.kv_cache = next_state.kv_cache;  // 保存 KV cache 供下一步使用
            state.use_cache_branch = next_state.use_cache_branch;  // 更新 use_cache_branch 状态
            
            println!("[DEBUG] After Step {}: use_cache_branch={}, has_kv_cache={}", 
                step, state.use_cache_branch, state.kv_cache.is_some());
        }

        println!("[NMT][translate] Generated IDs: {:?} (length: {})", state.generated_ids, state.generated_ids.len());

        // 5. 使用 tokenizer 解码（跳过 BOS token）
        let translated_ids: Vec<i64> = state.generated_ids.iter()
            .skip(1)  // 跳过 BOS token
            .copied()
            .collect();
        let translated_text = self.tokenizer.decode(&translated_ids);
        println!("[NMT][translate] Translated text: '{}'", translated_text);

        Ok(translated_text)
    }
}

/// 为 MarianNmtOnnx 实现 NmtIncremental trait
#[async_trait]
impl NmtIncremental for MarianNmtOnnx {
    async fn initialize(&self) -> EngineResult<()> {
        // ONNX 模型在 new_from_dir 时已经加载，这里只需要验证
        // 可以尝试运行一个简单的翻译来验证模型是否正常工作
        Ok(())
    }

    async fn translate(&self, request: TranslationRequest) -> EngineResult<TranslationResponse> {
        // 从 TranslationRequest 中提取源文本
        let source_text = request.transcript.text.clone();
        
        // 由于 self.translate() 是同步方法，但 trait 要求是 async，
        // 我们直接调用同步方法（虽然会阻塞当前任务，但对于翻译这种 CPU 密集型操作是合理的）
        let translated = self.translate(&source_text)
            .map_err(|e| {
                // 将 anyhow::Error 转换为 EngineError
                // String 可以转换为 Cow<'static, str>
                let error_msg = format!("Translation failed: {}", e);
                EngineError::new(error_msg)
            })?;

        Ok(TranslationResponse {
            translated_text: translated,
            is_stable: request.wait_k.is_none() || request.wait_k == Some(0),
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        // ONNX 会话会在对象销毁时自动清理
        Ok(())
    }
}
