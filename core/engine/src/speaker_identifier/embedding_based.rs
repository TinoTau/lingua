//! 基于 Speaker Embedding 的说话者识别
//! 
//! 这是一个准确的实现，适用于付费用户：
//! - 使用轻量级 Speaker Embedding 模型（如 ECAPA-TDNN）
//! - 提取音频片段的说话者特征向量
//! - 与已有说话者的 embedding 比较，判断是否为新说话者
//! 
//! 注意：当前为占位符实现，实际使用时需要集成 Speaker Embedding 模型

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::error::{EngineError, EngineResult};
use crate::types::AudioFrame;
use super::{SpeakerIdentifier, SpeakerIdentificationResult, SpeakerEmbeddingClient, SpeakerEmbeddingClientConfig, EmbeddingBasedMode};

/// 提取 embedding 的结果
struct ExtractResult {
    embedding: Option<Vec<f32>>,
    estimated_gender: Option<String>,
}

/// 基于 Speaker Embedding 的说话者识别器
pub struct EmbeddingBasedSpeakerIdentifier {
    /// HTTP 客户端（用于调用 Python 服务）
    embedding_client: SpeakerEmbeddingClient,
    /// 相似度阈值（0.0-1.0），超过此值认为是同一说话者
    similarity_threshold: f32,
    /// 识别模式：单人模式或多人模式（可动态切换）
    mode: Arc<RwLock<EmbeddingBasedMode>>,
    /// 已有说话者的 embedding 库（按模式分开存储）
    /// Key: speaker_id, Value: embedding vector
    /// 单人模式使用 "single_user" 作为 key，多人模式使用 "default_male"/"default_female" 等
    speaker_embeddings: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    /// 下一个说话者 ID 的计数器（多人模式使用）
    next_speaker_id: Arc<RwLock<u32>>,
    /// 每个说话者的参考音频片段列表（用于合并，按模式分开存储）
    /// Key: speaker_id, Value: Vec<参考音频片段>
    /// 当累积到足够长度时，会合并成一个更长的参考音频
    speaker_reference_audio_segments: Arc<RwLock<HashMap<String, Vec<Vec<f32>>>>>,
    /// 合并参考音频的最小总长度（样本数，16kHz，约 10 秒）
    min_merged_audio_samples: usize,
    /// 单人模式下的固定 speaker_id
    single_user_speaker_id: Arc<RwLock<Option<String>>>,
}

impl EmbeddingBasedSpeakerIdentifier {
    /// 创建新的基于 Speaker Embedding 的说话者识别器
    /// 
    /// # Arguments
    /// * `service_url` - HTTP 服务端点（例如：http://127.0.0.1:5003）
    /// * `similarity_threshold` - 相似度阈值（0.0-1.0）
    /// * `mode` - 识别模式：单人模式或多人模式
    pub fn new(
        service_url: Option<String>,
        similarity_threshold: f32,
        mode: EmbeddingBasedMode,
    ) -> EngineResult<Self> {
        let config = SpeakerEmbeddingClientConfig {
            endpoint: service_url.unwrap_or_else(|| "http://127.0.0.1:5003".to_string()),
            timeout_ms: 5000,
        };
        
        let embedding_client = SpeakerEmbeddingClient::new(config)?;
        
        Ok(Self {
            embedding_client,
            similarity_threshold,
            mode: Arc::new(RwLock::new(mode)),  // 使用 Arc<RwLock> 以支持动态切换
            speaker_embeddings: Arc::new(RwLock::new(HashMap::new())),
            next_speaker_id: Arc::new(RwLock::new(1)),
            speaker_reference_audio_segments: Arc::new(RwLock::new(HashMap::new())),
            min_merged_audio_samples: 160000,  // 16kHz * 10秒 = 160000 样本
            single_user_speaker_id: Arc::new(RwLock::new(None)),
        })
    }
    
    /// 生成新的说话者 ID
    async fn generate_speaker_id(&self) -> String {
        let mut counter = self.next_speaker_id.write().await;
        let id = format!("speaker_{}", *counter);
        *counter += 1;
        id
    }
    
