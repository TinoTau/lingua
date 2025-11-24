use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokenizers::models::bpe::BPE;
use tokenizers::Tokenizer;

/// M2M100 Tokenizer 实现
/// 
/// 使用 HuggingFace tokenizers crate 从 sentencepiece.bpe.model 和 vocab.json 构建 tokenizer
/// 支持语言 token 和编码/解码功能
pub struct M2M100Tokenizer {
    /// HuggingFace tokenizer 实例
    tokenizer: Tokenizer,
    /// 语言代码到 token ID 的映射（如 "en" -> 128022）
    lang_id_map: HashMap<String, i64>,
    /// Pad token ID
    pad_token_id: i64,
    /// EOS token ID
    eos_token_id: i64,
}

/// tokenizer_config.json 结构
#[derive(Debug, Deserialize)]
struct TokenizerConfig {
    #[serde(default)]
    bos_token: Option<String>,
    #[serde(default)]
    eos_token: Option<String>,
    #[serde(default)]
    pad_token: Option<String>,
    #[serde(default)]
    unk_token: Option<String>,
}

impl M2M100Tokenizer {
    /// 从模型目录加载 M2M100Tokenizer
    /// 
    /// # Arguments
    /// * `model_dir` - 模型目录路径（如 `models/nmt/m2m100-en-zh/`）
    /// 
    /// # Files Required
    /// - `sentencepiece.bpe.model` - SentencePiece BPE 模型
    /// - `vocab.json` - 词汇表
    /// - `tokenizer_config.json` - Tokenizer 配置（可选）
    pub fn from_model_dir(model_dir: &Path) -> Result<Self> {
        // 1. 检查必需文件
        let sp_model_path = model_dir.join("sentencepiece.bpe.model");
        let vocab_path = model_dir.join("vocab.json");
        
        if !sp_model_path.exists() {
            return Err(anyhow!(
                "sentencepiece.bpe.model not found at {}\n\
                Please ensure the model files are properly exported.",
                sp_model_path.display()
            ));
        }
        
        if !vocab_path.exists() {
            return Err(anyhow!(
                "vocab.json not found at {}\n\
                Please ensure the model files are properly exported.",
                vocab_path.display()
            ));
        }

        // 2. 加载 vocab.json 获取词汇表
        let vocab_str = fs::read_to_string(&vocab_path)
            .map_err(|e| anyhow!("failed to read vocab.json: {e}"))?;
        let vocab: HashMap<String, i64> = serde_json::from_str(&vocab_str)
            .map_err(|e| anyhow!("failed to parse vocab.json: {e}"))?;

        // 3. 从 vocab.json 提取语言 token ID
        let mut lang_id_map = HashMap::new();
        for (token, &id) in &vocab {
            // M2M100 使用双下划线格式：__en__, __zh__ 等
            if token.starts_with("__") && token.ends_with("__") && token.len() >= 6 && token.len() <= 8 {
                let lang_code = &token[2..token.len()-2]; // 提取 __en__ -> en
                if lang_code.len() >= 2 && lang_code.len() <= 3 {
                    lang_id_map.insert(lang_code.to_string(), id);
                }
            }
        }

        // 如果 vocab.json 中没有找到语言 token，使用已知的 M2M100 418M 模型的标准语言 ID（作为后备）
        if lang_id_map.is_empty() {
            eprintln!("Warning: Language tokens not found in vocab.json. Using fallback IDs.");
            lang_id_map.insert("en".to_string(), 128022);
            lang_id_map.insert("zh".to_string(), 128102);
        }

        // 4. 构建 tokenizer
        // tokenizers crate 需要 vocab.json 和 merges.txt 来构建 BPE tokenizer
        // 但 M2M100 只有 sentencepiece.bpe.model，没有 merges.txt
        // 
        // 解决方案：尝试使用 vocab.json 构建 BPE tokenizer，merges.txt 可以为空或使用默认值
        // 或者，我们需要从 sentencepiece.bpe.model 提取 merges.txt
        
        let merges_path = model_dir.join("merges.txt");
        
        // 尝试使用 BPE 模型构建 tokenizer
        let tokenizer = if merges_path.exists() {
            // 如果 merges.txt 存在，使用它
            // BPE::from_file 返回 BpeBuilder，需要调用 build() 方法
            let bpe = BPE::from_file(
                vocab_path.to_str().ok_or_else(|| anyhow!("Invalid vocab.json path"))?,
                merges_path.to_str().ok_or_else(|| anyhow!("Invalid merges.txt path"))?,
            )
            .build()
            .map_err(|e| anyhow!("Failed to load BPE from vocab.json and merges.txt: {e}"))?;
            Tokenizer::new(bpe)
        } else {
            // 如果 merges.txt 不存在，尝试使用 vocab.json 构建（可能需要默认 merges）
            // 注意：这可能需要创建一个基本的 merges.txt
            return Err(anyhow!(
                "merges.txt not found at {}\n\
                M2M100 model requires merges.txt to build BPE tokenizer.\n\
                Please convert sentencepiece.bpe.model to merges.txt using:\n\
                python scripts/convert_sentencepiece_to_merges.py --model {} --vocab-output {} --merges-output {}",
                merges_path.display(),
                sp_model_path.display(),
                vocab_path.display(),
                merges_path.display()
            ));
        };

        // 5. 从 tokenizer_config.json 或 vocab.json 获取特殊 token ID
        let config_path = model_dir.join("tokenizer_config.json");
        let (pad_token, eos_token) = if config_path.exists() {
            if let Ok(config_str) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<TokenizerConfig>(&config_str) {
                    (
                        config.pad_token.unwrap_or_else(|| "<pad>".to_string()),
                        config.eos_token.unwrap_or_else(|| "</s>".to_string()),
                    )
                } else {
                    ("<pad>".to_string(), "</s>".to_string())
                }
            } else {
                ("<pad>".to_string(), "</s>".to_string())
            }
        } else {
            ("<pad>".to_string(), "</s>".to_string())
        };

