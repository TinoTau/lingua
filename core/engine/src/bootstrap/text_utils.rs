//! 文本处理工具函数
//! 
//! 包含文本分割、转换等辅助函数

impl super::core::CoreEngine {
    /// 将文本按句子边界分割（支持中英文标点，以及无标点情况）
    /// 
    /// 分割规则：
    /// 1. 优先按句号、问号、感叹号分割（中英文）
    /// 2. 如果没有标点，按常见疑问词或语气词分割（中文：吗、呢、吧等）
    /// 3. 如果仍然无法分割，按长度和语义分割（每15-20个字符一个句子）
    /// 4. 保留标点符号
    /// 5. 过滤空句子
    pub(crate) fn split_into_sentences(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current_sentence = String::new();
        let mut has_punctuation = false;
        
        // 第一遍：按标点符号分割
        for ch in text.chars() {
            current_sentence.push(ch);
            
            // 检查是否为句子结束标点（句号、问号、感叹号）
            let is_sentence_end = matches!(
                ch,
                '.' | '!' | '?' | '。' | '！' | '？'
            );
            
            if is_sentence_end {
                has_punctuation = true;
                let trimmed = current_sentence.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current_sentence.clear();
            }
        }
        
        // 处理最后一个句子（如果没有结束标点）
        let trimmed = current_sentence.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }
        
        // 如果没有找到标点符号，尝试按语气词或疑问词分割（中文）
        if !has_punctuation && sentences.len() == 1 {
            let text = sentences[0].clone();
            sentences.clear();
            
            // 按常见的中文语气词/疑问词分割：吗、呢、吧、啊、呀、哦、嗯等
            let mut current = String::new();
            let mut chars = text.chars().peekable();
            
            while let Some(ch) = chars.next() {
                current.push(ch);
                
                // 检查是否为语气词/疑问词（后面通常跟空格或结束）
                let is_sentence_end = matches!(ch, '吗' | '呢' | '吧' | '啊' | '呀' | '哦');
                let next_is_space_or_end = chars.peek().map(|&c| c.is_whitespace()).unwrap_or(true);
                
                if is_sentence_end && next_is_space_or_end {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        sentences.push(trimmed);
                    }
                    current.clear();
                    // 跳过空格
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
            }
            
            // 处理剩余部分
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
        }
        
        // 如果仍然只有一个句子，按长度和空格分割（每15-20个字符一个句子）
        if sentences.len() == 1 {
            let text = sentences[0].clone();
            let char_count = text.chars().count();
            
            // 如果文本较长（>20个字符），尝试按空格或语义分割
            if char_count > 20 {
                sentences.clear();
                let words: Vec<&str> = text.split_whitespace().collect();
                let mut current_sentence = String::new();
                let mut current_length = 0;
                const MAX_SENTENCE_LENGTH: usize = 20; // 每个句子最多20个字符
                
                for word in words {
                    let word_length = word.chars().count();
                    if current_length + word_length > MAX_SENTENCE_LENGTH && !current_sentence.is_empty() {
                        sentences.push(current_sentence.trim().to_string());
                        current_sentence.clear();
                        current_length = 0;
                    }
                    if !current_sentence.is_empty() {
                        current_sentence.push(' ');
                    }
                    current_sentence.push_str(word);
                    current_length += word_length + 1; // +1 for space
                }
                
                if !current_sentence.is_empty() {
                    sentences.push(current_sentence.trim().to_string());
                }
            }
        }
        
        // 过滤空句子
        sentences.retain(|s| !s.trim().is_empty());
        
