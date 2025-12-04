mod time_based_vad;
mod silero_vad;
mod config;
mod adaptive_state;
mod feedback;

#[cfg(test)]
mod vad_feedback_test;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;
use crate::types::AudioFrame;

pub use time_based_vad::TimeBasedVad;
pub use silero_vad::SileroVad;
pub use config::SileroVadConfig;
// SpeakerAdaptiveState is crate-private, not exported
pub use feedback::VadFeedbackType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionOutcome {
    /// 是否检测到边界（自然停顿或强制截断）
    pub is_boundary: bool,
    /// 检测置信度（0.0-1.0）
    pub confidence: f32,
    /// 音频帧
    pub frame: AudioFrame,
    /// 边界类型（为自然停顿识别优化预留）
    /// None = 未检测到边界
    /// Some(BoundaryType::NaturalPause) = 自然停顿
    /// Some(BoundaryType::ForcedCutoff) = 强制截断（超过最大缓冲区）
    /// Some(BoundaryType::TimeBased) = 基于时间的截断（TimeBasedVad）
    pub boundary_type: Option<BoundaryType>,
}

/// 边界类型（为自然停顿识别优化预留）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BoundaryType {
    /// 自然停顿（通过 VAD 检测到的静音）
    NaturalPause,
    /// 强制截断（超过最大缓冲区）
    ForcedCutoff,
    /// 基于时间的截断
    TimeBased,
    /// 其他类型（可扩展）
    Other(String),
}

#[async_trait]
pub trait VoiceActivityDetector: Send + Sync {
    /// 检测语音活动和边界
    /// 
    /// # Arguments
    /// * `frame` - 音频帧
    /// 
    /// # Returns
    /// 返回检测结果，包含是否检测到边界、置信度和边界类型
    async fn detect(&self, frame: AudioFrame) -> EngineResult<DetectionOutcome>;
    
    /// 重置检测器状态（为新的会话或流式处理预留）
    /// 
    /// 默认实现为空操作，子类可以覆盖以实现状态重置
    async fn reset(&self) -> EngineResult<()> {
        Ok(())
    }
    
    /// 获取检测器配置信息（为调试和监控预留）
    /// 
    /// 返回一个描述性的字符串，包含检测器类型和配置参数
    fn get_info(&self) -> String {
        "Unknown VAD".to_string()
    }
}
