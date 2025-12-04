//! VAD 反馈类型
//! 
//! 用于自适应阈值调整的反馈类型

/// VAD反馈类型（用于自适应阈值调整）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadFeedbackType {
    /// 边界过长：检测到音频输入但ASR长时间无输出，需要降低阈值
    BoundaryTooLong,
    /// 边界过短：ASR识别结果混乱、被过滤、或NMT翻译异常，需要提高阈值
    BoundaryTooShort,
}

