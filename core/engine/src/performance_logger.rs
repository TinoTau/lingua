//! 性能日志记录模块
//! 
//! 用于记录端到端请求的耗时和质量指标

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// 性能日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceLog {
    /// 时间戳（ISO 8601）
    pub ts: String,
    /// 请求 ID
    pub id: String,
    /// 源语言
    pub src_lang: String,
    /// 目标语言
    pub tgt_lang: String,
    /// ASR 耗时（毫秒）
    pub asr_ms: u64,
    /// NMT 耗时（毫秒）
    pub nmt_ms: u64,
    /// TTS 耗时（毫秒）
    pub tts_ms: u64,
    /// 总耗时（毫秒）
    pub total_ms: u64,
    /// 是否成功
    pub ok: bool,
    /// 是否可疑翻译
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suspect_translation: Option<bool>,
    /// 原文长度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_text_len: Option<usize>,
    /// 译文长度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tgt_text_len: Option<usize>,
}

impl PerformanceLog {
    /// 创建新的性能日志
    pub fn new(
        id: String,
        src_lang: String,
        tgt_lang: String,
        asr_ms: u64,
        nmt_ms: u64,
        tts_ms: u64,
        total_ms: u64,
        ok: bool,
    ) -> Self {
        // 生成 ISO 8601 格式时间戳（不使用 chrono 依赖）
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        let secs = now.as_secs();
        let ts_iso = format_iso8601(secs);

        Self {
            ts: ts_iso,
            id,
            src_lang,
            tgt_lang,
            asr_ms,
            nmt_ms,
            tts_ms,
            total_ms,
            ok,
            suspect_translation: None,
            src_text_len: None,
            tgt_text_len: None,
        }
    }

    /// 检查是否为可疑翻译
    pub fn check_suspect_translation(&mut self, src_text: &str, tgt_text: &str) {
        self.src_text_len = Some(src_text.len());
        self.tgt_text_len = Some(tgt_text.len());

        // 规则 1: 原文长度 > 20 字符，而译文长度 < 3 字符
        if src_text.len() > 20 && tgt_text.len() < 3 {
            self.suspect_translation = Some(true);
            return;
        }

        // 规则 2: 译文中非字母/非汉字字符比例 > 70%
        let non_alpha_count = tgt_text
            .chars()
            .filter(|c| !c.is_alphabetic() && !c.is_alphanumeric() && !is_chinese_char(*c))
            .count();
        let non_alpha_ratio = if tgt_text.is_empty() {
            1.0
        } else {
            non_alpha_count as f64 / tgt_text.len() as f64
        };

        if non_alpha_ratio > 0.7 {
            self.suspect_translation = Some(true);
            return;
        }

        self.suspect_translation = Some(false);
    }

    /// 格式化为 JSON 字符串
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// 检查是否为中文字符
fn is_chinese_char(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF)
}

/// 格式化 Unix 时间戳为 ISO 8601 格式（简化版本）
fn format_iso8601(secs: u64) -> String {
    // 使用 UTC 时间（简化实现）
    // 注意：这是一个简化版本，不考虑时区偏移和闰秒
    let days = secs / 86400;
    let secs_in_day = secs % 86400;
    
    // 从 1970-01-01 开始计算
    let mut year = 1970;
    let mut day_of_year = days as i64;
    
    // 计算年份（考虑闰年）
    while day_of_year >= 365 {
        let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };
        if day_of_year >= days_in_year {
            year += 1;
            day_of_year -= days_in_year;
        } else {
            break;
        }
    }
    
    // 计算月日（使用固定月份天数）
    let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let mut month = 1;
    let mut day = day_of_year as u32 + 1;
    
    for &days_in_month in &month_days {
        let days = if month == 2 && is_leap { days_in_month + 1 } else { days_in_month };
        if day > days {
            day -= days;
            month += 1;
        } else {
            break;
        }
    }
    
    let hour = (secs_in_day / 3600) as u32;
    let minute = ((secs_in_day % 3600) / 60) as u32;
    let second = (secs_in_day % 60) as u32;
    
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hour, minute, second)
}

/// 性能日志记录器
pub struct PerformanceLogger {
    enabled: bool,
    log_suspect: bool,
}

impl PerformanceLogger {
    pub fn new(enabled: bool, log_suspect: bool) -> Self {
        Self {
            enabled,
            log_suspect,
        }
    }

    /// 记录性能日志
    pub fn log(&self, log: &PerformanceLog) {
        if !self.enabled {
            return;
        }

        // 输出 JSON 格式日志
        println!("[PERF] {}", log.to_json());

        // 如果可疑翻译且启用，额外输出警告
        if self.log_suspect {
            if let Some(true) = log.suspect_translation {
                eprintln!(
                    "[WARN] Suspect translation detected: id={}, src_len={}, tgt_len={}",
                    log.id,
                    log.src_text_len.unwrap_or(0),
                    log.tgt_text_len.unwrap_or(0)
                );
            }
        }
    }
}

impl Default for PerformanceLogger {
    fn default() -> Self {
        Self {
            enabled: true,
            log_suspect: true,
        }
    }
}

