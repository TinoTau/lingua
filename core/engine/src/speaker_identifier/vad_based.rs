//! 基于 VAD 边界的说话者识别
//! 
//! 这是一个简单的实现，适用于免费用户：
//! - 通过时间间隔判断说话者切换
//! - 如果边界间隔很短（< min_switch_interval_ms），认为是新说话者（插话）
//! - 如果边界间隔很长（> max_same_speaker_interval_ms），认为是新说话者
//! - 否则认为是同一说话者继续

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::EngineResult;
use crate::types::AudioFrame;
use super::{SpeakerIdentifier, SpeakerIdentificationResult};

/// 基于 VAD 边界的说话者识别器
pub struct VadBasedSpeakerIdentifier {
    /// 说话者切换的最小时间间隔（毫秒）
    min_switch_interval_ms: u64,
    /// 同一说话者的最大间隔（毫秒）
    max_same_speaker_interval_ms: u64,
    /// 上一个边界的时间戳
    last_boundary_timestamp: Arc<RwLock<Option<u64>>>,
    /// 当前说话者 ID
    current_speaker_id: Arc<RwLock<Option<String>>>,
    /// 下一个说话者 ID 的计数器
    next_speaker_id: Arc<RwLock<u32>>,
}

impl VadBasedSpeakerIdentifier {
    /// 创建新的基于 VAD 边界的说话者识别器
    /// 
    /// # Arguments
    /// * `min_switch_interval_ms` - 说话者切换的最小时间间隔（毫秒）
    /// * `max_same_speaker_interval_ms` - 同一说话者的最大间隔（毫秒）
    pub fn new(
        min_switch_interval_ms: u64,
        max_same_speaker_interval_ms: u64,
    ) -> Self {
        Self {
            min_switch_interval_ms,
            max_same_speaker_interval_ms,
            last_boundary_timestamp: Arc::new(RwLock::new(None)),
            current_speaker_id: Arc::new(RwLock::new(None)),
            next_speaker_id: Arc::new(RwLock::new(1)),
        }
    }
    
    /// 生成新的说话者 ID
    async fn generate_speaker_id(&self) -> String {
        let mut counter = self.next_speaker_id.write().await;
        let id = format!("speaker_{}", *counter);
        *counter += 1;
        id
    }
}

#[async_trait]
impl SpeakerIdentifier for VadBasedSpeakerIdentifier {
    async fn identify_speaker(
        &self,
        _audio_segment: &[AudioFrame],
        boundary_timestamp_ms: u64,
    ) -> EngineResult<SpeakerIdentificationResult> {
        let mut last_ts = self.last_boundary_timestamp.write().await;
        let mut current_id = self.current_speaker_id.write().await;
        
        // 如果是第一个边界，创建第一个说话者
        if last_ts.is_none() {
            let speaker_id = self.generate_speaker_id().await;
            *last_ts = Some(boundary_timestamp_ms);
            *current_id = Some(speaker_id.clone());
            
            return Ok(SpeakerIdentificationResult {
                speaker_id,
                is_new_speaker: true,
                confidence: 0.8,  // 中等置信度（基于规则的判断）
                voice_embedding: None,  // VAD 模式不提取音色特征
                reference_audio: None,  // VAD 模式不保存参考音频
            });
        }
        
        // 计算时间间隔
        let last_ts_value = last_ts.unwrap();
        let interval = boundary_timestamp_ms.saturating_sub(last_ts_value);
        
        // 判断是否为新说话者
        let is_new_speaker = if interval < self.min_switch_interval_ms {
            // 间隔很短，认为是插话（新说话者）
            true
        } else if interval > self.max_same_speaker_interval_ms {
            // 间隔很长，认为是新说话者
            true
        } else {
            // 间隔适中，认为是同一说话者继续
            false
        };
        
        // 更新说话者 ID
        let speaker_id = if is_new_speaker {
            let new_id = self.generate_speaker_id().await;
            *current_id = Some(new_id.clone());
            new_id
        } else {
            // 同一说话者继续，使用当前 ID
            // 如果当前 ID 为空（不应该发生），生成新的
            if current_id.is_none() {
                let new_id = self.generate_speaker_id().await;
                *current_id = Some(new_id.clone());
                new_id
            } else {
                current_id.clone().unwrap()
            }
        };
        
        // 更新边界时间戳
        *last_ts = Some(boundary_timestamp_ms);
        
        // 计算置信度（基于时间间隔）
        let confidence = if interval < self.min_switch_interval_ms {
            0.9  // 插话场景，置信度较高
        } else if interval > self.max_same_speaker_interval_ms {
            0.85  // 长时间间隔，置信度较高
        } else {
            0.7  // 中等间隔，置信度较低（可能是同一人，也可能是新说话者）
        };
        
        Ok(SpeakerIdentificationResult {
            speaker_id,
            is_new_speaker,
            confidence,
            voice_embedding: None,  // VAD 模式不提取音色特征
            reference_audio: None,  // VAD 模式不保存参考音频
        })
    }
    
