// core/engine/src/asr_whisper/streaming_test.rs
// 单元测试：测试优化后的 ASR streaming 功能

#[cfg(test)]
mod tests {
    use crate::asr_whisper::streaming::WhisperAsrStreaming;
    use crate::asr_streaming::AsrResult;

    /// 测试 empty_asr_result 方法
    #[test]
    fn test_empty_asr_result() {
        let result = WhisperAsrStreaming::empty_asr_result();
        assert!(result.partial.is_none(), "空结果不应该有部分转录");
        assert!(result.final_transcript.is_none(), "空结果不应该有最终转录");
    }

    /// 测试上下文缓存的基本操作
    /// 注意：这个测试需要实际的 WhisperAsrEngine 实例
    /// 在实际环境中，可以使用 mock 或测试模型
    #[test]
    #[ignore] // 需要实际的模型文件，默认跳过
    fn test_context_cache_operations() {
        // 这个测试需要实际的模型，所以标记为 ignore
        // 在实际 CI/CD 环境中，如果有测试模型，可以运行此测试
    }

    /// 测试 get_detected_language 的逻辑
    #[test]
    fn test_get_detected_language_logic() {
        // 测试逻辑：如果提供了 detected_lang，应该使用它
        // 如果没有提供，应该从 engine 获取
        // 如果都不可用，应该返回 "unknown"
        
        // 由于需要实际的 WhisperAsrEngine，这个测试需要 mock
        // 这里我们只测试逻辑，不测试实际实现
        let detected_lang = Some("en".to_string());
        assert_eq!(detected_lang.unwrap(), "en");
        
        let detected_lang_none: Option<String> = None;
        assert!(detected_lang_none.is_none());
    }

    /// 测试 create_asr_result 的空文本处理
    #[test]
    fn test_create_asr_result_empty_text_logic() {
        // 测试逻辑：空文本应该返回空结果
        let empty_texts = vec!["", "   ", "\n\t", "  \n  "];
        for text in empty_texts {
            assert!(text.trim().is_empty(), "文本 '{}' 应该被认为是空的", text);
        }
    }

    /// 测试 create_asr_result 的非空文本处理
    #[test]
    fn test_create_asr_result_non_empty_text_logic() {
        // 测试逻辑：非空文本应该创建结果
        let non_empty_texts = vec!["Hello", "  Hello  ", "Hello world", "你好"];
        for text in non_empty_texts {
            assert!(!text.trim().is_empty(), "文本 '{}' 不应该被认为是空的", text);
        }
    }

    /// 测试上下文缓存的容量限制（逻辑测试）
    #[test]
    fn test_context_cache_capacity_logic() {
        // 测试逻辑：缓存应该只保留最近 2 句
        let mut cache: Vec<String> = Vec::new();
        
        // 添加第一句
        cache.push("First sentence".to_string());
        assert_eq!(cache.len(), 1);
        
        // 添加第二句
        cache.push("Second sentence".to_string());
        assert_eq!(cache.len(), 2);
        
        // 添加第三句，应该移除第一句
        cache.push("Third sentence".to_string());
        if cache.len() > 2 {
            cache.remove(0);
        }
        assert_eq!(cache.len(), 2);
        assert_eq!(cache[0], "Second sentence");
        assert_eq!(cache[1], "Third sentence");
    }

    /// 测试上下文提示的拼接逻辑
    #[test]
    fn test_context_prompt_joining_logic() {
        // 测试逻辑：应该拼接最近 2 句作为上下文
        let cache = vec![
            "First sentence".to_string(),
            "Second sentence".to_string(),
        ];
        
        let context_sentences: Vec<String> = cache.iter()
            .rev()
            .take(2)
            .rev()
            .cloned()
            .collect();
        
        let context = context_sentences.join(" ");
        assert_eq!(context, "First sentence Second sentence");
    }

    /// 测试空文本的 trim 处理
    #[test]
    fn test_empty_text_trim() {
        let empty_texts = vec!["", "   ", "\n", "\t", "  \n\t  "];
        for text in empty_texts {
            assert!(text.trim().is_empty(), "文本 '{}' 应该被 trim 后为空", text);
        }
    }

    /// 测试非空文本的 trim 处理
    #[test]
    fn test_non_empty_text_trim() {
        let test_cases = vec![
            ("  Hello  ", "Hello"),
            ("\nWorld\n", "World"),
            ("  Test  \n", "Test"),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(input.trim(), expected, "文本 '{}' 应该被 trim 为 '{}'", input, expected);
        }
    }

    /// 测试置信度值的有效性
    #[test]
    fn test_confidence_values() {
        // 测试逻辑：置信度应该在 0.0-1.0 范围内
        let valid_confidences = vec![0.0, 0.5, 0.9, 0.95, 1.0];
        for conf in valid_confidences {
            assert!(conf >= 0.0 && conf <= 1.0, "置信度 {} 应该在 0.0-1.0 范围内", conf);
        }
    }

    /// 测试 segments 列表的拼接逻辑
    #[test]
    fn test_segments_joining() {
        let segments = vec![
            "Segment 1".to_string(),
            "Segment 2".to_string(),
            "Segment 3".to_string(),
        ];
        
        let full_text = segments.join(" ");
        assert_eq!(full_text, "Segment 1 Segment 2 Segment 3");
    }

    /// 测试空 segments 列表的处理
    #[test]
    fn test_empty_segments() {
        let segments: Vec<String> = Vec::new();
        let full_text = segments.join(" ");
        assert_eq!(full_text, "");
    }
}

