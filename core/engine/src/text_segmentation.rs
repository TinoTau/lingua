//! 文本分割模块
//! 
//! 用于将文本分割成短句，支持 TTS 增量播放

/// 文本分段信息（包含分段文本和停顿类型）
#[derive(Debug, Clone)]
pub struct TextSegment {
    pub text: String,
    pub pause_type: PauseType,
}

/// 停顿类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PauseType {
    /// 句子结束标点后的停顿（较长，如句号、问号、感叹号）
    SentenceEnd,
    /// 逗号后的停顿（较短）
    Comma,
    /// 无停顿（最后一段）
    None,
}

/// 文本分割器
pub struct TextSegmenter {
    max_sentence_length: usize,
    /// 是否在逗号处也分割（用于 TTS 停顿）
    pub split_on_comma: bool,
}

impl TextSegmenter {
    /// 创建新的文本分割器
    pub fn new(max_sentence_length: usize) -> Self {
        Self {
            max_sentence_length,
            split_on_comma: false,  // 默认不在逗号处分割（向后兼容）
        }
    }

    /// 创建新的文本分割器（支持在逗号处分割，用于 TTS）
    pub fn new_with_comma_splitting(max_sentence_length: usize) -> Self {
        Self {
            max_sentence_length,
            split_on_comma: true,  // 在逗号处也分割
        }
    }

    /// 分割文本为短句（返回带停顿类型的分段）
    /// 
    /// 分割规则：
    /// 1. 按句号、问号、感叹号分割（中英文）
    /// 2. 如果 `split_on_comma` 为 true，也在逗号处分割
    /// 3. 如果句子过长，按逗号或空格进一步分割
    /// 4. 保留标点符号
    pub fn segment_with_pause_type(&self, text: &str) -> Vec<TextSegment> {
        let text = text.trim();
        if text.is_empty() {
            return vec![];
        }

        let mut segments = Vec::new();
        let mut current_segment = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            current_segment.push(ch);

            // 检查是否为句子结束标点（句号、问号、感叹号）
            let is_sentence_end = matches!(
                ch,
                '.' | '!' | '?' | '。' | '！' | '？'
            );

            // 检查是否为逗号
            let is_comma = matches!(ch, ',' | '，');

            if is_sentence_end {
                // 对于 '.'，需要检查是否为小数点或缩写
                let should_split = if ch == '.' {
                    // 检查前后字符，判断是否为小数点
                    let prev_is_digit = current_segment
                        .chars()
                        .rev()
                        .nth(1)  // 跳过当前的 '.'
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false);
                    
                    let next_is_digit = chars.peek()
                        .map(|&c| c.is_ascii_digit())
                        .unwrap_or(false);
                    
                    // 如果前后都是数字，这是小数点，不应该分割
                    if prev_is_digit && next_is_digit {
                        false
                    } else {
                        // 检查是否为缩写（如果后面是小写字母，可能是缩写）
                        let is_abbreviation = if let Some(&next_ch) = chars.peek() {
                            next_ch.is_alphabetic() && next_ch.is_lowercase()
                        } else {
                            false
                        };
                        !is_abbreviation
                    }
                } else {
                    // 对于 '!' 和 '?'，直接分割
                    true
                };

                if should_split {
                    // 句子结束
                    let segment_text = current_segment.trim().to_string();
                    if !segment_text.is_empty() {
                        segments.push(TextSegment {
                            text: segment_text,
                            pause_type: PauseType::SentenceEnd,
                        });
                    }
                    current_segment.clear();
                    continue;
                }
            } else if is_comma && self.split_on_comma {
                // 逗号处也分割（如果启用）
                let segment_text = current_segment.trim().to_string();
                if !segment_text.is_empty() {
                    segments.push(TextSegment {
                        text: segment_text,
                        pause_type: PauseType::Comma,
                    });
                }
                current_segment.clear();
                continue;
            }

