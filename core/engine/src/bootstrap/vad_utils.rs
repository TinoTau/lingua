//! VAD 相关工具函数
//! 
//! 包含 VAD 阈值调整、语速更新等功能

use std::sync::Arc;

use crate::asr_streaming::AsrResult;
use crate::asr_filters::is_meaningless_transcript as is_meaningless_transcript_filter;
use crate::error::EngineError;
use crate::nmt_incremental::TranslationResponse;
use crate::types::StableTranscript;
use crate::vad::VadFeedbackType;

use super::core::CoreEngine;

impl CoreEngine {
    /// 基于ASR/NMT反馈调整VAD阈值（修订版）
    /// 
    /// # Arguments
    /// * `asr_result` - ASR识别结果
    /// * `translation_result` - NMT翻译结果（可选，StableTranscript 格式）
    /// * `translation_response` - NMT翻译响应（可选，包含质量指标）
    /// * `boundary_timestamp_ms` - VAD检测到边界的时间戳
    /// * `asr_start_timestamp_ms` - ASR开始处理的时间戳
    /// 
    /// # 修订版判断逻辑（无矛盾、去重）
    /// 1. **BoundaryTooLong（优先）**：文本过长（>50字）→ delta -= 150ms
    /// 2. **BadBoundary（合并质量异常）**：多个质量异常条件合并，只触发一次 → delta += 150ms
    /// 3. **去重逻辑**：TooLong 优先，BadBoundary 只执行一次
    pub(crate) fn adjust_vad_threshold_by_feedback(
        &self,
        asr_result: &AsrResult,
        translation_result: Option<&StableTranscript>,
        translation_response: Option<&TranslationResponse>,
        _boundary_timestamp_ms: u64,
        _asr_start_timestamp_ms: u64,
    ) {
        // 检查ASR结果
        if let Some(ref final_transcript) = asr_result.final_transcript {
            let text = &final_transcript.text;
            let text_len = text.chars().count();
            let is_filtered = is_meaningless_transcript_filter(text);
            
            // 收集所有反馈信号
            let mut is_too_long = false;
            let mut is_boundary_too_short = false;
            
            // 判断1：BoundaryTooLong（优先判断，文本过长）
            if text_len > 50 {
                eprintln!("[VAD Feedback] ⚠️  ASR result too long ({} chars), suggesting boundary may be too long (multiple sentences merged)", text_len);
                is_too_long = true;
            }
            
            // 判断2：文本被过滤（无意义文本）→ 不调整边界
            // 理由：这些文本通常是模型误识别（如"(笑)"、"詞曲:rol"），不是边界问题
            // 已过滤的文本不会影响后续处理，不需要调整边界
            // 如果调整边界，可能导致多个短句堆积，形成恶性循环
            if is_filtered {
                eprintln!("[VAD Feedback] ⚠️  ASR result filtered (meaningless), but NOT adjusting boundary (filtered text won't affect subsequent processing)");
                // 不调整边界，直接返回
                return;
            }
            
            // 判断3：BoundaryTooShort（只在明确是边界问题时才调整）
            // 只有同时满足"文本太短"和"质量异常"才判定为边界过短
            // 这样可以避免其他原因（噪音、模型问题等）导致的识别错误触发边界调整
            if !is_too_long {
                let mut has_quality_issues = false;
                
                // 3.1. 检查质量指标异常
                if let Some(ref translation_resp) = translation_response {
                    if let Some(ref metrics) = translation_resp.quality_metrics {
                        // 困惑度过高
                        if let Some(perplexity) = metrics.perplexity {
                            if perplexity > 100.0 {
                                eprintln!("[VAD Feedback] ⚠️  High perplexity ({:.2}) detected", perplexity);
                                has_quality_issues = true;
                            }
                        }
                        
                        // 平均概率过低
                        if let Some(avg_prob) = metrics.avg_probability {
                            if avg_prob < 0.05 {
                                eprintln!("[VAD Feedback] ⚠️  Low average probability ({:.4}) detected", avg_prob);
                                has_quality_issues = true;
                            }
                        }
                        
                        // 最小概率过低
                        if let Some(min_prob) = metrics.min_probability {
                            if min_prob < 0.001 {
                                eprintln!("[VAD Feedback] ⚠️  Very low min probability ({:.6}) detected", min_prob);
                                has_quality_issues = true;
                            }
                        }
                    }
                }
                
                // 3.2. 检查翻译长度比例异常
                let mut has_translation_ratio_issue = false;
                if let Some(ref translation) = translation_result {
                    let translation_len = translation.text.chars().count();
                    let length_ratio = if text_len > 0 {
                        translation_len as f32 / text_len as f32
                    } else {
                        1.0
                    };
                    
                    if length_ratio > 3.0 || length_ratio < 0.3 {
                        eprintln!("[VAD Feedback] ⚠️  Translation length ratio abnormal ({}:{} = {:.2}x) detected", 
                                 translation_len, text_len, length_ratio);
                        has_translation_ratio_issue = true;
                    }
                }
                
                // 3.3. 只有"文本太短 + 质量异常"才判定为边界过短
                // 这样可以避免其他原因导致的识别错误触发边界调整
                if text_len < 5 && (has_quality_issues || has_translation_ratio_issue) {
                    eprintln!("[VAD Feedback] ⚠️  ASR result too short ({} chars) with quality issues, suggesting boundary may be too short", text_len);
                    is_boundary_too_short = true;
                }
            }
            
            // 应用反馈调整（去重逻辑：TooLong 优先，TooShort 只执行一次）
            if is_too_long {
                // BoundaryTooLong → delta -= 150ms
                eprintln!("[VAD Feedback] ✅ Applying BoundaryTooLong feedback: delta -= 150ms");
                self.apply_vad_feedback(VadFeedbackType::BoundaryTooLong, 150);
            } else if is_boundary_too_short {
                // BoundaryTooShort → delta += 150ms（只在明确是边界问题时才调整）
                eprintln!("[VAD Feedback] 🔧 Applying BoundaryTooShort feedback (short text + quality issues, likely boundary too short): delta += 150ms");
                self.apply_vad_feedback(VadFeedbackType::BoundaryTooShort, 150);
            } else {
                eprintln!("[VAD Feedback] ℹ️  No feedback adjustment needed (text_len={}, filtered={})", text_len, is_filtered);
            }
        }
    }
    
