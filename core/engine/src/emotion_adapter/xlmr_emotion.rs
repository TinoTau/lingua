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

/// XLM-R Tokenizer（使用 tokenizers crate）
struct XlmRTokenizer {
    tokenizer: tokenizers::Tokenizer,
    max_length: usize,
}

impl XlmRTokenizer {
    /// 从模型目录加载 tokenizer
    fn from_model_dir(model_dir: &Path) -> Result<Self> {
        let tokenizer_path = model_dir.join("tokenizer.json");
        if !tokenizer_path.exists() {
            return Err(anyhow!("tokenizer.json not found at {}", tokenizer_path.display()));
        }

        // 使用 tokenizers crate 加载 tokenizer
        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("failed to load tokenizer from {}: {e}", tokenizer_path.display()))?;

        // XLM-R 最大长度通常是 514，但我们使用 128 以节省计算
        let max_length = 128;

        Ok(Self {
            tokenizer,
            max_length,
        })
    }

    /// 编码文本为 token IDs
    fn encode(&self, text: &str) -> Vec<i64> {
        // 使用 tokenizer 进行编码
        let encoding = match self.tokenizer.encode(text, true) {
            Ok(enc) => enc,
            Err(_) => {
                // 如果编码失败，返回空向量
                return vec![];
            }
        };

        let mut ids: Vec<i64> = encoding.get_ids()
            .iter()
            .map(|&id| id as i64)
            .collect();

        // 截断或填充到 max_length
        if ids.len() > self.max_length {
            ids.truncate(self.max_length);
        } else {
            // 获取 pad_token_id（从 config.json 读取，默认使用 1）
            // 注意：tokenizers crate 的 get_vocab 可能返回临时值，所以我们使用默认值
            let pad_token_id = 1i64;  // XLM-R 的 pad_token_id 通常是 1

            // 填充到 max_length
            while ids.len() < self.max_length {
                ids.push(pad_token_id);
            }
        }

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

        // 加载 ONNX 模型（优先使用 IR 9 版本）
        let model_path = if model_dir.join("model_ir9.onnx").exists() {
            model_dir.join("model_ir9.onnx")
        } else {
            model_dir.join("model.onnx")
        };
        
        if !model_path.exists() {
            return Err(anyhow!("model.onnx or model_ir9.onnx not found at {}", model_dir.display()));
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
        let input_ids = self.tokenizer.encode(text);
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

        // 4. 准备 attention_mask（XLM-R 模型需要）
        let attention_mask: Array2<i64> = Array2::ones((batch_size, seq_len));
        let arr_dyn_mask = attention_mask.into_dyn();
        let arr_owned_mask = arr_dyn_mask.to_owned();
        let cow_arr_mask = CowArray::from(arr_owned_mask);
        let attention_mask_value = ort::value::Value::from_array(ptr::null_mut(), &cow_arr_mask)
            .map_err(|e| anyhow!("failed to convert attention_mask to Value: {:?}", e))?;
        let attention_mask_value: ort::value::Value<'static> = unsafe {
            std::mem::transmute::<ort::value::Value, ort::value::Value<'static>>(attention_mask_value)
        };

        // 5. 运行模型
        let session = self.session.lock().unwrap();
        let inputs = vec![input_value, attention_mask_value];
        let outputs: Vec<ort::value::Value> = session.run(inputs)
            .map_err(|e| anyhow!("failed to run emotion model: {e}"))?;

        // 6. 提取 logits（第一个输出）
        let logits_value = outputs.get(0)
            .ok_or_else(|| anyhow!("model output is empty"))?;

        // 7. 转换为 ndarray
        let tensor: OrtOwnedTensor<f32, IxDyn> = logits_value.try_extract()
            .map_err(|e| anyhow!("failed to extract logits tensor: {e}"))?;
        let view = tensor.view();
        let logits: Array2<f32> = view
            .to_owned()
            .into_dimensionality::<Ix2>()
            .map_err(|e| anyhow!("failed to reshape logits: {e}"))?;

        // 8. 应用 softmax 并找到最大概率的类别
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

        // 9. 获取 label
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

