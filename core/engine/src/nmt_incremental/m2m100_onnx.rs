use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use ort::session::Session;
use super::m2m100_tokenizer::M2M100Tokenizer;

pub struct M2M100NmtOnnx {
    pub encoder_session: std::sync::Mutex<Session>,
    pub decoder_session: std::sync::Mutex<Session>,
    pub tokenizer: M2M100Tokenizer,
    pub decoder_start_token_id: i64,
    pub eos_token_id: i64,
    pub pad_token_id: i64,
    pub max_length: usize,
    pub src_lang: String,
    pub tgt_lang: String,
    /// 模型是否使用新格式（没有 use_cache_branch 输入）
    pub use_new_format: bool,
}

impl M2M100NmtOnnx {
    // M2M100 模型常量
    pub(crate) const NUM_LAYERS: usize = 12;  // M2M100 有 12 层（Marian 是 6）
    pub(crate) const NUM_HEADS: usize = 16;   // M2M100 有 16 头（Marian 是 8）
    pub(crate) const HEAD_DIM: usize = 64;    // 每个头的维度（与 Marian 相同）
    pub(crate) const HIDDEN_SIZE: usize = 1024; // Encoder 输出维度（Marian 是 512）

    /// 从模型目录加载 M2M100NmtOnnx
    /// 
    /// # Arguments
    /// * `model_dir` - 模型目录路径（如 `models/nmt/m2m100-en-zh/`）
    /// 
    /// # Files Required
    /// - `encoder.onnx` - Encoder 模型
    /// - `decoder.onnx` - Decoder 模型
    /// - `vocab.json` - 词汇表
    /// - `sentencepiece.bpe.model` - SentencePiece 模型
    /// - `tokenizer_config.json` - Tokenizer 配置（可选）
    /// - `config.json` - 模型配置（可选）
    pub fn new_from_dir(model_dir: &Path) -> Result<Self> {
        // 1. 先初始化 ORT 环境
        crate::onnx_utils::init_onnx_runtime()?;

        // 2. 从目录名识别语言对（m2m100-en-zh 或 m2m100-zh-en）
        let dir_name = model_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid model directory name"))?;
        
        let (src_lang, tgt_lang) = if dir_name.contains("en-zh") {
            ("en", "zh")
        } else if dir_name.contains("zh-en") {
            ("zh", "en")
        } else {
            return Err(anyhow!(
                "Cannot determine language pair from directory name: {}. Expected 'm2m100-en-zh' or 'm2m100-zh-en'",
                dir_name
            ));
        };

        // 3. 加载 tokenizer
        let tokenizer = M2M100Tokenizer::from_model_dir(model_dir)?;

        // 4. 加载 encoder 模型
        let encoder_path = model_dir.join("encoder.onnx");
        if !encoder_path.exists() {
            return Err(anyhow!(
                "encoder.onnx not found at {}. Please export it first using docs/models/export_m2m100_encoder.py",
                encoder_path.display()
            ));
        }

        // ort 1.16.3: 使用文件模式创建 session
        use ort::{SessionBuilder, Environment};
        use std::sync::Arc;
        let env = Arc::new(
            Environment::builder()
                .with_name("m2m100_nmt")
                .build()?
        );
        
        let encoder_session = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create encoder Session builder: {e}"))?
            .with_model_from_file(&encoder_path)
            .map_err(|e| anyhow!("failed to load encoder ONNX model from {}: {e}", encoder_path.display()))?;

        println!("[OK] Encoder model loaded: {}", encoder_path.display());

        // 5. 加载 decoder 模型
        let decoder_path = model_dir.join("decoder.onnx");
        if !decoder_path.exists() {
            return Err(anyhow!(
                "decoder.onnx not found at {}. Please export it first using docs/models/export_m2m100_decoder_kv.py",
                decoder_path.display()
            ));
        }

        let decoder_session = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create decoder Session builder: {e}"))?
            .with_model_from_file(&decoder_path)
            .map_err(|e| anyhow!("failed to load decoder ONNX model from {}: {e}", decoder_path.display()))?;

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

        // 验证输入/输出数量
        // 注意：模型调整后，输入数量可能变化
        // 原来的期望：3 base + 48 KV + 1 flag = 52
        // 现在可能是：3 base + 48 KV = 51（移除了 use_cache_branch）
        let actual_inputs = decoder_session.inputs.len();
        let expected_inputs_old = 3 + (Self::NUM_LAYERS * 4) + 1; // 52
        let expected_inputs_new = 3 + (Self::NUM_LAYERS * 4); // 51
        
        if actual_inputs != expected_inputs_old && actual_inputs != expected_inputs_new {
            return Err(anyhow!(
                "Decoder model has {} inputs, expected {} (old format) or {} (new format). \
                Old format: 3 base + {} KV cache + 1 flag = {}. \
                New format: 3 base + {} KV cache = {}.",
                actual_inputs,
                expected_inputs_old,
                expected_inputs_new,
                Self::NUM_LAYERS * 4,
                expected_inputs_old,
                Self::NUM_LAYERS * 4,
                expected_inputs_new
            ));
        }
        
        // 检查是否移除了 use_cache_branch
        let has_use_cache_branch = decoder_session.inputs.iter()
            .any(|input| input.name.contains("use_cache") || input.name.contains("flag"));
        
        // 存储模型是否使用新格式（没有 use_cache_branch）
        let use_new_format = !has_use_cache_branch && actual_inputs == expected_inputs_new;
        
        if use_new_format {
            println!("[INFO] Model uses new format (without use_cache_branch flag)");
        }
        
        // 验证输出数量
        let expected_outputs = 1 + (Self::NUM_LAYERS * 4); // 1 logits + 48 KV = 49
        if decoder_session.outputs.len() != expected_outputs {
            return Err(anyhow!(
                "Decoder model has {} outputs, expected {} (1 logits + {} KV cache)",
                decoder_session.outputs.len(),
                expected_outputs,
                Self::NUM_LAYERS * 4
            ));
        }

        // 6. 从 config.json 读取配置（如果存在）
        let config_path = model_dir.join("config.json");
        let (decoder_start_token_id, eos_token_id, pad_token_id, max_length) = 
            if config_path.exists() {
                let config_str = fs::read_to_string(&config_path)
                    .map_err(|e| anyhow!("failed to read config.json: {e}"))?;
                let config: serde_json::Value = serde_json::from_str(&config_str)
                    .map_err(|e| anyhow!("failed to parse config.json: {e}"))?;
                
                (
                    config["decoder_start_token_id"].as_i64().unwrap_or(2), // </s> token
                    config["eos_token_id"].as_i64().unwrap_or(2),
                    config["pad_token_id"].as_i64().unwrap_or(1),
                    config["max_length"].as_u64().unwrap_or(1024) as usize,
                )
            } else {
                // 默认值（M2M100 的标准值）
                (2, 2, 1, 1024) // </s> = 2, <pad> = 1
            };

        Ok(Self { 
            encoder_session: std::sync::Mutex::new(encoder_session),
            decoder_session: std::sync::Mutex::new(decoder_session), 
            tokenizer,
            decoder_start_token_id,
            eos_token_id,
            pad_token_id,
            max_length,
            src_lang: src_lang.to_string(),
            tgt_lang: tgt_lang.to_string(),
            use_new_format,
        })
    }
}