    async fn reset(&self) -> EngineResult<()> {
        let mut last_ts = self.last_boundary_timestamp.write().await;
        let mut current_id = self.current_speaker_id.write().await;
        let mut counter = self.next_speaker_id.write().await;
        
        *last_ts = None;
        *current_id = None;
        *counter = 1;
        
        Ok(())
    }
    
    fn get_info(&self) -> String {
        format!(
            "VadBasedSpeakerIdentifier(min_switch={}ms, max_same={}ms)",
            self.min_switch_interval_ms,
            self.max_same_speaker_interval_ms
        )
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
    async fn test_first_boundary() {
        let identifier = VadBasedSpeakerIdentifier::new(1000, 5000);
        
        let result = identifier.identify_speaker(&[], 0).await.unwrap();
        assert_eq!(result.speaker_id, "speaker_1");
        assert!(result.is_new_speaker);
    }
    
    #[tokio::test]
    async fn test_short_interval_interruption() {
        let identifier = VadBasedSpeakerIdentifier::new(1000, 5000);
        
        // 第一个边界
        let result1 = identifier.identify_speaker(&[], 0).await.unwrap();
        assert_eq!(result1.speaker_id, "speaker_1");
        assert!(result1.is_new_speaker);
        
        // 500ms 后（短间隔，应该是插话）
        let result2 = identifier.identify_speaker(&[], 500).await.unwrap();
        assert_eq!(result2.speaker_id, "speaker_2");
        assert!(result2.is_new_speaker);
        assert!(result2.confidence > 0.8);
    }
    
    #[tokio::test]
    async fn test_long_interval_new_speaker() {
        let identifier = VadBasedSpeakerIdentifier::new(1000, 5000);
        
        // 第一个边界
        let result1 = identifier.identify_speaker(&[], 0).await.unwrap();
        assert_eq!(result1.speaker_id, "speaker_1");
        
        // 6000ms 后（长间隔，应该是新说话者）
        let result2 = identifier.identify_speaker(&[], 6000).await.unwrap();
        assert_eq!(result2.speaker_id, "speaker_2");
        assert!(result2.is_new_speaker);
    }
    
    #[tokio::test]
    async fn test_medium_interval_same_speaker() {
        let identifier = VadBasedSpeakerIdentifier::new(1000, 5000);
        
        // 第一个边界
        let result1 = identifier.identify_speaker(&[], 0).await.unwrap();
        let speaker1_id = result1.speaker_id.clone();
        
        // 3000ms 后（中等间隔，应该是同一说话者）
        let result2 = identifier.identify_speaker(&[], 3000).await.unwrap();
        assert_eq!(result2.speaker_id, speaker1_id);
        assert!(!result2.is_new_speaker);
    }
    
    #[tokio::test]
    async fn test_reset() {
        let identifier = VadBasedSpeakerIdentifier::new(1000, 5000);
        
        identifier.identify_speaker(&[], 0).await.unwrap();
        identifier.identify_speaker(&[], 3000).await.unwrap();
        
        identifier.reset().await.unwrap();
        
        // 重置后应该是新的说话者
        let result = identifier.identify_speaker(&[], 5000).await.unwrap();
        assert_eq!(result.speaker_id, "speaker_1");
        assert!(result.is_new_speaker);
    }
}

