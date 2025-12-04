//! ASR 文本过滤工具
//! 
//! 用于过滤 Whisper 模型产生的无意义识别结果，如：
//! - 包含括号的文本（如 "(笑)"、"(字幕:J Chong)" 等）
//! - 视频结尾字幕（如 "謝謝大家收看" 等）
//! - 其他常见的误识别模式
//! 
//! 过滤规则从配置文件 `config/asr_filters.json` 加载，在服务启动时初始化。

pub mod config;

use config::get_config;

/// 检查文本是否为无意义的识别结果（带上下文判断）
/// 
/// 这个函数用于过滤 Whisper 模型在静音时产生的误识别文本。
/// 这些文本通常来自模型的训练数据（视频字幕），不应该被当作真实的语音输入。
/// 
/// # 过滤规则
/// 
/// 1. **单个字的无意义语气词**：说话时的填充词
///    - "嗯"、"啊"、"呃"、"额"、"哦"、"噢"、"诶"、"欸"
///    - 注意：只过滤单个字的语气词，不过滤包含这些字的其他文本（如"嗯嗯"、"啊呀"等）
/// 
/// 2. **包含括号的文本**：人类语音输入中不应该出现括号
///    - 英文括号：`()`, `[]`
///    - 中文括号：`（）`, `【】`
/// 
/// 3. **视频结尾字幕**：常见的视频结尾字幕模式
///    - "謝謝大家收看" / "谢谢大家收看"
///    - "謝謝大家觀看" / "谢谢大家观看"
///    - "thank you for watching" / "thanks for watching"
/// 
/// 4. **字幕标记**：包含字幕制作者信息的文本
///    - "(字幕:J Chong)" / "( 字幕:J Chong )"
///    - "字幕:J Chong"
///    - "詞曲:rol" / "词曲:rol"（词曲作者信息）
///    - 包含 "字幕" 和 "j chong" 的组合
///    - 包含 "詞曲:" 或 "词曲:" 的模式
/// 
/// 5. **其他无意义模式**：
///    - "titled by", "title:", "subtitle:", "source:" 等
///    - "詞曲:" / "词曲:"（词曲作者信息）
/// 
/// 6. **上下文相关的感谢语**（特殊规则）：
///    - "谢谢大家" / "感謝大家" / "感谢大家"
///    - "感谢观看" / "感謝觀看"
///    - 如果上下文为空或很短，可能是误识别（过滤）
///    - 如果上下文表明这是对话的结尾或感谢的语境，保留
/// 
/// # Arguments
/// 
/// * `text` - 要检查的文本
/// * `context` - 上下文提示（之前的识别结果），用于判断感谢语是否合理
/// 
/// # Returns
/// 
/// 返回 `true` 表示应该过滤掉（无意义），`false` 表示应该保留（有意义）
pub fn is_meaningless_transcript_with_context(text: &str, context: &str) -> bool {
    let config = get_config();
    let rules = &config.rules;
    
    let text_trimmed = text.trim();
    
    // 1. 检查空文本
    if rules.filter_empty && text_trimmed.is_empty() {
        return true;
    }
    
    // 2. 检查单个字的无意义语气词
    if rules.single_char_fillers.contains(&text_trimmed.to_string()) {
        return true;
    }
    
    // 3. 检查括号
    if rules.filter_brackets {
        if text_trimmed.contains('(') || text_trimmed.contains(')') 
            || text_trimmed.contains('（') || text_trimmed.contains('）')
            || text_trimmed.contains('[') || text_trimmed.contains(']')
            || text_trimmed.contains('【') || text_trimmed.contains('】') {
            return true;
        }
    }
    
    let text_lower = text_trimmed.to_lowercase();
    let context_lower = context.trim().to_lowercase();
    
    // 4. 检查上下文相关的感谢语
    if rules.context_aware_thanks.enabled {
        let is_thanks_text = rules.context_aware_thanks.thanks_patterns.iter()
            .any(|pattern| text_lower == pattern.to_lowercase() || text_lower.starts_with(&pattern.to_lowercase()));
        
        if is_thanks_text {
            if context_lower.is_empty() || context_lower.chars().count() < rules.context_aware_thanks.min_context_length {
                eprintln!("[ASR Filter] ⚠️  Filtering thanks text without context: \"{}\"", text_trimmed);
                return true;
            }
            
            let has_context_indicator = rules.context_aware_thanks.context_indicators.iter()
                .any(|indicator| context_lower.contains(&indicator.to_lowercase()));
            
            if !has_context_indicator {
                eprintln!("[ASR Filter] ⚠️  Filtering thanks text without context indicator: \"{}\" (context: \"{}\")", 
                         text_trimmed, context.chars().take(50).collect::<String>());
                return true;
            }
            
            eprintln!("[ASR Filter] ✅ Keeping thanks text with valid context: \"{}\"", text_trimmed);
        }
    }
    
    // 5. 检查精确匹配
    for pattern in &rules.exact_matches {
        if text_trimmed.eq_ignore_ascii_case(pattern) {
            return true;
        }
    }
    
    // 6. 检查部分匹配模式
    for pattern in &rules.contains_patterns {
        if text_lower.contains(&pattern.to_lowercase()) {
            return true;
        }
    }
    
    // 7. 检查需要同时包含多个模式的组合
    for all_contains in &rules.all_contains_patterns {
        if all_contains.patterns.iter().all(|p| text_lower.contains(&p.to_lowercase())) {
            return true;
        }
    }
    
    // 8. 检查字幕相关模式
    if text_lower.contains("字幕") {
        for pattern in &rules.subtitle_patterns {
            if text_lower.contains(&pattern.to_lowercase()) {
                return true;
            }
        }
        
        // 检查字幕志愿者信息（更具体的模式，包含名字）
        if text_lower.contains("中文字幕志愿者") 
            || text_lower.contains("中文字幕志願者")
            || (text_lower.contains("字幕志愿者") && text_lower.chars().count() > rules.subtitle_volunteer_min_length) {
            return true;
        }
    }
    
    // 9. 检查无意义模式（需要进一步检查是否在括号内）
    for pattern in &rules.meaningless_patterns {
        if text_lower.contains(&pattern.to_lowercase()) {
            let pattern_pos = text_lower.find(&pattern.to_lowercase());
            if let Some(pos) = pattern_pos {
                let before = if pos > 0 { &text_lower[..pos] } else { "" };
                let after = if pos + pattern.len() < text_lower.len() { &text_lower[pos + pattern.len()..] } else { "" };
                
                let has_open_bracket = before.chars().rev().take(10).any(|c| matches!(c, '(' | '[' | '（'));
                let has_close_bracket = after.chars().take(50).any(|c| matches!(c, ')' | ']' | '）'));
                
                if has_open_bracket || has_close_bracket {
                    return true;
                }
            }
        }
    }
    
    false
}