    /// 提取音频的 speaker embedding
    /// 
    /// 通过 HTTP 服务调用 Python 服务提取特征向量
    /// 如果音频太短，返回 None 和估计的性别
    async fn extract_embedding(&self, audio_segment: &[AudioFrame]) -> EngineResult<ExtractResult> {
        use std::time::Instant;
        let start_time = Instant::now();
        
        if audio_segment.is_empty() {
            return Err(crate::error::EngineError::new("Empty audio segment"));
        }
        
        eprintln!("[SpeakerIdentifier] ===== Extract Embedding Started =====");
        eprintln!("[SpeakerIdentifier] Audio segment: {} frames", audio_segment.len());
        
        // 1. 合并音频帧
        let merge_start = Instant::now();
        let mut merged_audio = Vec::new();
        let mut total_samples = 0;
        let mut sample_rate = 16000u32;
        for frame in audio_segment {
            // 确保采样率是 16kHz（ECAPA-TDNN 要求）
            if frame.sample_rate != 16000 {
                // TODO: 重采样到 16kHz（当前假设已经是 16kHz）
                eprintln!("[SpeakerIdentifier] ⚠ Warning: Audio sample rate is {}Hz, expected 16kHz", frame.sample_rate);
            }
            sample_rate = frame.sample_rate;
            merged_audio.extend_from_slice(&frame.data);
            total_samples += frame.data.len();
        }
        let merge_ms = merge_start.elapsed().as_millis() as u64;
        let duration_sec = total_samples as f32 / sample_rate as f32;
        let duration_ms = (duration_sec * 1000.0) as u64;
        eprintln!("[SpeakerIdentifier] Merged {} frames into {} samples in {}ms", 
                  audio_segment.len(), total_samples, merge_ms);
        eprintln!("[SpeakerIdentifier] Input audio duration: {:.2}s ({:.0}ms) at {}Hz", 
                  duration_sec, duration_ms, sample_rate);
        
        // 2. 调用 HTTP 服务提取 embedding
        eprintln!("[SpeakerIdentifier] Calling Speaker Embedding service...");
        let extract_result = self.embedding_client.extract_embedding(&merged_audio).await?;
        
        let total_ms = start_time.elapsed().as_millis() as u64;
        
        if extract_result.use_default {
            let gender = extract_result.estimated_gender.as_deref().unwrap_or("unknown");
            eprintln!("[SpeakerIdentifier] ⚠ Using default voice (audio too short, estimated gender: {})", gender);
            eprintln!("[SpeakerIdentifier] ✅ Extract embedding completed in {}ms (using default voice)", total_ms);
            eprintln!("[SpeakerIdentifier] ==========================================");
            return Ok(ExtractResult {
                embedding: None,
                estimated_gender: extract_result.estimated_gender,
            });
        }
        
        let embedding = extract_result.embedding.ok_or_else(|| {
            EngineError::new("Embedding extraction returned no embedding")
        })?;
        
        eprintln!("[SpeakerIdentifier] ✅ Extract embedding completed in {}ms (merge: {}ms, service: {}ms)", 
                  total_ms, merge_ms, total_ms - merge_ms);
        eprintln!("[SpeakerIdentifier] ==========================================");
        
        Ok(ExtractResult {
            embedding: Some(embedding),
            estimated_gender: None,
        })
    }
    