        sentences
    }

    /// 细粒度分割文本（用于处理无标点的长文本）
    /// 
    /// 分割规则：
    /// 1. 按空格分割成单词
    /// 2. 每10-15个字符组成一个句子
    /// 3. 尽量在语义边界处分割（如：疑问词、语气词后）
    pub(crate) fn split_into_sentences_fine_grained(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        if words.is_empty() {
            return sentences;
        }
        
        let mut current_sentence = String::new();
        let mut current_length = 0;
        const TARGET_SENTENCE_LENGTH: usize = 12; // 目标句子长度（字符数）
        const MAX_SENTENCE_LENGTH: usize = 18;    // 最大句子长度
        
        for word in words {
            let word_length = word.chars().count();
            
            // 检查是否应该在当前单词后分割（如果当前句子已经足够长）
            let should_split_after = current_length > 0 && 
                (current_length + word_length > MAX_SENTENCE_LENGTH ||
                 (current_length >= TARGET_SENTENCE_LENGTH && 
                  // 检查单词是否以语气词/疑问词结尾（中文）
                  (word.ends_with('吗') || word.ends_with('呢') || word.ends_with('吧') || 
                   word.ends_with('啊') || word.ends_with('呀') || word.ends_with('哦'))));
            
            if should_split_after && !current_sentence.is_empty() {
                let trimmed = current_sentence.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current_sentence.clear();
                current_length = 0;
            }
            
            if !current_sentence.is_empty() {
                current_sentence.push(' ');
            }
            current_sentence.push_str(word);
            current_length += word_length + if current_sentence.ends_with(' ') { 0 } else { 1 };
        }
        
        // 处理最后一个句子
        let trimmed = current_sentence.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }
        
        // 如果仍然只有一个句子，强制按固定长度分割
        if sentences.len() == 1 && text.chars().count() > 20 {
            sentences.clear();
            let chars: Vec<char> = text.chars().collect();
            let mut start = 0;
            while start < chars.len() {
                let end = (start + TARGET_SENTENCE_LENGTH).min(chars.len());
                let sentence: String = chars[start..end].iter().collect();
                let trimmed = sentence.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                start = end;
                // 跳过空格
                while start < chars.len() && chars[start].is_whitespace() {
                    start += 1;
                }
            }
        }
        
        // 过滤空句子
        sentences.retain(|s| !s.trim().is_empty());
        sentences
    }

    /// 将数字转换为中文（用于TTS输出）
    /// 
    /// 例如：123 -> "一百二十三"
    pub(crate) fn convert_decimals_to_chinese(text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch.is_ascii_digit() {
                // 开始读取数字
                let mut num_str = String::new();
                num_str.push(ch);
                
                // 检查下一个字符是否是小数点
                let mut has_decimal = false;
                if let Some(&'.') = chars.peek() {
                    chars.next(); // 跳过小数点
                    has_decimal = true;
                    num_str.push('.');
                    
                    // 读取小数点后的数字
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_ascii_digit() {
                            num_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                } else {
                    // 继续读取整数部分
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_ascii_digit() {
                            num_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                }
                
                // 如果是小数，转换为中文读法
                if has_decimal && num_str.contains('.') {
                    let parts: Vec<&str> = num_str.split('.').collect();
                    if parts.len() == 2 {
                        // 转换整数部分
                        let int_part = parts[0];
                        let mut chinese_num = String::new();
                        for digit_char in int_part.chars() {
                            if let Some(digit) = digit_char.to_digit(10) {
                                chinese_num.push_str(match digit {
                                    0 => "零",
                                    1 => "一",
                                    2 => "二",
                                    3 => "三",
                                    4 => "四",
                                    5 => "五",
                                    6 => "六",
                                    7 => "七",
                                    8 => "八",
                                    9 => "九",
                                    _ => "",
                                });
                            }
                        }
                        
                        // 添加"点"
                        chinese_num.push_str("点");
                        
                        // 转换小数部分
                        for digit_char in parts[1].chars() {
                            if let Some(digit) = digit_char.to_digit(10) {
                                chinese_num.push_str(match digit {
                                    0 => "零",
                                    1 => "一",
                                    2 => "二",
                                    3 => "三",
                                    4 => "四",
                                    5 => "五",
                                    6 => "六",
                                    7 => "七",
                                    8 => "八",
                                    9 => "九",
                                    _ => "",
                                });
                            }
                        }
                        
                        result.push_str(&chinese_num);
                        continue;
                    }
                }
                
                // 如果不是小数，保持原样
                result.push_str(&num_str);
            } else {
                result.push(ch);
            }
        }
        
        result
    }
}

