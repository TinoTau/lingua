use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

/// 文本预处理器
/// 负责文本规范化、音素转换和音素 ID 映射
#[derive(Clone)]
pub struct TextProcessor {
    /// 音素到 ID 的映射
    phone_to_id: HashMap<String, i64>,
    /// ID 到音素的映射（反向）
    id_to_phone: HashMap<i64, String>,
    /// 默认语言
    locale: String,
}

impl TextProcessor {
    /// 从模型目录加载文本预处理器
    pub fn new_from_dir(model_dir: &Path, locale: &str) -> Result<Self> {
        let phone_map_path = model_dir.join("fastspeech2-lite").join("phone_id_map.txt");
        
        if !phone_map_path.exists() {
            return Err(anyhow!("phone_id_map.txt not found at {}", phone_map_path.display()));
        }

        // 读取 phone_id_map.txt
        let content = fs::read_to_string(&phone_map_path)
            .map_err(|e| anyhow!("failed to read phone_id_map.txt: {e}"))?;

        let mut phone_to_id = HashMap::new();
        let mut id_to_phone = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let phone = parts[0].to_string();
                let id: i64 = parts[1].parse()
                    .map_err(|e| anyhow!("invalid phone ID in phone_id_map.txt: {}: {e}", parts[1]))?;
                
                phone_to_id.insert(phone.clone(), id);
                id_to_phone.insert(id, phone);
            }
        }

        Ok(Self {
            phone_to_id,
            id_to_phone,
            locale: locale.to_string(),
        })
    }

    /// 规范化文本（简化版：目前只做基本处理）
    pub fn normalize_text(&self, text: &str) -> String {
        let mut result = text.trim().to_string();
        
        if result.is_empty() {
            return result;
        }
        
        // 移除多余空格
        result = result.split_whitespace().collect::<Vec<&str>>().join(" ");
        
        // 基本标点符号处理（保留常见标点）
        // 移除特殊字符（保留字母、数字、常见标点）
        result = result.chars()
            .filter(|c| c.is_alphanumeric() || 
                       matches!(c, ' ' | '.' | ',' | '!' | '?' | ';' | ':' | '-' | '\'' | '"'))
            .collect();
        
        // TODO: 实现更复杂的规范化
        // - 数字转文字（"123" → "一百二十三" 或 "one hundred twenty three"）
        // - 日期时间处理
        // - 缩写展开（"Dr." → "Doctor"）
        // - 货币、单位转换
        
        result
    }

    /// 将文本转换为音素序列（简化版：目前只做基本映射）
    /// 
    /// 注意：这是一个简化实现，实际需要：
    /// - 中文：文本 → 拼音 → 音素
    /// - 英文：文本 → 音素（使用 CMUdict 或类似词典）
    /// 
    /// 当前实现：直接使用 phone_id_map.txt 中存在的音素
    /// 对于中文，尝试将字符映射到可能的音素
    /// 对于英文，尝试将单词映射到可能的音素
    pub fn text_to_phonemes(&self, text: &str) -> Result<Vec<String>> {
        let normalized = self.normalize_text(text);
        
        if normalized.is_empty() {
            return Ok(vec![]);
        }
        
        // 简化实现：根据 locale 选择不同的处理方式
        match self.locale.as_str() {
            "zh" | "chinese" | "zh-CN" => {
                // 中文：暂时返回字符级别的音素
                // 实际应该：文本 → 拼音 → 音素（如 "ni3", "hao3"）
                // TODO: 实现中文拼音转换
                // 当前：尝试将每个字符映射到可能的音素，如果找不到则使用 <unk>
                let mut phonemes = Vec::new();
                for ch in normalized.chars() {
                    let ch_str = ch.to_string();
                    // 检查字符是否在 phone_id_map 中
                    if self.phone_to_id.contains_key(&ch_str) {
                        phonemes.push(ch_str);
                    } else {
                        // 使用 <unk> 或跳过
                        phonemes.push("<unk>".to_string());
                    }
                }
                Ok(phonemes)
            }
            "en" | "english" | "en-US" => {
                // 英文：暂时返回单词级别的音素
                // 实际应该：文本 → 音素（如 "HH", "EH", "L", "OW"）
                // TODO: 实现英文音素转换（使用 CMUdict）
                // 当前：尝试将每个单词映射到可能的音素，如果找不到则使用 <unk>
                let mut phonemes = Vec::new();
                for word in normalized.split_whitespace() {
                    let word_lower = word.to_lowercase();
                    // 检查单词是否在 phone_id_map 中（英文音素通常是单个字母或组合）
                    // 这里简化处理：尝试将单词拆分为可能的音素
                    // 实际应该使用 CMUdict 查找
                    if self.phone_to_id.contains_key(&word_lower) {
                        phonemes.push(word_lower);
                    } else {
                        // 使用 <unk> 或尝试拆分
                        phonemes.push("<unk>".to_string());
                    }
                }
                Ok(phonemes)
            }
            _ => Err(anyhow!("Unsupported locale for text-to-phoneme: {}", self.locale)),
        }
    }

    /// 将音素序列转换为音素 ID 序列
    pub fn phonemes_to_ids(&self, phonemes: &[String]) -> Result<Vec<i64>> {
        let mut ids = Vec::new();
        
        for phone in phonemes {
            // 尝试直接查找
            if let Some(&id) = self.phone_to_id.get(phone) {
                ids.push(id);
            } else {
                // 如果找不到，尝试查找小写版本
                let phone_lower = phone.to_lowercase();
                if let Some(&id) = self.phone_to_id.get(&phone_lower) {
                    ids.push(id);
                } else {
                    // 如果还是找不到，使用 <unk> 的 ID（通常是 1）
                    let unk_id = self.phone_to_id.get("<unk>")
                        .copied()
                        .unwrap_or(1);
                    ids.push(unk_id);
                }
            }
        }
        
        Ok(ids)
    }

    /// 将文本直接转换为音素 ID 序列（便捷方法）
    pub fn text_to_phone_ids(&self, text: &str) -> Result<Vec<i64>> {
        let phonemes = self.text_to_phonemes(text)?;
        self.phonemes_to_ids(&phonemes)
    }

    /// 获取音素 ID 映射（用于调试）
    pub fn get_phone_to_id_map(&self) -> &HashMap<String, i64> {
        &self.phone_to_id
    }
}

