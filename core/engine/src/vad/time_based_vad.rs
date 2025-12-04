//! 基于时间的 VAD 实现
//! 
//! 这是一个简单的 VAD 实现，基于固定时间间隔来检测边界。
//! 用于测试和作为自然停顿识别的备选方案。

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::EngineResult;
use crate::types::AudioFrame;
use crate::vad::{DetectionOutcome, VoiceActivityDetector, BoundaryType};

/// 基于时间的 VAD
/// 
/// 在固定时间间隔（例如每 3 秒）检测一次边界，不考虑实际的语音活动。
pub struct TimeBasedVad {
    /// 片段时长（毫秒）
    segment_duration_ms: u64,
    /// 上一个边界的时间戳（毫秒）
    last_boundary_time: Arc<RwLock<u64>>,
}

impl TimeBasedVad {
    /// 创建新的 TimeBasedVad
    /// 
    /// # Arguments
    /// * `segment_duration_ms` - 片段时长（毫秒），例如 3000 表示每 3 秒检测一次边界
    pub fn new(segment_duration_ms: u64) -> Self {
        Self {
            segment_duration_ms,
            // 使用 u64::MAX 作为未初始化标记，避免与时间戳 0 冲突
            last_boundary_time: Arc::new(RwLock::new(u64::MAX)),
        }
    }
    
    /// 重置内部状态
    fn reset_internal(&self) {
        // 注意：这个方法不能直接使用，因为需要 async
        // 实际的重置通过 reset() 方法实现
    }
}

#[async_trait]
impl VoiceActivityDetector for TimeBasedVad {
    async fn detect(&self, frame: AudioFrame) -> EngineResult<DetectionOutcome> {
        let mut last = self.last_boundary_time.write().await;
        
        // 如果是第一帧（last == u64::MAX 表示未初始化），初始化边界时间
        if *last == u64::MAX {
            *last = frame.timestamp_ms;
            return Ok(DetectionOutcome {
                is_boundary: false,
                confidence: 1.0,
                frame,
                boundary_type: None,
            });
        }
        
        // 计算从上一个边界到现在的时长
        let elapsed = frame.timestamp_ms.saturating_sub(*last);
        
        // 如果超过或等于片段时长，判定为边界
        let is_boundary = elapsed >= self.segment_duration_ms;
        
        // 如果检测到边界，更新边界时间
        if is_boundary {
            *last = frame.timestamp_ms;
        }
        
        Ok(DetectionOutcome {
            is_boundary,
            confidence: 1.0,
            frame,
            boundary_type: if is_boundary {
                Some(BoundaryType::TimeBased)
            } else {
                None
            },
        })
    }
    
    async fn reset(&self) -> EngineResult<()> {
        let mut last = self.last_boundary_time.write().await;
        // 重置为未初始化状态
        *last = u64::MAX;
        Ok(())
    }
    
    fn get_info(&self) -> String {
        format!("TimeBasedVad(segment_duration={}ms)", self.segment_duration_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_frame(timestamp_ms: u64) -> AudioFrame {
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.0; 512],
            timestamp_ms,
        }
    }
    
    #[tokio::test]
    async fn test_time_based_detection() {
        let vad = TimeBasedVad::new(3000);  // 3秒间隔
        
        // 第一帧（0ms）- 初始化，不应该检测到边界
        let result = vad.detect(create_test_frame(0)).await.unwrap();
        assert!(!result.is_boundary, "第一帧（0ms）不应该检测到边界");
        
        // 1秒后（1000ms，从0ms开始经过1000ms < 3000ms）
        let result = vad.detect(create_test_frame(1000)).await.unwrap();
        assert!(!result.is_boundary, "1000ms时不应该检测到边界（经过1000ms < 3000ms）");
        
        // 2秒后（2000ms，从0ms开始经过2000ms < 3000ms）
        let result = vad.detect(create_test_frame(2000)).await.unwrap();
        assert!(!result.is_boundary, "2000ms时不应该检测到边界（经过2000ms < 3000ms）");
        
        // 3秒后（3000ms，从0ms开始经过3000ms >= 3000ms，应该检测到边界）
        // 注意：由于第一帧是0ms，last被设置为0，所以3000ms时elapsed = 3000 - 0 = 3000 >= 3000
        let result = vad.detect(create_test_frame(3000)).await.unwrap();
        eprintln!("DEBUG: 3000ms frame - is_boundary={}, timestamp={}", result.is_boundary, result.frame.timestamp_ms);
        assert!(result.is_boundary, "应该在3000ms时检测到边界（从0ms开始经过3000ms）");
        
        // 3.5秒后（3500ms，从上一个边界3000ms开始只有500ms < 3000ms）
        let result = vad.detect(create_test_frame(3500)).await.unwrap();
        assert!(!result.is_boundary, "3500ms时不应该检测到边界（从3000ms开始经过500ms < 3000ms）");
        
        // 6秒后（6000ms，从上一个边界3000ms开始经过3000ms >= 3000ms，应该检测到边界）
        let result = vad.detect(create_test_frame(6000)).await.unwrap();
        assert!(result.is_boundary, "应该在6000ms时检测到边界（从3000ms开始经过3000ms）");
    }
    
    #[tokio::test]
    async fn test_reset() {
        let vad = TimeBasedVad::new(3000);
        
        // 处理一些帧
        vad.detect(create_test_frame(0)).await.unwrap();
        vad.detect(create_test_frame(3000)).await.unwrap();
        
        // 重置
        vad.reset().await.unwrap();
        
        // 下一帧应该重新开始计时
        let result = vad.detect(create_test_frame(5000)).await.unwrap();
        // 重置后，last 被设置为 u64::MAX，然后第一帧（5000ms）会设置 last = 5000
        // 所以这一帧（5000ms）不会检测到边界（因为是初始化帧）
        assert!(!result.is_boundary);
    }
}
