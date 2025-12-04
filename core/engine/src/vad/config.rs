//! Silero VAD 配置
//! 
//! 包含 SileroVadConfig 结构体和默认配置

/// Silero VAD 配置
#[derive(Clone)]
pub struct SileroVadConfig {
    /// 模型文件路径
    pub model_path: String,
    /// 采样率（Silero VAD 要求 16kHz）
    pub sample_rate: u32,
    /// 帧大小（512 samples @ 16kHz = 32ms）
    pub frame_size: usize,
    /// 静音阈值（0.0-1.0），低于此值认为是静音
    pub silence_threshold: f32,
    /// 最小静音时长（毫秒），超过此时长才判定为自然停顿
    pub min_silence_duration_ms: u64,
    /// 是否启用自适应调整（按用户）
    #[allow(dead_code)]
    pub adaptive_enabled: bool,
    /// 自适应调整的最小样本数（每个用户至少需要这么多样本才开始调整）
    #[allow(dead_code)]
    pub adaptive_min_samples: usize,
    /// 自适应调整的速率（每次调整的幅度，0.0-1.0）
    #[allow(dead_code)]
    pub adaptive_rate: f32,
    /// 基础阈值范围（语速自适应输出的基础范围，毫秒）
    #[allow(dead_code)]
    pub base_threshold_min_ms: u64,
    /// 基础阈值范围（语速自适应输出的基础范围，毫秒）
    #[allow(dead_code)]
    pub base_threshold_max_ms: u64,
    /// Delta 偏移量范围（质量反馈偏移量，毫秒）
    #[allow(dead_code)]
    pub delta_min_ms: i64,
    /// Delta 偏移量范围（质量反馈偏移量，毫秒）
    #[allow(dead_code)]
    pub delta_max_ms: i64,
    /// 最终阈值范围（实际使用的有效范围，毫秒）
    #[allow(dead_code)]
    pub final_threshold_min_ms: u64,
    /// 最终阈值范围（实际使用的有效范围，毫秒）
    #[allow(dead_code)]
    pub final_threshold_max_ms: u64,
    /// 最小话语时长（防止半句话被切掉，毫秒）
    #[allow(dead_code)]
    pub min_utterance_ms: u64,
}

impl Default for SileroVadConfig {
    fn default() -> Self {
        Self {
            model_path: "models/vad/silero/silero_vad.onnx".to_string(),
            sample_rate: 16000,
            frame_size: 512,  // 32ms @ 16kHz
            silence_threshold: 0.2,  // 降低阈值，提高语音检测灵敏度（从 0.5 降到 0.2）
            min_silence_duration_ms: 300,  // 基础阈值（从500ms降低到300ms以更快响应）
            adaptive_enabled: true,  // 默认启用自适应
            adaptive_min_samples: 1,  // 至少1个样本（降低以更快开始调整）
            adaptive_rate: 0.4,  // 每次调整40%（提高调整速度，更快适应语速变化）
            base_threshold_min_ms: 200,  // 基础阈值范围：200-600ms（从400-800ms降低，更快响应短句）
            base_threshold_max_ms: 600,
            delta_min_ms: -200,  // Delta 偏移量范围：-200 ~ +200ms（质量反馈，降低范围）
            delta_max_ms: 200,
            final_threshold_min_ms: 200,  // 最终阈值范围：200-800ms（从300-1000ms降低，更快响应）
            final_threshold_max_ms: 800,
            min_utterance_ms: 1000,  // 最小话语时长：1000ms（降低以防止过度等待）
        }
    }
}

