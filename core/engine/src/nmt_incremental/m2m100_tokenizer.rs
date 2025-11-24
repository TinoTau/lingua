use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// M2M100 Tokenizer 实现
/// 
/// 基于 vocab.json 的最长匹配分词，支持语言 token 和解码
pub struct M2M100Tokenizer {
    /// token -> id 映射
    vocab: HashMap<String, i64>,
    /// id -> token 映射
    id_to_piece: Vec<String>,
    /// 子词最长匹配 Trie
    trie: Vec<TrieNode>,
    /// 语言代码到 token ID 的映射（如 "en" -> 128022）
    lang_id_map: HashMap<String, i64>,
    /// 语言 token ID 集合（用于解码时过滤）
    lang_token_ids: HashSet<i64>,
    /// Pad token ID
    pad_token_id: i64,
    /// EOS token ID
    eos_token_id: i64,
    /// UNK token ID
    unk_token_id: i64,
}

#[derive(Default)]
struct TrieNode {
    children: HashMap<char, usize>,
    token_id: Option<i64>,
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
    /// - `vocab.json` - 词汇表
    /// - `lang_ids.json` - 语言 ID 映射（推荐）
    /// - `tokenizer_config.json` - Tokenizer 配置（可选）
    pub fn from_model_dir(model_dir: &Path) -> Result<Self> {
        // 1. 检查必需文件
        let vocab_path = model_dir.join("vocab.json");

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

        // 如果 vocab.json 中没有找到语言 token，尝试从 lang_ids.json 加载
        let lang_ids_path = model_dir.join("lang_ids.json");
        if lang_id_map.is_empty() && lang_ids_path.exists() {
            if let Ok(lang_ids_str) = fs::read_to_string(&lang_ids_path) {
                if let Ok(extra_map) = serde_json::from_str::<HashMap<String, i64>>(&lang_ids_str) {
                    lang_id_map.extend(extra_map);
                }
            }
        }

        // 如果仍然没有找到语言 token，使用已知的标准语言 ID（作为后备）
        if lang_id_map.is_empty() {
            eprintln!("Warning: Language tokens not found in vocab.json. Using fallback IDs.");
            lang_id_map.insert("en".to_string(), 128022);
            lang_id_map.insert("zh".to_string(), 128102);
        }

        // 4. 构建 id -> piece 映射（按最大 ID 分配）
        let max_id = vocab.values().copied().max().unwrap_or(0) as usize;
        let mut id_to_piece = vec![String::new(); max_id + 1];
        for (piece, &id) in &vocab {
            let idx = id as usize;
            if idx >= id_to_piece.len() {
                id_to_piece.resize(idx + 1, String::new());
            }
            id_to_piece[idx] = piece.clone();
        }

        // 构建 Trie
        let mut trie = TrieNode::default();
        let mut nodes = vec![trie];
        for (piece, &id) in &vocab {
            if should_skip_piece(piece) {
                continue;
            }
            insert_piece(&mut nodes, piece, id);
        }

        // 5. 从 tokenizer_config.json 或 vocab.json 获取特殊 token ID
        let config_path = model_dir.join("tokenizer_config.json");
        let (pad_token, eos_token, unk_token) = if config_path.exists() {
            if let Ok(config_str) = fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str::<TokenizerConfig>(&config_str) {
                    (
                        config.pad_token.unwrap_or_else(|| "<pad>".to_string()),
                        config.eos_token.unwrap_or_else(|| "</s>".to_string()),
                        config.unk_token.unwrap_or_else(|| "<unk>".to_string()),
                    )
                } else {
                    ("<pad>".to_string(), "</s>".to_string(), "<unk>".to_string())
                }
            } else {
                ("<pad>".to_string(), "</s>".to_string(), "<unk>".to_string())
            }
        } else {
            ("<pad>".to_string(), "</s>".to_string(), "<unk>".to_string())
        };

