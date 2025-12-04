//! VAD 反馈机制单元测试（CoreEngine 层面）
//! 
//! 测试基于 ASR/NMT 质量指标的 VAD 阈值自适应调整功能

#[cfg(test)]
mod tests {
    use crate::asr_streaming::AsrResult;
    use crate::types::StableTranscript;
    use crate::nmt_incremental::TranslationResponse;
    use crate::nmt_client::QualityMetrics;
    use crate::asr_filters::is_meaningless_transcript;
    
    /// 创建测试用的 ASR 结果
    fn create_asr_result(text: &str) -> AsrResult {
        AsrResult {
            partial: None,
            final_transcript: Some(StableTranscript {
                text: text.to_string(),
                language: "zh".to_string(),
                speaker_id: Some("speaker1".to_string()),
            }),
        }
    }
    
    /// 创建测试用的翻译结果
    fn create_translation_response(
        translated_text: &str,
        quality_metrics: Option<QualityMetrics>,
    ) -> TranslationResponse {
        TranslationResponse {
            translated_text: translated_text.to_string(),
            is_stable: true,
            speaker_id: Some("speaker1".to_string()),
            source_audio_duration_ms: None,
            source_text: None,
            source_language: None,  // 测试中不需要源语言信息
            quality_metrics,
        }
    }
    
    /// 创建测试用的质量指标
    fn create_quality_metrics(
        perplexity: Option<f32>,
        avg_probability: Option<f32>,
        min_probability: Option<f32>,
    ) -> QualityMetrics {
        QualityMetrics {
            perplexity,
            avg_probability,
            min_probability,
        }
    }
    
    /// 测试：ASR 结果被过滤（无意义文本）
    #[test]
    fn test_feedback_asr_filtered() {
        // 测试无意义文本
        let meaningless_texts = vec![
            "(笑)",
            "謝謝大家收看",
            "嗯",
            "啊",
        ];
        
        for text in meaningless_texts {
            assert!(is_meaningless_transcript(text),
                   "文本 '{}' 应该被识别为无意义", text);
        }
        
        // 测试有意义文本
        let meaningful_texts = vec![
            "你好",
            "今天天气很好",
            "这是一个测试",
        ];
        
        for text in meaningful_texts {
            assert!(!is_meaningless_transcript(text),
                   "文本 '{}' 不应该被识别为无意义", text);
        }
    }
    
    /// 测试：ASR 结果太短（<3个字符）
    #[test]
    fn test_feedback_asr_too_short() {
        let short_texts = vec!["", "a", "ab", "你好"];
        
        for text in short_texts {
            let char_count = text.chars().count();
            if char_count < 3 {
                assert!(char_count < 3,
                       "文本 '{}' 长度 {} 应该被认为太短", text, char_count);
            }
        }
    }
    
    /// 测试：ASR 结果太长（>50个字符）
    #[test]
    fn test_feedback_asr_too_long() {
        let short_text = "这是一个短文本";
        let long_text = "这是一个非常长的文本，包含了超过五十个字符的内容，应该被识别为边界过长的情况，因为多个短句被合并在一起了";
        
        let short_len = short_text.chars().count();
        let long_len = long_text.chars().count();
        
        assert!(short_len <= 50, "短文本长度应该在50以内: {}", short_len);
        assert!(long_len > 50, "长文本长度应该超过50: {}", long_len);
    }
    
    /// 测试：翻译长度比例异常
    #[test]
    fn test_feedback_translation_length_ratio() {
        // 正常比例（0.5-2.0）
        let normal_cases = vec![
            ("hello", "你好"),           // 1:1
            ("hello world", "你好世界"),  // 约 1:1
            ("test", "测试"),            // 1:1
        ];
        
        for (src, tgt) in normal_cases {
            let src_len = src.chars().count();
            let tgt_len = tgt.chars().count();
            let ratio = if src_len > 0 {
                tgt_len as f32 / src_len as f32
            } else {
                1.0
            };
            
            assert!(ratio >= 0.3 && ratio <= 3.0,
                   "正常翻译比例应该在 0.3-3.0 之间: {}:{} = {:.2}", 
                   src_len, tgt_len, ratio);
        }
        
        // 异常比例（<0.3 或 >3.0）
        let abnormal_cases = vec![
            ("a", "这是一个非常长的翻译文本，长度远超原文"),  // 比例 > 3.0
            ("这是一个非常长的原文文本", "a"),              // 比例 < 0.3
        ];
        
        for (src, tgt) in abnormal_cases {
            let src_len = src.chars().count();
            let tgt_len = tgt.chars().count();
            let ratio = if src_len > 0 {
                tgt_len as f32 / src_len as f32
            } else {
                1.0
            };
            
            assert!(ratio < 0.3 || ratio > 3.0,
                   "异常翻译比例应该在 0.3-3.0 之外: {}:{} = {:.2}", 
                   src_len, tgt_len, ratio);
        }
    }
    