        // 从 vocab.json 获取特殊 token ID
        let pad_token_id = vocab.get(&pad_token).copied().unwrap_or(1);
        let eos_token_id = vocab.get(&eos_token).copied().unwrap_or(2);

        Ok(Self {
            tokenizer,
            lang_id_map,
            pad_token_id,
            eos_token_id,
        })
    }

    /// 编码文本
    /// 
    /// # Arguments
    /// * `text` - 要编码的文本
    /// * `src_lang` - 源语言代码（如 "en" 或 "zh"）
    /// * `add_special_tokens` - 是否添加特殊 token
    /// 
    /// # Returns
    /// 编码后的 token ID 列表
    /// 
    /// # Note
    /// M2M100 使用双下划线格式的语言 token（__en__, __zh__），
    /// 需要在文本前添加语言 token
    pub fn encode(&self, text: &str, src_lang: &str, add_special_tokens: bool) -> Result<Vec<i64>> {
        // M2M100 需要在文本前添加语言 token
        // 注意：应该直接使用语言 token ID，而不是通过字符串编码
        // 格式：[lang_token_id, ...text_tokens..., eos_token_id]
        
        // 1. 获取语言 token ID
        let lang_token_id = self.get_lang_id(src_lang);
        
        // 2. 编码文本（不添加语言 token 字符串）
        let encoding = self.tokenizer
            .encode(text, add_special_tokens)
            .map_err(|e| anyhow!("failed to encode text: {e}"))?;

        // 3. 转换为 Vec<i64>
        let mut ids: Vec<i64> = encoding
            .get_ids()
            .iter()
            .map(|&id| id as i64)
            .collect();

        // 4. 在开头添加语言 token ID
        ids.insert(0, lang_token_id);
        
        // 5. 如果 add_special_tokens 为 true，在末尾添加 EOS token
        if add_special_tokens {
            ids.push(self.eos_token_id);
        }

        Ok(ids)
    }

    /// 解码 token ID 列表为文本
    /// 
    /// # Arguments
    /// * `ids` - token ID 列表
    /// * `skip_special_tokens` - 是否跳过特殊 token（语言 token、pad、eos 等）
    /// 
    /// # Returns
    /// 解码后的文本
    pub fn decode(&self, ids: &[i64], skip_special_tokens: bool) -> Result<String> {
        // 转换为 u32 数组（tokenizers 需要的类型）
        let ids_u32: Vec<u32> = ids.iter().map(|&id| id as u32).collect();

        // 使用 tokenizer 解码
        let text = self.tokenizer
            .decode(&ids_u32, skip_special_tokens)
            .map_err(|e| anyhow!("failed to decode ids: {e}"))?;

        Ok(text)
    }

    /// 获取语言 token ID
    /// 
    /// # Arguments
    /// * `lang` - 语言代码（如 "en" 或 "zh"）
    /// 
    /// # Returns
    /// 语言 token 的 ID
    /// 
    /// # Panics
    /// 如果语言代码无效，会 panic（fail-fast 设计）
    pub fn get_lang_id(&self, lang: &str) -> i64 {
        self.lang_id_map
            .get(lang)
            .copied()
            .expect(&format!("Invalid M2M100 language code: {}. Available languages: {:?}", 
                lang, 
                self.lang_id_map.keys().collect::<Vec<_>>()))
    }

    /// 获取 pad token ID
    pub fn pad_token_id(&self) -> i64 {
        self.pad_token_id
    }

    /// 获取 eos token ID
    pub fn eos_token_id(&self) -> i64 {
        self.eos_token_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_test_model_dir() -> PathBuf {
        // 假设测试时模型在标准位置
        PathBuf::from("models/nmt/m2m100-en-zh")
    }

    #[test]
    fn test_tokenizer_load() {
        let model_dir = get_test_model_dir();
        if !model_dir.exists() {
            eprintln!("Skipping test: model directory not found at {}", model_dir.display());
            return;
        }

        let tokenizer = M2M100Tokenizer::from_model_dir(&model_dir);
        assert!(tokenizer.is_ok(), "Failed to load tokenizer: {:?}", tokenizer.err());
    }

    #[test]
    fn test_get_lang_id() {
        let model_dir = get_test_model_dir();
        if !model_dir.exists() {
            eprintln!("Skipping test: model directory not found at {}", model_dir.display());
            return;
        }

        let tokenizer = M2M100Tokenizer::from_model_dir(&model_dir).unwrap();
        
        // 测试获取语言 ID
        let en_id = tokenizer.get_lang_id("en");
        let zh_id = tokenizer.get_lang_id("zh");
        
        assert_ne!(en_id, zh_id, "en and zh should have different IDs");
        assert!(en_id > 0, "en ID should be positive");
        assert!(zh_id > 0, "zh ID should be positive");
    }

    #[test]
    #[should_panic(expected = "Invalid M2M100 language code")]
    fn test_get_lang_id_invalid() {
        let model_dir = get_test_model_dir();
        if !model_dir.exists() {
            eprintln!("Skipping test: model directory not found at {}", model_dir.display());
            return;
        }

        let tokenizer = M2M100Tokenizer::from_model_dir(&model_dir).unwrap();
        tokenizer.get_lang_id("invalid"); // 应该 panic
    }

    #[test]
    fn test_encode_decode() {
        let model_dir = get_test_model_dir();
        if !model_dir.exists() {
            eprintln!("Skipping test: model directory not found at {}", model_dir.display());
            return;
        }

        let tokenizer = M2M100Tokenizer::from_model_dir(&model_dir).unwrap();
        
        // 测试编码
        let text = "Hello world";
        let ids = tokenizer.encode(text, "en", true).unwrap();
        assert!(!ids.is_empty(), "Encoded IDs should not be empty");
        
        // 测试解码
        let decoded = tokenizer.decode(&ids, true).unwrap();
        // 解码后的文本可能不完全相同（因为 tokenization 的影响），但应该包含原文本的内容
        assert!(decoded.to_lowercase().contains("hello") || decoded.to_lowercase().contains("world"),
            "Decoded text should contain original words. Decoded: {}", decoded);
    }
}
