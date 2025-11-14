use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};

/// Marian NMT 支持的语言代码
/// 对应模型目录命名：marian-{source}-{target}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageCode {
    En,  // English
    Zh,  // Chinese
    Es,  // Spanish
    Ja,  // Japanese
    // 可以继续添加其他语言
}

impl LanguageCode {
    /// 从字符串转换为语言代码
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "en" | "eng" | "english" => Ok(LanguageCode::En),
            "zh" | "zho" | "chinese" | "中文" => Ok(LanguageCode::Zh),
            "es" | "spa" | "spanish" | "español" => Ok(LanguageCode::Es),
            "ja" | "jpn" | "japanese" | "日本語" => Ok(LanguageCode::Ja),
            _ => Err(anyhow!("Unsupported language code: {}", s)),
        }
    }

    /// 转换为目录名格式（如 "en", "zh"）
    pub fn to_dir_name(&self) -> &'static str {
        match self {
            LanguageCode::En => "en",
            LanguageCode::Zh => "zh",
            LanguageCode::Es => "es",
            LanguageCode::Ja => "ja",
        }
    }
}

/// 语言对：源语言 -> 目标语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanguagePair {
    pub source: LanguageCode,
    pub target: LanguageCode,
}

impl LanguagePair {
    /// 创建新的语言对
    pub fn new(source: LanguageCode, target: LanguageCode) -> Self {
        Self { source, target }
    }

    /// 从字符串创建语言对（如 "en-zh", "eng-zho"）
    pub fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid language pair format: {}", s));
        }
        let source = LanguageCode::from_str(parts[0])?;
        let target = LanguageCode::from_str(parts[1])?;
        Ok(Self { source, target })
    }

    /// 转换为模型目录名（如 "marian-en-zh"）
    pub fn to_model_dir_name(&self) -> String {
        format!("marian-{}-{}", self.source.to_dir_name(), self.target.to_dir_name())
    }

    /// 从模型目录名解析语言对（如 "marian-en-zh" -> LanguagePair { source: En, target: Zh }）
    pub fn from_model_dir_name(dir_name: &str) -> Result<Self> {
        // 移除 "marian-" 前缀
        let name = dir_name.strip_prefix("marian-")
            .ok_or_else(|| anyhow!("Invalid model directory name: {}", dir_name))?;
        
        // 分割源语言和目标语言
        let parts: Vec<&str> = name.split('-').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid model directory name format: {}", dir_name));
        }
        
        let source = LanguageCode::from_str(parts[0])?;
        let target = LanguageCode::from_str(parts[1])?;
        Ok(Self { source, target })
    }

    /// 根据语言对查找模型目录
    /// 
    /// # Arguments
    /// * `base_dir` - 模型基础目录（如 `core/engine/models/nmt/`）
    /// 
    /// # Returns
    /// 完整的模型目录路径（如 `core/engine/models/nmt/marian-en-zh/`）
    pub fn find_model_dir(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.to_model_dir_name())
    }

    /// 从模型目录路径自动识别语言对
    /// 
    /// # Arguments
    /// * `model_dir` - 模型目录路径（如 `core/engine/models/nmt/marian-en-zh/`）
    /// 
    /// # Returns
    /// 解析出的语言对
    pub fn from_model_dir(model_dir: &Path) -> Result<Self> {
        let dir_name = model_dir.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid model directory path: {}", model_dir.display()))?;
        
        Self::from_model_dir_name(dir_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_pair_from_str() {
        let pair = LanguagePair::from_str("en-zh").unwrap();
        assert_eq!(pair.source, LanguageCode::En);
        assert_eq!(pair.target, LanguageCode::Zh);
    }

    #[test]
    fn test_language_pair_to_model_dir_name() {
        let pair = LanguagePair::new(LanguageCode::En, LanguageCode::Zh);
        assert_eq!(pair.to_model_dir_name(), "marian-en-zh");
    }

    #[test]
    fn test_language_pair_from_model_dir_name() {
        let pair = LanguagePair::from_model_dir_name("marian-en-zh").unwrap();
        assert_eq!(pair.source, LanguageCode::En);
        assert_eq!(pair.target, LanguageCode::Zh);
    }
}

