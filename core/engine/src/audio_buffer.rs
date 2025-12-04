use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::EngineResult;
use crate::types::AudioFrame;

/// 音频缓冲管理器
/// 
/// 支持双缓冲机制，用于连续输入输出场景：
/// - 当前缓冲区：累积音频帧，等待 VAD 检测到边界
/// - 备用缓冲区：接收新的音频输入
/// 
/// 当检测到边界时，提交当前缓冲区的内容给 ASR，同时切换到备用缓冲区继续接收新音频。
pub struct AudioBufferManager {
    /// 当前缓冲区（正在累积的音频帧）
    current_buffer: Arc<RwLock<VecDeque<AudioFrame>>>,
    
    /// 备用缓冲区（用于切换）
    next_buffer: Arc<RwLock<VecDeque<AudioFrame>>>,
    
    /// 最大缓冲时长（毫秒），防止缓冲区溢出
    max_buffer_duration_ms: u64,
    
    /// 最小片段时长（毫秒），防止过短片段
    min_segment_duration_ms: u64,
    
    /// 第一个帧的时间戳（用于计算缓冲时长）
    first_frame_timestamp: Arc<RwLock<Option<u64>>>,
}

impl AudioBufferManager {
    /// 创建新的音频缓冲管理器
    /// 
    /// 默认配置符合第二阶段目标要求：
    /// - 最大缓冲时长：5 秒（符合 3-5 秒要求）
    /// - 最小片段时长：200ms
    pub fn new() -> Self {
        Self {
            current_buffer: Arc::new(RwLock::new(VecDeque::new())),
            next_buffer: Arc::new(RwLock::new(VecDeque::new())),
            max_buffer_duration_ms: 5000,   // 5秒（符合第二阶段目标：3-5秒）
            min_segment_duration_ms: 200,    // 200ms
            first_frame_timestamp: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 创建带自定义配置的音频缓冲管理器
    pub fn with_config(
        max_buffer_duration_ms: u64,
        min_segment_duration_ms: u64,
    ) -> Self {
        Self {
            current_buffer: Arc::new(RwLock::new(VecDeque::new())),
            next_buffer: Arc::new(RwLock::new(VecDeque::new())),
            max_buffer_duration_ms,
            min_segment_duration_ms,
            first_frame_timestamp: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 添加音频帧到当前缓冲区
    pub async fn push_frame(&self, frame: AudioFrame) -> EngineResult<()> {
        let mut buffer = self.current_buffer.write().await;
        let mut first_ts = self.first_frame_timestamp.write().await;
        
        // 记录第一个帧的时间戳
        if first_ts.is_none() {
            *first_ts = Some(frame.timestamp_ms);
        }
        
        // 检查缓冲区是否溢出
        if let Some(first_ts_value) = *first_ts {
            let duration = frame.timestamp_ms.saturating_sub(first_ts_value);
            if duration > self.max_buffer_duration_ms {
                // 缓冲区溢出，返回错误（调用者应该强制截断）
                return Err(crate::error::EngineError::new(
                    format!(
                        "Buffer overflow: duration {}ms exceeds max {}ms",
                        duration, self.max_buffer_duration_ms
                    )
                ));
            }
        }
        
        buffer.push_back(frame);
        Ok(())
    }
    
    /// 获取当前缓冲区的所有帧（用于 ASR 推理）
    /// 
    /// 注意：调用此方法后，当前缓冲区会被清空
    pub async fn take_current_buffer(&self) -> Vec<AudioFrame> {
        let mut buffer = self.current_buffer.write().await;
        let mut first_ts = self.first_frame_timestamp.write().await;
        
        let frames: Vec<AudioFrame> = buffer.drain(..).collect();
        *first_ts = None;  // 重置第一个帧的时间戳
        
        frames
    }
    
    /// 切换到下一个缓冲区
    /// 
    /// 将当前缓冲区和备用缓冲区交换，用于继续接收新音频
    pub async fn swap_buffers(&self) {
        let mut current = self.current_buffer.write().await;
        let mut next = self.next_buffer.write().await;
        std::mem::swap(&mut *current, &mut *next);
        
        // 重置第一个帧的时间戳
        let mut first_ts = self.first_frame_timestamp.write().await;
        *first_ts = None;
    }
    
    /// 检查当前缓冲区是否满足最小片段时长
    pub async fn check_min_duration(&self) -> bool {
        let buffer = self.current_buffer.read().await;
        let first_ts = self.first_frame_timestamp.read().await;
        
        if buffer.is_empty() {
            return false;
        }
        
        if let Some(first_ts_value) = *first_ts {
            if let Some(last_frame) = buffer.back() {
                let duration = last_frame.timestamp_ms.saturating_sub(first_ts_value);
                return duration >= self.min_segment_duration_ms;
            }
        }
        
        false
    }
    
    /// 获取当前缓冲区的帧数
    pub async fn frame_count(&self) -> usize {
        let buffer = self.current_buffer.read().await;
        buffer.len()
    }
    
    /// 获取当前缓冲区的时长（毫秒）
    pub async fn duration_ms(&self) -> u64 {
        let buffer = self.current_buffer.read().await;
        let first_ts = self.first_frame_timestamp.read().await;
        
        if buffer.is_empty() {
            return 0;
        }
        
        if let Some(first_ts_value) = *first_ts {
            if let Some(last_frame) = buffer.back() {
                return last_frame.timestamp_ms.saturating_sub(first_ts_value);
            }
        }
        
        0
    }
    
    /// 清空当前缓冲区
    pub async fn clear(&self) {
        let mut buffer = self.current_buffer.write().await;
        let mut first_ts = self.first_frame_timestamp.write().await;
        buffer.clear();
        *first_ts = None;
    }
    
    /// 检查缓冲区是否为空
    pub async fn is_empty(&self) -> bool {
        let buffer = self.current_buffer.read().await;
        buffer.is_empty()
    }
}

impl Default for AudioBufferManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 合并多个音频帧为单个音频数据
pub fn merge_frames(frames: &[AudioFrame]) -> Vec<f32> {
    if frames.is_empty() {
        return Vec::new();
    }
    
    let total_samples = frames.iter().map(|f| f.data.len()).sum();
    let mut merged = Vec::with_capacity(total_samples);
    
    for frame in frames {
        merged.extend_from_slice(&frame.data);
    }
    
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_frame(timestamp_ms: u64, data: Vec<f32>) -> AudioFrame {
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data,
            timestamp_ms,
        }
    }
    
    #[tokio::test]
    async fn test_push_and_take() {
        let manager = AudioBufferManager::new();
        
        // 添加几个帧
        manager.push_frame(create_test_frame(0, vec![1.0, 2.0])).await.unwrap();
        manager.push_frame(create_test_frame(100, vec![3.0, 4.0])).await.unwrap();
        
        assert_eq!(manager.frame_count().await, 2);
        
        // 获取缓冲区内容
        let frames = manager.take_current_buffer().await;
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].data, vec![1.0, 2.0]);
        assert_eq!(frames[1].data, vec![3.0, 4.0]);
        
        // 缓冲区应该为空
        assert!(manager.is_empty().await);
    }
    
    #[tokio::test]
    async fn test_min_duration_check() {
        let manager = AudioBufferManager::with_config(10000, 500);
        
        // 添加一个短片段（100ms < 500ms）
        manager.push_frame(create_test_frame(0, vec![1.0])).await.unwrap();
        manager.push_frame(create_test_frame(100, vec![2.0])).await.unwrap();
        
        assert!(!manager.check_min_duration().await);
        
        // 添加更长的片段（600ms > 500ms）
        manager.push_frame(create_test_frame(600, vec![3.0])).await.unwrap();
        assert!(manager.check_min_duration().await);
    }
    
    #[tokio::test]
    async fn test_buffer_overflow() {
        let manager = AudioBufferManager::with_config(1000, 200);
        
        // 添加帧，总时长超过 1000ms
        manager.push_frame(create_test_frame(0, vec![1.0])).await.unwrap();
        manager.push_frame(create_test_frame(500, vec![2.0])).await.unwrap();
        
        // 这个帧会导致溢出（1500ms > 1000ms）
        let result = manager.push_frame(create_test_frame(1500, vec![3.0])).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_merge_frames() {
        let frames = vec![
            create_test_frame(0, vec![1.0, 2.0]),
            create_test_frame(100, vec![3.0, 4.0]),
            create_test_frame(200, vec![5.0, 6.0]),
        ];
        
        let merged = merge_frames(&frames);
        assert_eq!(merged, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }
}