/// 检查文本是否为无意义的识别结果（不带上下文，向后兼容）
/// 
/// 这个函数调用 `is_meaningless_transcript_with_context`，传入空上下文。
/// 
/// # Arguments
/// 
/// * `text` - 要检查的文本
/// 
/// # Returns
/// 
/// 返回 `true` 表示应该过滤掉（无意义），`false` 表示应该保留（有意义）
pub fn is_meaningless_transcript(text: &str) -> bool {
    is_meaningless_transcript_with_context(text, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brackets_filtering() {
        assert!(is_meaningless_transcript("(笑)"));
        assert!(is_meaningless_transcript("(字幕:J Chong)"));
        assert!(is_meaningless_transcript("（笑）"));
        assert!(is_meaningless_transcript("[字幕]"));
        assert!(is_meaningless_transcript("【字幕】"));
        assert!(!is_meaningless_transcript("你好"));
    }

    #[test]
    fn test_video_end_subtitles() {
        assert!(is_meaningless_transcript("謝謝大家收看"));
        assert!(is_meaningless_transcript("谢谢大家收看"));
        assert!(is_meaningless_transcript("thank you for watching"));
        assert!(is_meaningless_transcript("Thanks for watching"));
        assert!(!is_meaningless_transcript("谢谢你的帮助"));
    }

    #[test]
    fn test_subtitle_markers() {
        assert!(is_meaningless_transcript("(字幕:J Chong)"));
        assert!(is_meaningless_transcript("字幕:J Chong"));
        assert!(is_meaningless_transcript("字幕 j chong"));
        assert!(is_meaningless_transcript("中文字幕——YK"));
        assert!(is_meaningless_transcript("字幕志愿者 杨茜茜"));
        assert!(is_meaningless_transcript("字幕——YK"));
        assert!(is_meaningless_transcript("字幕 - YK"));
        assert!(is_meaningless_transcript("字幕制作者 ABC"));
        assert!(is_meaningless_transcript("詞曲:rol"));
        assert!(is_meaningless_transcript("词曲:rol"));
        assert!(is_meaningless_transcript("詞曲:ROL"));
        assert!(is_meaningless_transcript("词曲:abc"));
        assert!(!is_meaningless_transcript("这是字幕"));
    }

    #[test]
    fn test_empty_text() {
        assert!(is_meaningless_transcript(""));
        assert!(is_meaningless_transcript("   "));
        assert!(!is_meaningless_transcript("你好世界"));
    }

    #[test]
    fn test_filler_words() {
        // 单个字的语气词应该被过滤
        assert!(is_meaningless_transcript("嗯"));
        assert!(is_meaningless_transcript("啊"));
        assert!(is_meaningless_transcript("呃"));
        assert!(is_meaningless_transcript("额"));
        assert!(is_meaningless_transcript("哦"));
        assert!(is_meaningless_transcript("噢"));
        assert!(is_meaningless_transcript("诶"));
        assert!(is_meaningless_transcript("欸"));
        
        // 包含语气词但不是单独一个字的应该保留
        assert!(!is_meaningless_transcript("嗯嗯"));
        assert!(!is_meaningless_transcript("啊呀"));
        assert!(!is_meaningless_transcript("呃呃"));
        assert!(!is_meaningless_transcript("嗯，好的"));
        assert!(!is_meaningless_transcript("啊，我明白了"));
    }
}