    /// 计算两个 embedding 的余弦相似度
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }
    
    /// 查找最相似的说话者
    async fn find_most_similar_speaker(
        &self,
        embedding: &[f32],
    ) -> Option<(String, f32)> {
        let embeddings = self.speaker_embeddings.read().await;
        
        if embeddings.is_empty() {
            eprintln!("[SpeakerIdentifier] 📊 No existing speakers in database");
            return None;
        }
        
        eprintln!("[SpeakerIdentifier] 📊 Comparing with {} existing speaker(s)...", embeddings.len());
        
        let mut best_match: Option<(String, f32)> = None;
        let mut all_similarities: Vec<(String, f32)> = Vec::new();
        
        for (speaker_id, speaker_embedding) in embeddings.iter() {
            let similarity = Self::cosine_similarity(embedding, speaker_embedding);
            all_similarities.push((speaker_id.clone(), similarity));
            
            if let Some((_, best_sim)) = best_match {
                if similarity > best_sim {
                    best_match = Some((speaker_id.clone(), similarity));
                }
            } else {
                best_match = Some((speaker_id.clone(), similarity));
            }
        }
        
        // 打印所有相似度值（用于调试）
        eprintln!("[SpeakerIdentifier] 📊 Similarity scores:");
        for (sid, sim) in all_similarities.iter() {
            eprintln!("[SpeakerIdentifier]   - {}: {:.4}", sid, sim);
        }
        
        if let Some((best_id, best_sim)) = best_match.as_ref() {
            eprintln!("[SpeakerIdentifier] 🎯 Best match: {} (similarity: {:.4})", best_id, best_sim);
        }
        
        best_match
    }
    
    /// 单人模式：所有语音视为同一用户，合并不足7秒的音频到10秒左右，持续优化音色
    async fn identify_single_user_mode(
        &self,
        audio_segment: &[AudioFrame],
    ) -> EngineResult<SpeakerIdentificationResult> {
        eprintln!("[SpeakerIdentifier] 🔵 Single User Mode: treating all audio as same user");
        
        // 1. 获取或创建固定的 speaker_id
        let speaker_id = {
            let mut single_id = self.single_user_speaker_id.write().await;
            if single_id.is_none() {
                *single_id = Some("single_user".to_string());
                eprintln!("[SpeakerIdentifier] 🆕 Created single user speaker_id: single_user");
            }
            single_id.clone().unwrap()
        };
        
        // 2. 合并当前音频片段
        let mut current_audio = Vec::new();
        for frame in audio_segment {
            current_audio.extend_from_slice(&frame.data);
        }
        
        let current_duration_sec = current_audio.len() as f32 / 16000.0;
        eprintln!("[SpeakerIdentifier] 📊 Current audio segment: {:.2}s ({} samples @ 16kHz)", 
                 current_duration_sec, current_audio.len());
        
        // 3. 累积音频片段（合并不足7秒的音频到10秒左右）
        let mut segments = self.speaker_reference_audio_segments.write().await;
        let segments_list = segments.entry(speaker_id.clone()).or_insert_with(Vec::new);
        segments_list.push(current_audio.clone());
        
        // 计算累积的总长度
        let total_samples: usize = segments_list.iter().map(|seg| seg.len()).sum();
        let total_duration_sec = total_samples as f32 / 16000.0;
        eprintln!("[SpeakerIdentifier] 📊 Accumulated audio: {} segments, {:.2}s total", 
                 segments_list.len(), total_duration_sec);
        
        // 4. 如果累积的音频达到约7秒（112000样本），尝试提取特征
        // 如果达到10秒（160000样本），合并并提取特征
        let min_samples_for_extraction = 112000;  // 7秒 @ 16kHz
        let reference_audio = if total_samples >= self.min_merged_audio_samples {
            // 达到10秒，合并所有片段
            eprintln!("[SpeakerIdentifier] 🔗 Merging {} reference audio segments (total: {:.2}s)", 
                     segments_list.len(), total_duration_sec);
            let merged: Vec<f32> = segments_list.iter().flat_map(|seg| seg.iter().cloned()).collect();
            // 保留合并后的音频，但不清空（继续累积以持续优化）
            segments_list.clear();
            segments_list.push(merged.clone());
            eprintln!("[SpeakerIdentifier] ✅ Merged reference audio ready ({} samples, {:.2}s)", 
                     merged.len(), merged.len() as f32 / 16000.0);
            Some(merged)
        } else if total_samples >= min_samples_for_extraction {
            // 达到7秒，可以提取特征，但继续累积到10秒
            eprintln!("[SpeakerIdentifier] ⚠️  Audio reached {:.2}s (>= 7s), can extract features, but continuing to accumulate to 10s", 
                     total_duration_sec);
            // 合并当前所有片段用于特征提取
            let merged: Vec<f32> = segments_list.iter().flat_map(|seg| seg.iter().cloned()).collect();
            Some(merged)
        } else {
            // 不足7秒，继续累积
            eprintln!("[SpeakerIdentifier] ⏳ Audio only {:.2}s (< 7s), continuing to accumulate", 
                     total_duration_sec);
            Some(current_audio)
        };
        
        // 5. 提取 embedding 和性别信息（如果音频足够长）
        let (embedding, estimated_gender) = if total_samples >= min_samples_for_extraction {
            // 使用累积的音频提取 embedding
            let merged_for_embedding: Vec<f32> = segments_list.iter()
                .flat_map(|seg| seg.iter().cloned())
                .collect();
            
            // 创建临时的 AudioFrame 用于提取 embedding
            let temp_frames: Vec<AudioFrame> = vec![AudioFrame {
                data: merged_for_embedding,
                sample_rate: 16000,
                channels: 1,
                timestamp_ms: 0,
            }];
            
            let extract_result = self.extract_embedding(&temp_frames).await?;
            let gender = extract_result.estimated_gender.clone();
            
            if let Some(emb) = extract_result.embedding {
                // 更新或保存 embedding（持续优化）
                let mut embeddings = self.speaker_embeddings.write().await;
                if let Some(existing_emb) = embeddings.get_mut(&speaker_id) {
                    // 使用加权平均更新 embedding（持续优化音色）
                    for (i, new_val) in emb.iter().enumerate() {
                        if i < existing_emb.len() {
                            existing_emb[i] = existing_emb[i] * 0.7 + new_val * 0.3;
                        }
                    }
                    eprintln!("[SpeakerIdentifier] 🔄 Updated embedding for single user (weighted average: 0.7 old + 0.3 new)");
                } else {
                    // 首次保存 embedding
                    embeddings.insert(speaker_id.clone(), emb.clone());
                    eprintln!("[SpeakerIdentifier] 💾 Saved initial embedding for single user");
                }
                (Some(emb), gender)
            } else {
                // 提取失败，尝试使用当前片段提取性别信息
                let current_extract = self.extract_embedding(audio_segment).await?;
                (None, current_extract.estimated_gender)
            }
        } else {
            // 音频不足7秒，无法提取 embedding，但可以提取性别信息
            let extract_result = self.extract_embedding(audio_segment).await?;
            (None, extract_result.estimated_gender)
        };
        
        Ok(SpeakerIdentificationResult {
            speaker_id,
            is_new_speaker: false,  // 单人模式下始终是同一用户
            confidence: 1.0,  // 单人模式下置信度最高
            voice_embedding: embedding,
            reference_audio,
            estimated_gender,
        })
    }
    
    /// 多人模式：仅区分男女，使用默认的男声或女声
    async fn identify_multi_user_mode(
        &self,
        audio_segment: &[AudioFrame],
    ) -> EngineResult<SpeakerIdentificationResult> {
        eprintln!("[SpeakerIdentifier] 🟢 Multi User Mode: only distinguishing gender");
        
        // 1. 提取 embedding 和性别信息
        let extract_result = self.extract_embedding(audio_segment).await?;
        
        // 2. 根据性别分配 speaker_id（仅区分男女）
        let estimated_gender = extract_result.estimated_gender.as_deref().unwrap_or("unknown");
        let speaker_id = match estimated_gender.to_lowercase().as_str() {
            "male" | "m" => "default_male".to_string(),
            "female" | "f" => "default_female".to_string(),
            _ => "default_speaker".to_string(),  // 未知性别使用通用默认
        };
        
        eprintln!("[SpeakerIdentifier] 👤 Gender-based speaker ID: {} (estimated gender: {})", 
                 speaker_id, estimated_gender);
        
        // 3. 多人模式下不使用参考音频和 embedding（使用默认音色）
        Ok(SpeakerIdentificationResult {
            speaker_id,
            is_new_speaker: false,  // 默认说话者不算新说话者
            confidence: 0.8,  // 基于性别的识别置信度
            voice_embedding: None,  // 不使用 embedding
            reference_audio: None,  // 不使用参考音频
            estimated_gender: extract_result.estimated_gender,
        })
    }
    
    /// 动态切换模式（不会清空另一种模式的数据）
    pub async fn set_mode(&self, new_mode: EmbeddingBasedMode) {
        let mut mode = self.mode.write().await;
        let old_mode = format!("{:?}", *mode);
        *mode = new_mode;
        let new_mode_str = format!("{:?}", *mode);
        eprintln!("[SpeakerIdentifier] 🔄 Mode switched from {} to {} (data preserved)", old_mode, new_mode_str);
    }

    /// 获取当前模式
    pub async fn get_mode(&self) -> EmbeddingBasedMode {
        self.mode.read().await.clone()
    }
}

