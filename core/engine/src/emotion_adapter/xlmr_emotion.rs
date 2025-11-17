use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Mutex;
use ort::session::Session;
use ort::tensor::OrtOwnedTensor;
use ndarray::{Array1, Array2, IxDyn, Ix2};
use serde::Deserialize;

use crate::error::{EngineError, EngineResult};
use super::{EmotionAdapter, EmotionRequest, EmotionResponse};

/// 标准化情绪标签名称
/// 
/// 根据 Emotion_Adapter_Spec.md，标准情绪为：
/// "neutral" | "joy" | "sadness" | "anger" | "fear" | "surprise"
fn normalize_emotion_label(label: &str) -> String {
    let label_lower = label.to_lowercase();
    
    // 映射常见的情绪标签变体到标准格式
    match label_lower.as_str() {
        "positive" | "happy" | "happiness" | "joy" | "joyful" => "joy".to_string(),
        "negative" | "sad" | "sadness" | "sorrow" => "sadness".to_string(),
        "angry" | "anger" | "rage" => "anger".to_string(),
        "fear" | "afraid" | "scared" => "fear".to_string(),
        "surprise" | "surprised" | "shock" => "surprise".to_string(),
        "neutral" | "none" | "normal" => "neutral".to_string(),
        _ => {
            // 如果无法识别，尝试从 label 中提取关键词
            if label_lower.contains("positive") || label_lower.contains("joy") || label_lower.contains("happy") {
                "joy".to_string()
            } else if label_lower.contains("negative") || label_lower.contains("sad") {
                "sadness".to_string()
            } else if label_lower.contains("angry") || label_lower.contains("anger") {
                "anger".to_string()
            } else if label_lower.contains("fear") || label_lower.contains("afraid") {
                "fear".to_string()
            } else if label_lower.contains("surprise") || label_lower.contains("shock") {
                "surprise".to_string()
            } else {
                // 默认返回 neutral
                "neutral".to_string()
            }
        }
    }
}

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

        // 加载 ONNX 模型（优先使用 PyTorch 1.13 导出的 IR 9 版本）
        let model_path = if model_dir.join("model_ir9_pytorch13.onnx").exists() {
            model_dir.join("model_ir9_pytorch13.onnx")
        } else if model_dir.join("model_ir9.onnx").exists() {
            model_dir.join("model_ir9.onnx")
        } else {
            model_dir.join("model.onnx")
        };
        
        if !model_path.exists() {
            return Err(anyhow!("model.onnx, model_ir9.onnx, or model_ir9_pytorch13.onnx not found at {}", model_dir.display()));
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
    /// 
    /// 根据 Emotion_Adapter_Spec.md 实现后处理规则：
    /// - 文本过短（< 3 字符）→ 强制 neutral
    /// - logits 差值过小（< 0.1）→ neutral
    /// - confidence = softmax(top1)
    fn infer(&self, text: &str) -> Result<EmotionResponse> {
        // 0. 后处理规则：文本过短 → 强制 neutral
        let text_trimmed = text.trim();
        if text_trimmed.len() < 3 {
            return Ok(EmotionResponse {
                primary: "neutral".to_string(),
                intensity: 0.0,
                confidence: 1.0,
            });
        }
        
        // 1. 编码文本
        let input_ids = self.tokenizer.encode(text_trimmed);
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

        // 找到最大概率的索引和次大概率
        let mut indexed_probs: Vec<(usize, f32)> = probs
            .iter()
            .enumerate()
            .map(|(i, &p)| (i, p))
            .collect();
        indexed_probs.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        
        let (predicted_id, top1_prob) = indexed_probs[0];
        let top2_prob = indexed_probs.get(1).map(|(_, p)| *p).unwrap_or(0.0);
        
        // 后处理规则：logits 差值过小（< 0.1）→ neutral
        let prob_diff = top1_prob - top2_prob;
        let primary = if prob_diff < 0.1 {
            "neutral".to_string()
        } else {
            // 获取 label 并标准化为规范格式
            let label = self.label_map.get(predicted_id)
                .cloned()
                .unwrap_or_else(|| format!("unknown_{}", predicted_id));
            
            // 标准化 label 名称（根据 Emotion_Adapter_Spec.md）
            normalize_emotion_label(&label)
        };
        
        // intensity 使用 top1 概率，confidence 也使用 top1 概率
        let intensity = top1_prob;
        let confidence = top1_prob;

        Ok(EmotionResponse {
            primary,
            intensity,
            confidence,
        })
    }
}

#[async_trait::async_trait]
impl EmotionAdapter for XlmREmotionEngine {
    async fn analyze(&self, request: EmotionRequest) -> EngineResult<EmotionResponse> {
        // 根据 Emotion_Adapter_Spec.md，直接使用 text 和 lang
        // 注意：lang 参数目前未使用，但保留在接口中以便未来扩展
        let _lang = &request.lang;
        let text = &request.text;
        
        self.infer(text)
            .map_err(|e| EngineError::new(format!("emotion analysis failed: {e}")))
    }
}