// M2M100 Encoder 实现
impl M2M100NmtOnnx {
    /// 运行 encoder 模型，获取 encoder_hidden_states
    /// 
    /// # Arguments
    /// * `input_ids` - 编码后的输入 token IDs（包含语言 token）
    /// 
    /// # Returns
    /// (encoder_hidden_states, encoder_attention_mask)
    /// encoder_hidden_states shape: [1, seq_len, 1024] (M2M100 是 1024 维，Marian 是 512)
    pub(crate) fn run_encoder(&self, input_ids: &[i64]) -> Result<(ndarray::Array3<f32>, ndarray::Array2<i64>)> {
        use ort::tensor::OrtOwnedTensor;
        use ort::value::Value;
        use ndarray::{Array2, Array3, IxDyn, Ix3};

        let batch_size = 1usize;
        let seq_len = input_ids.len();

        // 准备输入
        let input_ids_array: Array2<i64> = Array2::from_shape_vec(
            (batch_size, seq_len),
            input_ids.to_vec(),
        )?;

        let attention_mask: Array2<i64> = Array2::ones((batch_size, seq_len));

        // 转换为 ONNX Value
        use std::ptr;
        use ndarray::CowArray;
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

        let input_ids_value: ort::Value<'static> = array_to_value!(input_ids_array, i64)?;
        let attention_mask_value: ort::Value<'static> = array_to_value!(attention_mask.clone(), i64)?;

        // 运行 encoder
        let encoder_session = self.encoder_session.lock().unwrap();
        let inputs = vec![input_ids_value, attention_mask_value];
        let outputs: Vec<ort::Value> = encoder_session.run(inputs)
            .map_err(|e| anyhow!("failed to run encoder model: {e}"))?;

        // 提取 encoder_hidden_states (last_hidden_state)
        let hidden_states_value = &outputs[0];

        // 提取为 Array3<f32>
        let tensor: OrtOwnedTensor<f32, IxDyn> = hidden_states_value
            .try_extract::<f32>()
            .map_err(|e| anyhow!("failed to extract encoder hidden states: {e}"))?;
        let view = tensor.view();
        let encoder_hidden_states: Array3<f32> = view
            .to_owned()
            .into_dimensionality::<Ix3>()
            .map_err(|e| anyhow!("failed to convert to Array3: {e}"))?;

        // 验证维度（M2M100 应该是 1024）
        let hidden_dim = encoder_hidden_states.shape()[2];
        if hidden_dim != Self::HIDDEN_SIZE {
            return Err(anyhow!(
                "Encoder hidden states dimension mismatch: expected {}, got {}",
                Self::HIDDEN_SIZE,
                hidden_dim
            ));
        }

        Ok((encoder_hidden_states, attention_mask))
    }
}

