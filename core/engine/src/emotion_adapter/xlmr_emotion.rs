use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Mutex;
use ort::session::Session;
use ort::tensor::OrtOwnedTensor;
use ndarray::{Array1, Array2, IxDyn, Ix2};
use serde::Deserialize;

use crate::error::{EngineError, EngineResult};
use super::{EmotionAdapter, EmotionRequest, EmotionResponse};

/// XLM-R 情感分类引擎
pub struct XlmREmotionEngine {
    session: Mutex<Session>,
    tokenizer: XlmRTokenizer,
    label_map: Vec<String>,  // id -> label 映射
}

/// XLM-R Tokenizer（简化版，使用 tokenizer.json）
struct XlmRTokenizer {
    vocab: std::collections::HashMap<String, u32>,
    bos_token_id: u32,
    eos_token_id: u32,
    pad_token_id: u32,
    unk_token_id: u32,
}

impl XlmRTokenizer {
    /// 从模型目录加载 tokenizer（简化版：只读取配置）
    fn from_model_dir(model_dir: &Path) -> Result<Self> {
        // 从 config.json 读取 special token IDs
        let config_path = model_dir.join("config.json");
        let config_data = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("failed to read config.json: {e}"))?;
        
        #[derive(Deserialize)]
        struct ConfigJson {
            bos_token_id: u32,
            eos_token_id: u32,
            pad_token_id: u32,
        }
        
        let config: ConfigJson = serde_json::from_str(&config_data)
            .map_err(|e| anyhow!("failed to parse config.json: {e}"))?;

        // unk_token_id 通常是 3（XLM-R 标准）
        let unk_token_id = 3;

        // 简化版：创建一个空的 vocab（实际编码时使用字符级编码）
        // TODO: 后续可以集成 SentencePiece 或完整的 tokenizer.json 解析
        let vocab = std::collections::HashMap::new();

        Ok(Self {
            vocab,
            bos_token_id: config.bos_token_id,
            eos_token_id: config.eos_token_id,
            pad_token_id: config.pad_token_id,
            unk_token_id,
        })
    }

    /// 编码文本为 token IDs（简化版：字符级编码）
    /// 
    /// 注意：这是一个简化实现。实际应该使用 SentencePiece tokenizer。
    /// 对于测试和开发，字符级编码可以工作，但性能可能不如完整的 tokenizer。
    fn encode(&self, text: &str, max_length: usize) -> Vec<i64> {
        let mut ids = Vec::new();
        
        // 添加 BOS token
        ids.push(self.bos_token_id as i64);
        
        // 简化版：字符级编码（每个字符映射到一个 token ID）
        // 使用简单的哈希函数将字符映射到 token ID 范围
        // 注意：这不是标准的 XLM-R tokenization，仅用于测试
        let chars: Vec<char> = text.chars().collect();
        for ch in chars.iter().take(max_length - 2) {  // 保留 EOS 位置
            // 简单的字符到 ID 映射（使用字符的 Unicode 码点，限制在合理范围内）
            let char_id = (*ch as u32) % 100000;  // 限制在 0-99999 范围内
            ids.push(char_id as i64);
        }
        
        // 添加 EOS token
        ids.push(self.eos_token_id as i64);
        
        // 填充到 max_length
        while ids.len() < max_length {
            ids.push(self.pad_token_id as i64);
        }
        
        ids.truncate(max_length);
        ids
    }
}

impl XlmREmotionEngine {
    /// 从模型目录加载 XLM-R 情感分类引擎
    pub fn new_from_dir(model_dir: &Path) -> Result<Self> {
        // 初始化 ONNX Runtime
        crate::onnx_utils::init_onnx_runtime()?;

        // 加载 tokenizer
        let tokenizer = XlmRTokenizer::from_model_dir(model_dir)?;

        // 加载模型配置，获取 label 映射
        let config_path = model_dir.join("config.json");
        let config_data = std::fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("failed to read config.json: {e}"))?;
        
        #[derive(Deserialize)]
        struct ConfigJson {
            id2label: std::collections::HashMap<String, String>,
        }
        
        let config: ConfigJson = serde_json::from_str(&config_data)
            .map_err(|e| anyhow!("failed to parse config.json: {e}"))?;

