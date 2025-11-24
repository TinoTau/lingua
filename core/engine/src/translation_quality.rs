//! 翻译质量增强模块
//! 
//! 用于检测和修复翻译质量问题

use crate::error::EngineResult;

/// 翻译质量检查器
pub struct TranslationQualityChecker {
    /// 是否启用质量检查
    enabled: bool,
}

impl TranslationQualityChecker {
    /// 创建新的质量检查器
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// 检查并修复翻译质量
    /// 
    /// # Arguments
    /// * `src_text` - 原文
    /// * `tgt_text` - 译文
    /// * `target_lang` - 目标语言
    /// 
    /// # Returns
    /// 修复后的译文
    pub fn check_and_fix(&self, src_text: &str, tgt_text: &str, target_lang: &str) -> String {
        if !self.enabled {
            return tgt_text.to_string();
        }

        let mut result = tgt_text.to_string();

        // 1. 删除重复序列
        result = self.remove_repetitive_sequences(&result);

        // 2. 检查可疑字符比例
        if self.is_suspicious_quality(&result, target_lang) {
            // 如果质量可疑，尝试简单修复
            result = self.attempt_fix(&result, target_lang);
        }

        result
    }

    /// 删除明显异常的重复序列
    /// 
    /// 例如：`"to to to to"` → `"to"`
    ///       `"theiriririr"` → `"their"`
    #[doc(hidden)]
    pub fn remove_repetitive_sequences(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // 检测重复的单词（连续出现 3 次以上）
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut new_words = Vec::new();
        let mut i = 0;
        
        while i < words.len() {
            let word = words[i];
            let mut count = 1;
            
            // 计算连续重复次数
            while i + count < words.len() && words[i + count] == word {
                count += 1;
            }
            
            // 如果重复超过 2 次，只保留一次
            if count > 2 {
                new_words.push(word);
                i += count;
            } else {
                new_words.push(word);
                i += 1;
            }
        }
        
        new_words.join(" ")
    }

    /// 检查是否为可疑质量
    /// 
    /// 规则：
    /// - 英文：非字母字符比例 > 70%
    /// - 中文：字符数量极少或全为标点
    #[doc(hidden)]
    pub fn is_suspicious_quality(&self, text: &str, target_lang: &str) -> bool {
        if text.trim().is_empty() {
            return true;
        }

        if target_lang == "en" || target_lang == "en-US" {
            // 英文：检查非字母字符比例
            let non_alpha_count = text
                .chars()
                .filter(|c| !c.is_alphabetic() && !c.is_whitespace())
                .count();
            let total_chars = text.chars().filter(|c| !c.is_whitespace()).count();
            
            if total_chars > 0 {
                let non_alpha_ratio = non_alpha_count as f32 / total_chars as f32;
                return non_alpha_ratio > 0.7;
            }
        } else if target_lang == "zh" || target_lang == "zh-CN" {
            // 中文：检查是否为全标点或字符数极少
            let chinese_chars: Vec<char> = text
                .chars()
                .filter(|c| self.is_chinese_char(*c))
                .collect();
            
            if chinese_chars.is_empty() && text.trim().len() > 0 {
                // 有内容但没有中文字符，可能是全标点
                return true;
            }
            
            // 如果原文长度 > 20，而译文长度 < 3，可疑
            if text.trim().len() < 3 {
                return true;
            }
        }

        false
    }

    /// 尝试修复可疑翻译
    fn attempt_fix(&self, text: &str, target_lang: &str) -> String {
        let mut result = text.to_string();
        
        // 移除过多的标点符号
        result = self.remove_excessive_punctuation(&result);
        
        // 如果修复后仍然可疑，返回空字符串（触发重试）
        if self.is_suspicious_quality(&result, target_lang) {
            return String::new();
        }
        
        result
    }

    /// 移除过多的标点符号
    #[doc(hidden)]
    pub fn remove_excessive_punctuation(&self, text: &str) -> String {
        let mut result = text.to_string();
        
        // 移除连续重复的标点
        result = result.replace("...", ".");
        result = result.replace("!!!", "!");
        result = result.replace("???", "?");
        result = result.replace("。。。", "。");
        result = result.replace("！！！", "！");
        result = result.replace("？？？", "？");
        
        result
    }

    /// 检查是否为中文字符
    fn is_chinese_char(&self, c: char) -> bool {
        matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF)
    }
}

impl Default for TranslationQualityChecker {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_repetitive_sequences() {
        let checker = TranslationQualityChecker::new(true);
        assert_eq!(
            checker.remove_repetitive_sequences("to to to to"),
            "to"
        );
        assert_eq!(
            checker.remove_repetitive_sequences("hello world world world"),
            "hello world"
        );
    }

    #[test]
    fn test_is_suspicious_quality_english() {
        let checker = TranslationQualityChecker::new(true);
        assert!(checker.is_suspicious_quality("???###$$$", "en"));
        assert!(!checker.is_suspicious_quality("Hello world", "en"));
    }

    #[test]
    fn test_is_suspicious_quality_chinese() {
        let checker = TranslationQualityChecker::new(true);
        assert!(checker.is_suspicious_quality("？？？", "zh"));
        assert!(!checker.is_suspicious_quality("你好世界", "zh"));
    }

    #[test]
    fn test_check_and_fix() {
        let checker = TranslationQualityChecker::new(true);
        let result = checker.check_and_fix("Hello", "to to to to", "en");
        assert_eq!(result, "to");
    }
}