            // 如果当前句子过长，尝试在逗号处分割
            if current_segment.len() >= self.max_sentence_length {
                // 查找最后一个逗号或分号
                if let Some(last_comma_pos) = current_segment
                    .char_indices()
                    .rev()
                    .find(|(_, c)| matches!(c, ',' | ';' | '，' | '；'))
                    .map(|(pos, _)| pos)
                {
                    // 在逗号后分割
                    let first_part = current_segment[..=last_comma_pos].trim().to_string();
                    if !first_part.is_empty() {
                        segments.push(TextSegment {
                            text: first_part,
                            pause_type: PauseType::Comma,
                        });
                    }
                    current_segment = current_segment[last_comma_pos + 1..].trim().to_string();
                } else {
                    // 没有逗号，尝试在空格处分割
                    if let Some(last_space_pos) = current_segment
                        .char_indices()
                        .rev()
                        .find(|(_, c)| c.is_whitespace())
                        .map(|(pos, _)| pos)
                    {
                        let first_part = current_segment[..last_space_pos].trim().to_string();
                        if !first_part.is_empty() {
                            segments.push(TextSegment {
                                text: first_part,
                                pause_type: PauseType::None,  // 空格处不添加停顿
                            });
                        }
                        current_segment = current_segment[last_space_pos..].trim().to_string();
                    }
                }
            }
        }

        // 添加最后一个句子
        let last_segment = current_segment.trim().to_string();
        if !last_segment.is_empty() {
            segments.push(TextSegment {
                text: last_segment,
                pause_type: PauseType::None,
            });
        }

        // 如果没有任何分割，返回整个文本
        if segments.is_empty() {
            segments.push(TextSegment {
                text: text.to_string(),
                pause_type: PauseType::None,
            });
        }

        segments
    }

    /// 分割文本为短句（向后兼容的旧接口，返回 String 列表）
    /// 
    /// 分割规则：
    /// 1. 按句号、问号、感叹号分割（中英文）
    /// 2. 如果句子过长，按逗号或空格进一步分割
    /// 3. 保留标点符号
    pub fn segment(&self, text: &str) -> Vec<String> {
        self.segment_with_pause_type(text)
            .into_iter()
            .map(|s| s.text)
            .collect()
    }
}

impl Default for TextSegmenter {
    fn default() -> Self {
        Self {
            max_sentence_length: 50,
            split_on_comma: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_simple() {
        let segmenter = TextSegmenter::default();
        let text = "Hello, world. How are you? I'm fine!";
        let segments = segmenter.segment(text);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], "Hello, world.");
        assert_eq!(segments[1], "How are you?");
        assert_eq!(segments[2], "I'm fine!");
    }

    #[test]
    fn test_segment_chinese() {
        let segmenter = TextSegmenter::default();
        let text = "你好，世界。你好吗？我很好！";
        let segments = segmenter.segment(text);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], "你好，世界。");
        assert_eq!(segments[1], "你好吗？");
        assert_eq!(segments[2], "我很好！");
    }

    #[test]
    fn test_segment_long_sentence() {
        let segmenter = TextSegmenter::new(20);
        let text = "This is a very long sentence that should be split at commas or spaces.";
        let segments = segmenter.segment(text);
        assert!(segments.len() > 1);
    }

    #[test]
    fn test_segment_empty() {
        let segmenter = TextSegmenter::default();
        let segments = segmenter.segment("");
        assert!(segments.is_empty());
    }

    #[test]
    fn test_segment_no_punctuation() {
        let segmenter = TextSegmenter::default();
        let text = "Hello world";
        let segments = segmenter.segment(text);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], "Hello world");
    }

    #[test]
    fn test_segment_with_decimal_numbers() {
        let segmenter = TextSegmenter::default();
        // 测试数字中的小数点不应该被分割
        let text = "This is version 1.0. It works well.";
        let segments = segmenter.segment(text);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], "This is version 1.0.");
        assert_eq!(segments[1], "It works well.");
    }

    #[test]
    fn test_segment_with_pause_type_decimal_numbers() {
        let segmenter = TextSegmenter::default();
        // 测试带停顿类型的数字分割
        let text = "The price is 3.14 dollars. It's cheap.";
        let segments = segmenter.segment_with_pause_type(&text);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "The price is 3.14 dollars.");
        assert_eq!(segments[0].pause_type, PauseType::SentenceEnd);
        assert_eq!(segments[1].text, "It's cheap.");
        assert_eq!(segments[1].pause_type, PauseType::SentenceEnd);
    }

    #[test]
    fn test_segment_with_version_numbers() {
        let segmenter = TextSegmenter::default();
        // 测试版本号
        let text = "Version 0.26 is released. Version 1.0 is coming.";
        let segments = segmenter.segment(text);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], "Version 0.26 is released.");
        assert_eq!(segments[1], "Version 1.0 is coming.");
    }
}

