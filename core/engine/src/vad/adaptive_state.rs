//! 自适应状态管理
//! 
//! 包含 SpeakerAdaptiveState 及其实现，用于管理语速历史和阈值调整

use std::collections::VecDeque;

use super::config::SileroVadConfig;

/// 每个说话者的自适应状态
pub(crate) struct SpeakerAdaptiveState {
    /// 语速历史（字符/秒）
    pub(crate) speech_rate_history: VecDeque<f32>,
    /// 基础阈值（由语速自适应生成，毫秒）
    pub(crate) base_threshold_ms: u64,
    /// Delta 偏移量（由质量反馈生成，毫秒）
    pub(crate) delta_ms: i64,
    /// 样本数量
    pub(crate) sample_count: usize,
}

impl SpeakerAdaptiveState {
    pub(crate) fn new(base_duration_ms: u64) -> Self {
        eprintln!("[SileroVad] 🆕 Initialized SpeakerAdaptiveState with base_duration_ms={}ms", base_duration_ms);
        Self {
            speech_rate_history: VecDeque::with_capacity(20),  // 保留最近20个样本
            base_threshold_ms: base_duration_ms,
            delta_ms: 0,  // 初始 delta 为 0
            sample_count: 0,
        }
    }
    
    /// 更新语速并调整阈值
    /// 
    /// 使用更精细的语速调整策略：
    /// - 根据语速动态计算阈值倍数（连续函数，而非分段函数）
    /// - 快语速 → 更短的阈值（说话者句子之间停顿短）
    /// - 慢语速 → 更长的阈值（说话者可能在句子中间思考停顿）
    pub(crate) fn update_speech_rate(&mut self, speech_rate: f32, config: &SileroVadConfig) {
        self.speech_rate_history.push_back(speech_rate);
        if self.speech_rate_history.len() > 20 {
            self.speech_rate_history.pop_front();
        }
        self.sample_count += 1;
        
        // 即使样本数不足，也允许使用当前语速进行快速调整（降低延迟）
        let history_len = self.speech_rate_history.len();
        let avg_speech_rate = if history_len > 0 {
            // 使用加权平均（最近的值权重更高）
            let mut weighted_sum = 0.0;
            let mut total_weight = 0.0;
            for (i, &rate) in self.speech_rate_history.iter().enumerate() {
                let weight = (i + 1) as f32;  // 越新的值权重越大
                weighted_sum += rate * weight;
                total_weight += weight;
            }
            weighted_sum / total_weight
        } else {
            speech_rate
        };
        
        // 即使样本数不足，也允许进行快速调整（使用当前语速）
        // 这样可以更快地响应语速变化，减少多个短句被合并的情况
        
        // 根据语速动态计算阈值倍数（使用连续函数，而非分段函数）
        // 语速范围：0-20 字符/秒（正常范围：3-12 字符/秒）
        // 目标：快语速（> 8 字符/秒）→ 更短的阈值，慢语速（< 4 字符/秒）→ 更长的阈值
        
        // 使用 sigmoid 函数将语速映射到阈值倍数
        // sigmoid(x) = 1 / (1 + e^(-x))
        // 调整后的 sigmoid：sigmoid((rate - 6) / 2) * 2 - 1，映射到 [0, 2] 范围
        // - 语速 = 2 字符/秒 → multiplier ≈ 1.4（延长40%）
        // - 语速 = 6 字符/秒 → multiplier ≈ 1.0（保持原值）
        // - 语速 = 10 字符/秒 → multiplier ≈ 0.6（缩短40%）
        
        // 将语速映射到 [-2, 2] 范围（sigmoid 的有效范围）
        let normalized_rate = (avg_speech_rate - 6.0) / 2.0;
        let sigmoid_value = 1.0 / (1.0 + (-normalized_rate).exp());
        // 将 sigmoid 值 [0, 1] 映射到 [0.6, 1.4] 范围（阈值倍数）
        let multiplier = 0.6 + (sigmoid_value * 0.8);
        
        // 当 sigmoid_value = 0.5（语速 = 6）时，multiplier = 1.0
        // 当 sigmoid_value < 0.5（语速 < 6，慢语速）时，multiplier > 1.0
        // 当 sigmoid_value > 0.5（语速 > 6，快语速）时，multiplier < 1.0
        
        // 应用调整（使用平滑更新）- 只调整 base_threshold
        let base_threshold_center = (config.base_threshold_min_ms + config.base_threshold_max_ms) / 2;
        let target_base = (base_threshold_center as f32 * multiplier) as u64;
        let old_base = self.base_threshold_ms;
        let adjustment = (target_base as f32 - self.base_threshold_ms as f32) * config.adaptive_rate;
        self.base_threshold_ms = ((self.base_threshold_ms as f32 + adjustment) as u64)
            .clamp(config.base_threshold_min_ms, config.base_threshold_max_ms);
        
        // 记录阈值调整（仅在阈值变化较大时记录，避免日志过多）
        let change_ratio = if old_base > 0 {
            (self.base_threshold_ms as f32 - old_base as f32) / old_base as f32
        } else {
            0.0
        };
        if change_ratio.abs() > 0.1 {  // 变化超过10%时记录
            let effective = self.get_effective_threshold(config);
            eprintln!("[SileroVad] 🔧 Threshold adjusted: {}ms -> {}ms (target: {}ms, multiplier: {:.2}, avg_rate: {:.2} chars/s, effective: {}ms, change: {:.1}%)", 
                     old_base, self.base_threshold_ms, target_base, multiplier, avg_speech_rate, effective, change_ratio * 100.0);
        }
    }
    
    /// 获取有效阈值（base + delta，限制在最终范围内）
    pub(crate) fn get_effective_threshold(&self, config: &SileroVadConfig) -> u64 {
        let effective = (self.base_threshold_ms as i64 + self.delta_ms) as u64;
        effective.clamp(config.final_threshold_min_ms, config.final_threshold_max_ms)
    }
    
    /// 获取当前调整后的阈值（兼容旧接口）
    pub(crate) fn get_adjusted_duration(&self, config: &SileroVadConfig) -> u64 {
        // 即使样本数不足，也使用调整后的阈值（如果已经调整过）
        // 这样可以更快地响应语速变化，减少多个短句被合并的情况
        if self.sample_count == 0 {
            config.min_silence_duration_ms
        } else {
            self.get_effective_threshold(config)
        }
    }
    
    /// 获取平均语速
    pub(crate) fn get_avg_speech_rate(&self) -> Option<f32> {
        if self.speech_rate_history.is_empty() {
            None
        } else {
            Some(self.speech_rate_history.iter().sum::<f32>() / self.speech_rate_history.len() as f32)
        }
    }
}