    /// 应用 VAD 反馈调整
    pub(crate) fn apply_vad_feedback(&self, feedback_type: VadFeedbackType, adjustment_ms: i64) {
        // 尝试将VAD转换为SileroVad（使用与update_vad_speech_rate相同的方法）
        let vad_ptr = Arc::as_ptr(&self.vad);
        let silero_vad_ptr = vad_ptr as *const crate::vad::SileroVad;
        
        unsafe {
            if let Some(silero_vad) = silero_vad_ptr.as_ref() {
                silero_vad.adjust_delta_by_feedback(feedback_type, adjustment_ms);
            } else {
                eprintln!("[VAD Feedback] ⚠️  VAD is not SileroVad, cannot apply feedback adjustment");
            }
        }
    }
    
    /// 更新VAD中的全局语速（用于自适应调整）
    /// 
    /// 不区分说话者，每个短句都根据上一个短句的语速调整。
    pub(crate) fn update_vad_speech_rate(&self, text: &str, audio_duration_ms: u64) {
        eprintln!("[CoreEngine] 📝 update_vad_speech_rate called: text='{}' ({} chars), duration={}ms", 
                 text.chars().take(30).collect::<String>(), text.chars().count(), audio_duration_ms);
        
        // 尝试将 VAD 转换为 SileroVad
        let vad_ptr = Arc::as_ptr(&self.vad);
        let silero_vad_ptr = vad_ptr as *const crate::vad::SileroVad;
        
        unsafe {
            if let Some(silero_vad) = silero_vad_ptr.as_ref() {
                silero_vad.update_speech_rate(text, audio_duration_ms);
            } else {
                eprintln!("[CoreEngine] ⚠️  update_vad_speech_rate: VAD is not SileroVad, cannot update speech rate");
            }
        }
    }
    
    /// 获取全局语速（用于传递给TTS）
    pub(crate) fn get_vad_speech_rate(&self) -> Option<f32> {
        // 尝试将 VAD 转换为 SileroVad
        let vad_ptr = Arc::as_ptr(&self.vad);
        let silero_vad_ptr = vad_ptr as *const crate::vad::SileroVad;
        
        unsafe {
            if let Some(silero_vad) = silero_vad_ptr.as_ref() {
                silero_vad.get_speech_rate()
            } else {
                None
            }
        }
    }
}