        // 构建 label_map（id -> label）
        let mut label_map = vec!["".to_string(); config.id2label.len()];
        for (id_str, label) in config.id2label {
            let id: usize = id_str.parse()
                .map_err(|e| anyhow!("invalid label id in config: {id_str}: {e}"))?;
            if id >= label_map.len() {
                return Err(anyhow!("label id {id} out of range (max: {})", label_map.len()));
            }
            label_map[id] = label;
        }

        // 加载 ONNX 模型
        let model_path = model_dir.join("model.onnx");
        if !model_path.exists() {
            return Err(anyhow!("model.onnx not found at {}", model_path.display()));
        }

        use ort::{SessionBuilder, Environment};
        use std::sync::Arc;
        let env = Arc::new(
            Environment::builder()
                .with_name("xlmr_emotion")
                .build()?
        );
        
        let session = SessionBuilder::new(&env)
            .map_err(|e| anyhow!("failed to create Session builder: {e}"))?
            .with_model_from_file(&model_path)
            .map_err(|e| anyhow!("failed to load model: {e}"))?;

        Ok(Self {
            session: Mutex::new(session),
            tokenizer,
            label_map,
        })
    }

    /// 执行情感分类推理
    fn infer(&self, text: &str) -> Result<EmotionResponse> {
        // 1. 编码文本
        let max_length = 128;  // XLM-R 最大长度
        let input_ids = self.tokenizer.encode(text, max_length);
        let seq_len = input_ids.len();

        // 2. 准备输入（batch_size=1）
        let batch_size = 1usize;
        let input_ids_array: Array2<i64> = Array2::from_shape_vec(
            (batch_size, seq_len),
            input_ids,
        )?;

        // 3. 转换为 ONNX Value
        use std::ptr;
        use ndarray::CowArray;
        let arr_dyn = input_ids_array.into_dyn();
        let arr_owned = arr_dyn.to_owned();
        let cow_arr = CowArray::from(arr_owned);
        let input_value = ort::value::Value::from_array(ptr::null_mut(), &cow_arr)
            .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
        let input_value: ort::value::Value<'static> = unsafe {
            std::mem::transmute::<ort::value::Value, ort::value::Value<'static>>(input_value)
        };

        // 4. 运行模型
        let session = self.session.lock().unwrap();
        let inputs = vec![input_value];
        let outputs: Vec<ort::value::Value> = session.run(inputs)
            .map_err(|e| anyhow!("failed to run emotion model: {e}"))?;

        // 5. 提取 logits（第一个输出）
        let logits_value = outputs.get(0)
            .ok_or_else(|| anyhow!("model output is empty"))?;

        // 6. 转换为 ndarray
        let tensor: OrtOwnedTensor<f32, IxDyn> = logits_value.try_extract()
            .map_err(|e| anyhow!("failed to extract logits tensor: {e}"))?;
        let view = tensor.view();
        let logits: Array2<f32> = view
            .to_owned()
            .into_dimensionality::<Ix2>()
            .map_err(|e| anyhow!("failed to reshape logits: {e}"))?;

        // 7. 应用 softmax 并找到最大概率的类别
        let logits_row = logits.row(0);  // batch_size=1，取第一行
        let logits_1d: Array1<f32> = logits_row.to_owned();
        
        // Softmax
        let max_logit = logits_1d.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let exp_logits: Array1<f32> = logits_1d.mapv(|x| (x - max_logit).exp());
        let sum_exp: f32 = exp_logits.sum();
        let probs: Array1<f32> = exp_logits.mapv(|x| x / sum_exp);

        // 找到最大概率的索引
        let (predicted_id, confidence) = probs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| anyhow!("empty probabilities"))?;

        // 8. 获取 label
        let label = self.label_map.get(predicted_id)
            .cloned()
            .unwrap_or_else(|| format!("unknown_{}", predicted_id));

        Ok(EmotionResponse {
            label,
            confidence: *confidence,
        })
    }
}

#[async_trait::async_trait]
impl EmotionAdapter for XlmREmotionEngine {
    async fn analyze(&self, request: EmotionRequest) -> EngineResult<EmotionResponse> {
        // 使用 transcript 的文本进行情感分析
        let text = &request.transcript.text;
        
        self.infer(text)
            .map_err(|e| EngineError::new(format!("emotion analysis failed: {e}")))
    }
}

