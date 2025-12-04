//! ASR 过滤规则配置结构

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::fs;
use std::path::Path;

use crate::error::{EngineError, EngineResult};

/// ASR 过滤规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrFilterConfig {
    pub version: String,
    #[serde(default)]
    pub description: String,
    pub rules: FilterRules,
}

/// 过滤规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRules {
    /// 是否过滤空文本
    #[serde(default = "default_true")]
    pub filter_empty: bool,
    
    /// 是否过滤包含括号的文本
    #[serde(default = "default_true")]
    pub filter_brackets: bool,
    
    /// 单个字的无意义语气词列表
    #[serde(default)]
    pub single_char_fillers: Vec<String>,
    
    /// 精确匹配列表（不区分大小写）
    #[serde(default)]
    pub exact_matches: Vec<String>,
    
    /// 部分匹配模式列表
    #[serde(default)]
    pub contains_patterns: Vec<String>,
    
    /// 需要同时包含多个模式的组合
    #[serde(default)]
    pub all_contains_patterns: Vec<AllContainsPattern>,
    
    /// 字幕相关模式
    #[serde(default)]
    pub subtitle_patterns: Vec<String>,
    
    /// 其他无意义模式
    #[serde(default)]
    pub meaningless_patterns: Vec<String>,
    
    /// 上下文相关的感谢语规则
    #[serde(default)]
    pub context_aware_thanks: ContextAwareThanks,
    
    /// 字幕志愿者信息的最小长度阈值
    #[serde(default = "default_subtitle_volunteer_min_length")]
    pub subtitle_volunteer_min_length: usize,
}

/// 需要同时包含多个模式的组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllContainsPattern {
    pub patterns: Vec<String>,
    #[serde(default)]
    pub description: String,
}

/// 上下文相关的感谢语规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwareThanks {
    /// 是否启用上下文判断
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// 最小上下文长度（字符数）
    #[serde(default = "default_min_context_length")]
    pub min_context_length: usize,
    
    /// 感谢语模式列表
    #[serde(default)]
    pub thanks_patterns: Vec<String>,
    
    /// 上下文指示词列表（表明这是对话结尾）
    #[serde(default)]
    pub context_indicators: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_min_context_length() -> usize {
    10
}

fn default_subtitle_volunteer_min_length() -> usize {
    8
}

impl Default for ContextAwareThanks {
    fn default() -> Self {
        Self {
            enabled: true,
            min_context_length: 10,
            thanks_patterns: Vec::new(),
            context_indicators: Vec::new(),
        }
    }
}

impl Default for FilterRules {
    fn default() -> Self {
        Self {
            filter_empty: true,
            filter_brackets: true,
            single_char_fillers: Vec::new(),
            exact_matches: Vec::new(),
            contains_patterns: Vec::new(),
            all_contains_patterns: Vec::new(),
            subtitle_patterns: Vec::new(),
            meaningless_patterns: Vec::new(),
            context_aware_thanks: ContextAwareThanks::default(),
            subtitle_volunteer_min_length: 8,
        }
    }
}

impl AsrFilterConfig {
    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> EngineResult<Self> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| EngineError::new(format!("Failed to read config file: {}", e)))?;
        
        let config: AsrFilterConfig = serde_json::from_str(&content)
            .map_err(|e| EngineError::new(format!("Failed to parse config file: {}", e)))?;
        
        Ok(config)
    }
    
    /// 从默认路径加载配置
    pub fn load_default() -> EngineResult<Self> {
        // 尝试从多个可能的路径加载
        let possible_paths = vec![
            "config/asr_filters.json",
            "core/engine/config/asr_filters.json",
            "../config/asr_filters.json",
            "../../config/asr_filters.json",
        ];
        
        for path in &possible_paths {
            if Path::new(path).exists() {
                eprintln!("[ASR Filter] Loading config from: {}", path);
                return Self::load_from_file(path);
            }
        }
        
        // 如果找不到配置文件，返回默认配置
        eprintln!("[ASR Filter] ⚠️  Config file not found, using default rules");
        Ok(Self::default())
    }
    
    /// 创建默认配置（用于测试或作为后备）
    pub fn default() -> Self {
        // 这里可以返回硬编码的默认配置，或者从内嵌的 JSON 加载
        // 为了简化，我们返回一个基本的默认配置
        Self {
            version: "1.0".to_string(),
            description: "Default ASR filter rules".to_string(),
            rules: FilterRules::default(),
        }
    }
}

/// 全局配置实例（使用 Arc 实现线程安全共享）
static mut GLOBAL_CONFIG: Option<Arc<AsrFilterConfig>> = None;
static CONFIG_INIT: std::sync::Once = std::sync::Once::new();

/// 初始化全局配置
pub fn init_config(config: AsrFilterConfig) {
    unsafe {
        GLOBAL_CONFIG = Some(Arc::new(config));
    }
}

/// 尝试从文件加载并初始化全局配置
pub fn init_config_from_file() -> EngineResult<()> {
    CONFIG_INIT.call_once(|| {
        match AsrFilterConfig::load_default() {
            Ok(config) => {
                eprintln!("[ASR Filter] ✅ Config loaded successfully");
                init_config(config);
            }
            Err(e) => {
                eprintln!("[ASR Filter] ⚠️  Failed to load config: {}, using default", e);
                init_config(AsrFilterConfig::default());
            }
        }
    });
    Ok(())
}

/// 获取全局配置
pub fn get_config() -> Arc<AsrFilterConfig> {
    // 确保配置已初始化
    let _ = init_config_from_file();
    
    unsafe {
        GLOBAL_CONFIG
            .as_ref()
            .cloned()
            .unwrap_or_else(|| Arc::new(AsrFilterConfig::default()))
    }
}

