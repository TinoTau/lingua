//! 文本后处理模块
//! 
//! 用于清洗和优化翻译后的文本

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 文本后处理器
pub struct TextPostProcessor {
    terms_map: HashMap<String, String>,
    enabled: bool,
}

impl TextPostProcessor {
    /// 创建新的后处理器
    pub fn new(terms_file: Option<&Path>, enabled: bool) -> Self {
        let terms_map = if let Some(path) = terms_file {
            Self::load_terms(path).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self { terms_map, enabled }
    }

    /// 从 JSON 文件加载术语表
    fn load_terms(path: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)?;

        let mut terms = HashMap::new();
        if let Value::Object(map) = json {
            for (key, value) in map {
                if let Value::String(val) = value {
                    terms.insert(key, val);
                }
            }
        }

        Ok(terms)
    }

    /// 处理文本
    pub fn process(&self, text: &str, target_lang: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }

        let mut result = text.to_string();

        // 1. 基础清洗
        result = self.clean_text(&result);

        // 2. 术语替换
        result = self.replace_terms(&result);

        // 3. 句号兜底
        result = self.add_punctuation_if_needed(&result, target_lang);

        result
    }

    /// 基础文本清洗
    fn clean_text(&self, text: &str) -> String {
        let mut result = text.trim().to_string();

        // 合并连续空格
        while result.contains("  ") {
            result = result.replace("  ", " ");
        }

        // 处理重复标点（中文）
        result = result.replace("。。。。", "。");
        result = result.replace("，，，，", "，");
        result = result.replace("？？？？", "？");
        result = result.replace("！！！！", "！");

        // 处理重复标点（英文）
        result = result.replace("....", ".");
        result = result.replace(",,,", ",");
        result = result.replace("???", "?");
        result = result.replace("!!!", "!");

        result
    }

    /// 术语替换
    fn replace_terms(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (key, value) in &self.terms_map {
            // 简单替换（区分大小写）
            result = result.replace(key, value);
        }
        result
    }

    /// 添加标点符号（如果需要）
    fn add_punctuation_if_needed(&self, text: &str, target_lang: &str) -> String {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return text.to_string();
        }

        // 检查末尾是否已有标点
        let last_char = trimmed.chars().last().unwrap_or(' ');
        let has_punctuation = matches!(
            last_char,
            '.' | ',' | '!' | '?' | '。' | '，' | '！' | '？' | ';' | '；' | ':' | '：'
        );

        if !has_punctuation {
            if target_lang == "zh" || target_lang == "zh-CN" {
                return format!("{}。", trimmed);
            } else {
                return format!("{}.", trimmed);
            }
        }

        text.to_string()
    }
}

impl Default for TextPostProcessor {
    fn default() -> Self {
        Self {
            terms_map: HashMap::new(),
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let processor = TextPostProcessor::default();
        assert_eq!(processor.clean_text("  hello  world  "), "hello world");
        assert_eq!(processor.clean_text("test...."), "test.");
        assert_eq!(processor.clean_text("测试。。。。"), "测试。");
    }

    #[test]
    fn test_add_punctuation() {
        let processor = TextPostProcessor::default();
        assert_eq!(processor.add_punctuation_if_needed("hello", "en"), "hello.");
        assert_eq!(processor.add_punctuation_if_needed("测试", "zh"), "测试。");
        assert_eq!(processor.add_punctuation_if_needed("hello.", "en"), "hello.");
        assert_eq!(processor.add_punctuation_if_needed("测试。", "zh"), "测试。");
    }
}