        // 从 vocab.json 获取特殊 token ID
        let pad_token_id = vocab.get(&pad_token).copied().unwrap_or(1);
        let eos_token_id = vocab.get(&eos_token).copied().unwrap_or(2);
        let unk_token_id = vocab.get(&unk_token).copied().unwrap_or(3);

        let lang_token_ids = lang_id_map.values().copied().collect();

        Ok(Self {
            vocab,
            id_to_piece,
            trie: nodes,
            lang_id_map,
            lang_token_ids,
            pad_token_id,
            eos_token_id,
            unk_token_id,
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
        let normalized_chars = normalize_text(text);

        let mut ids = Vec::with_capacity(normalized_chars.len() + 2);
        ids.push(lang_token_id);

        let mut pos = 0;
        while pos < normalized_chars.len() {
            let mut node_idx = 0usize;
            let mut last_match: Option<(usize, i64)> = None;
            let mut inner = pos;
            while inner < normalized_chars.len() {
                let ch = normalized_chars[inner];
                if let Some(&next_idx) = self.trie[node_idx].children.get(&ch) {
                    node_idx = next_idx;
                    if let Some(token_id) = self.trie[node_idx].token_id {
                        last_match = Some((inner - pos + 1, token_id));
                    }
                    inner += 1;
                } else {
                    break;
                }
            }

            if let Some((len, token_id)) = last_match {
                ids.push(token_id);
                pos += len;
            } else {
                // 回退为单字符（若不存在则使用 UNK）
                let ch = normalized_chars[pos];
                let mut fallback = ch.to_string();
                if ch == '▁' {
                    fallback = "▁".to_string();
                }
                if let Some(token_id) = self.vocab.get(&fallback).copied() {
                    ids.push(token_id);
                } else {
                    ids.push(self.unk_token_id);
                }
                pos += 1;
            }
        }

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
        let mut filtered_pieces: Vec<String> = Vec::with_capacity(ids.len());
        for &id in ids {
            if skip_special_tokens {
                if id == self.pad_token_id || id == self.eos_token_id || self.lang_token_ids.contains(&id) {
                    continue;
                }
            }
            let idx = id as usize;
            if idx < self.id_to_piece.len() {
                filtered_pieces.push(self.id_to_piece[idx].clone());
            }
        }

        let mut text = filtered_pieces.join("");
        text = text.replace('▁', " ");
        Ok(text.trim().to_string())
    }

    /// 获取 token ID 对应的文本片段（用于调试）
    pub fn id_to_piece(&self, id: i64) -> Option<String> {
        let idx = id as usize;
        if idx < self.id_to_piece.len() {
            Some(self.id_to_piece[idx].clone())
        } else {
            None
        }
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

    /// 获取 unk token ID
    pub fn unk_token_id(&self) -> i64 {
        self.unk_token_id
    }
}

fn should_skip_piece(piece: &str) -> bool {
    if piece.starts_with('<') && piece.ends_with('>') {
        return true;
    }
    if piece.starts_with("__") && piece.ends_with("__") {
        return true;
    }
    piece.is_empty()
}

fn insert_piece(nodes: &mut Vec<TrieNode>, piece: &str, token_id: i64) {
    let mut current_idx = 0usize;
    for ch in piece.chars() {
        let next_idx = if let Some(&idx) = nodes[current_idx].children.get(&ch) {
            idx
        } else {
            let idx = nodes.len();
            nodes.push(TrieNode::default());
            nodes[current_idx].children.insert(ch, idx);
            idx
        };
        current_idx = next_idx;
    }
    nodes[current_idx].token_id = Some(token_id);
}

fn normalize_text(text: &str) -> Vec<char> {
    let mut chars = Vec::new();
    let mut prev_space = true;
    for ch in text.chars() {
        if ch.is_whitespace() {
            prev_space = true;
            continue;
        }
        if prev_space {
            chars.push('▁');
            prev_space = false;
        }
        chars.push(ch);
    }
    if chars.is_empty() {
        chars.push('▁');
    }
    chars
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
