//! 说话者识别模块
//! 
//! 支持两种模式：
//! 1. 基于 VAD 边界的简单模式（免费用户）
//! 2. 基于 Speaker Embedding 的准确模式（付费用户）

mod vad_based;
mod embedding_based;
mod speaker_embedding_client;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::EngineResult;
use crate::types::AudioFrame;

pub use vad_based::VadBasedSpeakerIdentifier;
pub use embedding_based::EmbeddingBasedSpeakerIdentifier;
pub use speaker_embedding_client::{SpeakerEmbeddingClient, SpeakerEmbeddingClientConfig};

/// 说话者识别结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerIdentificationResult {
    /// 说话者 ID（如果识别为新说话者，会生成新的 ID）
    pub speaker_id: String,
    /// 是否为新的说话者
    pub is_new_speaker: bool,
    /// 识别置信度（0.0-1.0）
    pub confidence: f32,
    /// 说话者的音色特征向量（用于 Voice Cloning，可选）
    /// 如果为 None，表示未提取音色特征（使用预定义 voice）
    /// 如果为 Some，表示已提取音色特征（可用于 zero-shot TTS）
    pub voice_embedding: Option<Vec<f32>>,
    /// 参考音频片段（用于 Voice Cloning，可选）
    /// 如果为 None，表示未保存参考音频
    /// 如果为 Some，表示已保存参考音频（可用于 zero-shot TTS）
    pub reference_audio: Option<Vec<f32>>,
}

/// 说话者识别器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeakerIdentifierMode {
    /// 基于 VAD 边界的简单模式（免费用户）
    /// 通过时间间隔和简单规则判断说话者切换
    VadBased {
        /// 说话者切换的最小时间间隔（毫秒）
        /// 如果两个边界之间的间隔小于此值，认为是新说话者（插话）
        min_switch_interval_ms: u64,
        /// 同一说话者的最大间隔（毫秒）
        /// 如果两个边界之间的间隔大于此值，认为是新说话者
        max_same_speaker_interval_ms: u64,
    },
    /// 基于 Speaker Embedding 的准确模式（付费用户）
    /// 使用轻量级模型提取说话者特征并识别
    EmbeddingBased {
        /// HTTP 服务端点（例如：http://127.0.0.1:5003）
        /// 如果为 None，使用默认端点
        service_url: Option<String>,
        /// 相似度阈值（0.0-1.0），超过此值认为是同一说话者
        similarity_threshold: f32,
    },
}

impl Default for SpeakerIdentifierMode {
    fn default() -> Self {
        SpeakerIdentifierMode::VadBased {
            min_switch_interval_ms: 1000,  // 1秒内切换认为是插话（新说话者）
            max_same_speaker_interval_ms: 5000,  // 5秒以上认为是新说话者
        }
    }
}

/// 说话者识别器 trait
#[async_trait]
pub trait SpeakerIdentifier: Send + Sync {
    /// 识别说话者
    /// 
    /// # Arguments
    /// * `audio_segment` - 音频片段（从上一个边界到当前边界的音频）
    /// * `boundary_timestamp_ms` - 边界时间戳（毫秒）
    /// 
    /// # Returns
    /// 返回说话者识别结果
    async fn identify_speaker(
        &self,
        audio_segment: &[AudioFrame],
        boundary_timestamp_ms: u64,
    ) -> EngineResult<SpeakerIdentificationResult>;
    
    /// 重置识别器状态（用于新的会话）
    async fn reset(&self) -> EngineResult<()>;
    
    /// 获取识别器信息
    fn get_info(&self) -> String;
}

