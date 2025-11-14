use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::language_pair::LanguagePair;

/// vocab.json 的反序列化类型：简单的 { token: id } 映射
#[derive(Debug, Deserialize)]
pub struct MarianVocab(pub HashMap<String, i32>);

pub struct MarianTokenizer {
    pub vocab: MarianVocab,
    pub bos_id: i32,
    pub eos_id: i32,
    pub pad_id: i32,
    pub language_pair: LanguagePair,
    // 未来可以添加：
    // pub source_tokenizer: SentencePieceProcessor,  // 从 source.spm 加载
    // pub target_tokenizer: SentencePieceProcessor,  // 从 target.spm 加载
}

impl MarianTokenizer {
    /// 从模型目录加载 tokenizer
    /// 
    /// # Arguments
    /// * `model_dir` - 模型目录路径（如 `models/nmt/marian-en-zh/`）
    /// * `language_pair` - 语言对信息
    pub fn from_model_dir(model_dir: &Path, language_pair: LanguagePair) -> Result<Self> {
        let vocab_path = model_dir.join("vocab.json");
        Self::from_file_internal(&vocab_path, language_pair)
    }

    /// 从给定路径加载 vocab.json（保留向后兼容）
    /// 
    /// # Deprecated
    /// 建议使用 `from_model_dir` 以支持多语言
    pub fn from_file(vocab_path: &Path) -> Result<Self> {
        // 尝试从路径推断语言对（如果路径包含 marian-xx-yy）
        let model_dir = vocab_path.parent()
            .ok_or_else(|| anyhow!("Invalid vocab path: {}", vocab_path.display()))?;
        
        let language_pair = LanguagePair::from_model_dir(model_dir)
            .unwrap_or_else(|_| {
                // 如果无法识别，默认使用 en-zh
                LanguagePair::new(
                    super::language_pair::LanguageCode::En,
                    super::language_pair::LanguageCode::Zh,
                )
            });
        
        Self::from_file_internal(vocab_path, language_pair)
    }

    /// 从给定路径加载 vocab.json（内部实现）
    fn from_file_internal(vocab_path: &Path, language_pair: LanguagePair) -> Result<Self> {
        if !vocab_path.exists() {
            return Err(anyhow!(
                "vocab.json not found at {}",
                vocab_path.display()
            ));
        }

        let data = fs::read_to_string(vocab_path)
            .map_err(|e| anyhow!("failed to read vocab.json: {e}"))?;
        let map: HashMap<String, i32> = serde_json::from_str(&data)
            .map_err(|e| anyhow!("failed to parse vocab.json: {e}"))?;

        // 封装成 MarianVocab
        let vocab = MarianVocab(map);

        // 依据 config.json 中的信息设置这些 special id（你现在的 config 是这样的）:
        //   "bos_token_id": 0,
        //   "eos_token_id": 0,
        //   "pad_token_id": 65000,
        //   "decoder_start_token_id": 65000,
        //   "vocab_size": 65001
        // 这里先按 config.json 写死，后面你也可以从 config.json 里动态读取
        let bos_id = 0;
        let eos_id = 0;
        let pad_id = 65000;

        Ok(Self {
            vocab,
            bos_id,
            eos_id,
            pad_id,
            language_pair,
        })
    }

    /// 极简版：按空格拆词，然后用 vocab 做“最长匹配”，找不到就用 unk（这里先用 eos_id 顶一下）
    pub fn encode(&self, text: &str, add_special_tokens: bool) -> Vec<i64> {
        let mut ids: Vec<i64> = Vec::new();

        if add_special_tokens {
            ids.push(self.bos_id as i64);
        }

        for token in text.split_whitespace() {
            // vocab 里是 SentencePiece 风格的 token，大部分是带前缀 "▁"（\u2581），
            // 这里简单地尝试带 "▁" 的形式
            let sp_token = format!("\u{2581}{}", token);
            if let Some(id) = self.vocab.0.get(&sp_token) {
                ids.push(*id as i64);
            } else if let Some(id) = self.vocab.0.get(token) {
                ids.push(*id as i64);
            } else {
                // 真正的实现应该有 unk_id，这里为了不崩溃，先用 eos_id 顶一下
                ids.push(self.eos_id as i64);
            }
        }

        if add_special_tokens {
            ids.push(self.eos_id as i64);
        }

        ids
    }

    /// 极简版解码：把每个 id 找回 token 字符串，然后把 "▁" 当作空格
    pub fn decode(&self, ids: &[i64]) -> String {
        // 先反向构建一个 id -> token 的表
        let mut id2tok: HashMap<i32, &str> = HashMap::new();
        for (tok, id) in self.vocab.0.iter() {
            id2tok.insert(*id, tok.as_str());
        }

        let mut out = String::new();
        for &id in ids {
            let id_i32 = id as i32;

            // 跳过特殊符号
            if id_i32 == self.bos_id || id_i32 == self.eos_id || id_i32 == self.pad_id {
                continue;
            }

            if let Some(tok) = id2tok.get(&id_i32) {
                let t = tok.replace('\u{2581}', " ");
                out.push_str(&t);
            }
        }

        out.trim().to_string()
    }
}