#[async_trait]
impl SpeakerIdentifier for EmbeddingBasedSpeakerIdentifier {
    async fn identify_speaker(
        &self,
        audio_segment: &[AudioFrame],
        _boundary_timestamp_ms: u64,
    ) -> EngineResult<SpeakerIdentificationResult> {
        let current_mode = self.mode.read().await.clone();
        match current_mode {
            EmbeddingBasedMode::SingleUser => {
                self.identify_single_user_mode(audio_segment).await
            }
            EmbeddingBasedMode::MultiUser => {
                self.identify_multi_user_mode(audio_segment).await
            }
        }
    }
    
    async fn reset(&self) -> EngineResult<()> {
        let mut embeddings = self.speaker_embeddings.write().await;
        let mut counter = self.next_speaker_id.write().await;
        let mut segments = self.speaker_reference_audio_segments.write().await;
        let mut single_id = self.single_user_speaker_id.write().await;
        
        embeddings.clear();
        segments.clear();
        *counter = 1;
        *single_id = None;  // 重置单人模式的 speaker_id
        
        Ok(())
    }
    
    fn get_info(&self) -> String {
        // 注意：这里不能使用 async，所以使用 try_read 或返回固定信息
        format!(
            "EmbeddingBasedSpeakerIdentifier(threshold={})",
            self.similarity_threshold
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
    #[ignore]  // 需要 HTTP 服务运行
    async fn test_first_speaker() {
        let identifier = EmbeddingBasedSpeakerIdentifier::new(
            Some("http://127.0.0.1:5003".to_string()),
            0.7,
            EmbeddingBasedMode::MultiUser,  // 使用多人模式进行测试
        ).unwrap();
        
        let result = identifier.identify_speaker(&[create_test_frame(0)], 0).await.unwrap();
        // 在多人模式下，speaker_id 可能是 default_male 或 default_female
        assert!(result.speaker_id.starts_with("default_") || result.speaker_id.starts_with("speaker_"));
    }
    
    #[tokio::test]
    #[ignore]  // 需要 HTTP 服务运行
    async fn test_single_user_mode() {
        let identifier = EmbeddingBasedSpeakerIdentifier::new(
            Some("http://127.0.0.1:5003".to_string()),
            0.7,
            EmbeddingBasedMode::SingleUser,
        ).unwrap();
        
        let result = identifier.identify_speaker(&[create_test_frame(0)], 0).await.unwrap();
        assert_eq!(result.speaker_id, "single_user");
        assert!(!result.is_new_speaker);  // 单人模式下始终不是新说话者
    }
    
    #[tokio::test]
    #[ignore]  // 需要 HTTP 服务运行
    async fn test_reset() {
        let identifier = EmbeddingBasedSpeakerIdentifier::new(
            Some("http://127.0.0.1:5003".to_string()),
            0.7,
            EmbeddingBasedMode::MultiUser,
        ).unwrap();
        
        identifier.identify_speaker(&[create_test_frame(0)], 0).await.unwrap();
        identifier.reset().await.unwrap();
        
        let result = identifier.identify_speaker(&[create_test_frame(1000)], 1000).await.unwrap();
        // 重置后，speaker_id 应该重新开始
        assert!(result.speaker_id.starts_with("default_") || result.speaker_id.starts_with("speaker_"));
    }
    
    #[tokio::test]
    async fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let similarity = EmbeddingBasedSpeakerIdentifier::cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 0.001);
        
        let c = vec![0.0, 1.0, 0.0];
        let similarity = EmbeddingBasedSpeakerIdentifier::cosine_similarity(&a, &c);
        assert!((similarity - 0.0).abs() < 0.001);
    }
}

