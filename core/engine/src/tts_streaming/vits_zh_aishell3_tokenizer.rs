use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;

/// VITS 中文 AISHELL3 Tokenizer
/// 
/// 处理中文文本到音素序列的转换
pub struct VitsZhAishell3Tokenizer {
    /// token 到 ID 的映射（从 tokens.txt）
    token_to_id: HashMap<String, i64>,
    /// 汉字到拼音的映射（从 lexicon.txt）
    /// 格式：汉字 -> (声母, 韵母, 音调)
    char_to_pinyin: HashMap<char, (String, String, u8)>,
    /// 特殊 token IDs
    sil_id: i64,  // 0
    eos_id: i64,  // 1
    sp_id: i64,   // 2
}

impl VitsZhAishell3Tokenizer {
    /// 从模型目录加载 tokenizer
    pub fn from_model_dir(model_dir: &Path) -> Result<Self> {
        // 1. 加载 tokens.txt
        let tokens_path = model_dir.join("tokens.txt");
        if !tokens_path.exists() {
            return Err(anyhow!("tokens.txt not found at {}", tokens_path.display()));
        }
        
        let tokens_data = std::fs::read_to_string(&tokens_path)
            .map_err(|e| anyhow!("failed to read tokens.txt: {e}"))?;
        
        let mut token_to_id = HashMap::new();
        let mut sil_id = 0i64;
        let mut eos_id = 1i64;
        let mut sp_id = 2i64;
        
        for line in tokens_data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let token = parts[0].to_string();
                let id: i64 = parts[1].parse()
                    .map_err(|e| anyhow!("invalid token ID in tokens.txt: {e}"))?;
                
                token_to_id.insert(token.clone(), id);
                
                // 记录特殊 token IDs
                if token == "sil" {
                    sil_id = id;
                } else if token == "eos" {
                    eos_id = id;
                } else if token == "sp" {
                    sp_id = id;
                }
            }
        }
        
        // 2. 加载 lexicon.txt
        let lexicon_path = model_dir.join("lexicon.txt");
        if !lexicon_path.exists() {
            return Err(anyhow!("lexicon.txt not found at {}", lexicon_path.display()));
        }
        
        let lexicon_data = std::fs::read_to_string(&lexicon_path)
            .map_err(|e| anyhow!("failed to read lexicon.txt: {e}"))?;
        
        let mut char_to_pinyin = HashMap::new();
        
        for line in lexicon_data.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            // 格式：汉字 声母 韵母 音调 #0
            // 例如：一 ^ i1 #0
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let ch = parts[0].chars().next()
                    .ok_or_else(|| anyhow!("empty character in lexicon.txt"))?;
                let initial = parts[1].to_string();  // 声母
                let final_part = parts[2].to_string();  // 韵母
                
                // 提取音调（从韵母字符串中，例如 "i1" -> 1）
                let tone = if let Some(last_char) = final_part.chars().last() {
                    if last_char.is_ascii_digit() {
                        last_char.to_digit(10).unwrap_or(5) as u8
                    } else {
                        5  // 默认音调
                    }
                } else {
                    5
                };
                
                // 移除音调数字，只保留韵母部分
                let final_clean = final_part
                    .chars()
                    .filter(|c| !c.is_ascii_digit())
                    .collect::<String>();
                
                // 如果已经有这个汉字，跳过（使用第一个）
                if !char_to_pinyin.contains_key(&ch) {
                    char_to_pinyin.insert(ch, (initial, final_clean, tone));
                }
            }
        }
        
        Ok(Self {
            token_to_id,
            char_to_pinyin,
            sil_id,
            eos_id,
            sp_id,
        })
    }
    
    /// 将拼音转换为音素 token IDs
    /// 
    /// 拼音格式：声母 + 韵母 + 音调
    /// 注意：零声母（^）也需要添加 token
    /// 例如：^ + i + 1 -> ^ (7) + i1 (79)
    fn pinyin_to_tokens(&self, initial: &str, final_part: &str, tone: u8) -> Result<Vec<i64>> {
        let mut tokens = Vec::new();
        
        // 1. 添加声母 token（包括零声母 ^）
        if !initial.is_empty() {
            if let Some(&id) = self.token_to_id.get(initial) {
                tokens.push(id);
            } else {
                // 如果找不到声母 token，尝试使用 ^（零声母）
                if let Some(&id) = self.token_to_id.get("^") {
                    tokens.push(id);
                }
            }
        }
        
        // 2. 添加韵母+音调 token（例如 i1, a2）
        let final_with_tone = format!("{}{}", final_part, tone);
        if let Some(&id) = self.token_to_id.get(&final_with_tone) {
            tokens.push(id);
        } else {
            // 如果找不到带音调的，尝试不带音调的（使用默认音调 5）
            let final_with_tone5 = format!("{}5", final_part);
            if let Some(&id) = self.token_to_id.get(&final_with_tone5) {
                tokens.push(id);
            } else {
                return Err(anyhow!("token not found for final: {} (tried {} and {})", final_with_tone, final_with_tone, final_with_tone5));
            }
        }
        
        Ok(tokens)
    }
    
    /// 编码中文文本为 token IDs
    /// 
    /// 流程：
    /// 1. 中文文本 -> 汉字序列
    /// 2. 每个汉字 -> 查找 lexicon.txt -> 得到 (声母, 韵母, 音调)
    /// 3. (声母, 韵母, 音调) -> 转换为音素 tokens
    /// 4. 添加特殊 token：开头 sil，结尾 eos，字之间可能加 sp
    /// 
    /// 返回：(token_ids, sequence_length)
    pub fn encode(&self, text: &str) -> Result<(Vec<i64>, i64)> {
        let mut token_ids = Vec::new();
        
        // 1. 添加开头的 sil token
        token_ids.push(self.sil_id);
        
        // 2. 处理每个字符
        let mut prev_was_chinese = false;
        
        for ch in text.chars() {
            if ch.is_whitespace() {
                // 空格：添加 sp token
                if !token_ids.is_empty() && *token_ids.last().unwrap() != self.sp_id {
                    token_ids.push(self.sp_id);
                }
                prev_was_chinese = false;
                continue;
            }
            
            // 检查是否是中文字符
            if ch >= '\u{4e00}' && ch <= '\u{9fff}' {
                // 中文字符：查找拼音
                if let Some((initial, final_part, tone)) = self.char_to_pinyin.get(&ch) {
                    // 在字之间添加 sp token（如果前一个也是中文）
                    if prev_was_chinese {
                        token_ids.push(self.sp_id);
                    }
                    
                    // 转换为音素 tokens
                    let pinyin_tokens = self.pinyin_to_tokens(initial, final_part, *tone)?;
                    token_ids.extend(pinyin_tokens);
                    prev_was_chinese = true;
                } else {
                    // 未找到拼音，跳过或使用默认处理
                    eprintln!("[WARN] Character not found in lexicon: {}", ch);
                    prev_was_chinese = false;
                }
            } else {
                // 非中文字符（标点符号等）：跳过或添加 sp
                if prev_was_chinese {
                    token_ids.push(self.sp_id);
                }
                prev_was_chinese = false;
            }
        }
        
        // 3. 添加结尾的 eos token
        token_ids.push(self.eos_id);
        
        // 4. 计算序列长度
        let seq_len = token_ids.len() as i64;
        
        Ok((token_ids, seq_len))
    }
}

