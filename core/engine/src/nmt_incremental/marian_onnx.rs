use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use ort::session::Session;
use super::tokenizer::MarianTokenizer;
use super::language_pair::LanguagePair;

// 子模块在 mod.rs 中声明，这里直接使用
use super::decoder_state::DecoderState;

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
    pub(crate) const NUM_LAYERS: usize = 6;
    pub(crate) const NUM_HEADS: usize = 8;
    pub(crate) const HEAD_DIM: usize = 64;

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
}