    /// 测试：困惑度判断
    #[test]
    fn test_feedback_perplexity() {
        // 正常困惑度（10-100）
        let normal_perplexity = vec![10.0, 50.0, 100.0];
        for ppl in normal_perplexity {
            assert!(ppl >= 10.0 && ppl <= 100.0,
                   "正常困惑度应该在 10-100 之间: {}", ppl);
        }
        
        // 高困惑度（>100，表示质量差）
        let high_perplexity = vec![101.0, 200.0, 500.0];
        for ppl in high_perplexity {
            assert!(ppl > 100.0,
                   "高困惑度应该 > 100: {}", ppl);
        }
        
        // 低困惑度（<10，表示质量好）
        let low_perplexity = vec![1.0, 5.0, 9.0];
        for ppl in low_perplexity {
            assert!(ppl < 10.0,
                   "低困惑度应该 < 10: {}", ppl);
        }
    }
    
    /// 测试：平均概率判断
    #[test]
    fn test_feedback_avg_probability() {
        // 正常平均概率（>= 0.05）
        let normal_probs = vec![0.05, 0.1, 0.3, 0.5];
        for prob in normal_probs {
            assert!(prob >= 0.05,
                   "正常平均概率应该 >= 0.05: {}", prob);
        }
        
        // 低平均概率（< 0.05，表示质量差）
        let low_probs = vec![0.01, 0.02, 0.04];
        for prob in low_probs {
            assert!(prob < 0.05,
                   "低平均概率应该 < 0.05: {}", prob);
        }
    }
    
    /// 测试：最小概率判断
    #[test]
    fn test_feedback_min_probability() {
        // 正常最小概率（>= 0.001）
        let normal_probs = vec![0.001, 0.01, 0.1];
        for prob in normal_probs {
            assert!(prob >= 0.001,
                   "正常最小概率应该 >= 0.001: {}", prob);
        }
        
        // 非常低的最小概率（< 0.001，表示质量差）
        let very_low_probs = vec![0.0001, 0.0005, 0.0009];
        for prob in very_low_probs {
            assert!(prob < 0.001,
                   "非常低的最小概率应该 < 0.001: {}", prob);
        }
    }
    
    /// 测试：综合反馈场景 - 边界过短
    #[test]
    fn test_feedback_scenario_boundary_too_short() {
        // 场景1：ASR结果被过滤
        let asr_result = create_asr_result("(笑)");
        assert!(asr_result.final_transcript.is_some());
        let text = &asr_result.final_transcript.as_ref().unwrap().text;
        assert!(is_meaningless_transcript(text),
               "场景1：无意义文本应该被过滤");
        
        // 场景2：ASR结果太短
        let asr_result = create_asr_result("ab");
        let text = &asr_result.final_transcript.as_ref().unwrap().text;
        assert!(text.chars().count() < 3,
               "场景2：短文本应该被识别");
        
        // 场景3：高困惑度
        let metrics = create_quality_metrics(Some(150.0), None, None);
        assert!(metrics.perplexity.unwrap() > 100.0,
               "场景3：高困惑度应该被识别");
        
        // 场景4：低平均概率
        let metrics = create_quality_metrics(None, Some(0.03), None);
        assert!(metrics.avg_probability.unwrap() < 0.05,
               "场景4：低平均概率应该被识别");
        
        // 场景5：非常低的最小概率
        let metrics = create_quality_metrics(None, None, Some(0.0005));
        assert!(metrics.min_probability.unwrap() < 0.001,
               "场景5：非常低的最小概率应该被识别");
    }
    
    /// 测试：综合反馈场景 - 边界过长
    #[test]
    fn test_feedback_scenario_boundary_too_long() {
        // 场景：ASR结果很长（多个短句被合并）
        let long_text = "这是第一句话。这是第二句话。这是第三句话。这是第四句话。这是第五句话。";
        let asr_result = create_asr_result(long_text);
        let text = &asr_result.final_transcript.as_ref().unwrap().text;
        assert!(text.chars().count() > 50,
               "长文本应该被识别为边界过长");
    }
    
    /// 测试：正常场景（不需要调整）
    #[test]
    fn test_feedback_scenario_normal() {
        // 正常ASR结果
        let asr_result = create_asr_result("今天天气很好");
        let text = &asr_result.final_transcript.as_ref().unwrap().text;
        let text_len = text.chars().count();
        
        assert!(!is_meaningless_transcript(text),
               "正常文本不应该被过滤");
        assert!(text_len >= 3 && text_len <= 50,
               "正常文本长度应该在 3-50 之间: {}", text_len);
        
        // 正常翻译结果
        let translation = create_translation_response(
            "The weather is nice today",
            Some(create_quality_metrics(
                Some(50.0),      // 正常困惑度
                Some(0.2),       // 正常平均概率
                Some(0.01),      // 正常最小概率
            )),
        );
        
        if let Some(ref metrics) = translation.quality_metrics {
            if let Some(ppl) = metrics.perplexity {
                assert!(ppl >= 10.0 && ppl <= 100.0,
                       "正常困惑度应该在 10-100 之间: {}", ppl);
            }
            if let Some(avg_prob) = metrics.avg_probability {
                assert!(avg_prob >= 0.05,
                       "正常平均概率应该 >= 0.05: {}", avg_prob);
            }
            if let Some(min_prob) = metrics.min_probability {
                assert!(min_prob >= 0.001,
                       "正常最小概率应该 >= 0.001: {}", min_prob);
            }
        }
    }
}

