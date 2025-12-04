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
use super::{SpeakerIdentifier, SpeakerIdentificationResult, SpeakerEmbeddingClient, SpeakerEmbeddingClientConfig};

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
    /// 已有说话者的 embedding 库
    /// Key: speaker_id, Value: embedding vector
    speaker_embeddings: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    /// 下一个说话者 ID 的计数器
    next_speaker_id: Arc<RwLock<u32>>,
}

impl EmbeddingBasedSpeakerIdentifier {
    /// 创建新的基于 Speaker Embedding 的说话者识别器
    /// 
    /// # Arguments
    /// * `service_url` - HTTP 服务端点（例如：http://127.0.0.1:5003）
    /// * `similarity_threshold` - 相似度阈值（0.0-1.0）
    pub fn new(
        service_url: Option<String>,
        similarity_threshold: f32,
    ) -> EngineResult<Self> {
        let config = SpeakerEmbeddingClientConfig {
            endpoint: service_url.unwrap_or_else(|| "http://127.0.0.1:5003".to_string()),
            timeout_ms: 5000,
        };
        
        let embedding_client = SpeakerEmbeddingClient::new(config)?;
        
        Ok(Self {
            embedding_client,
            similarity_threshold,
            speaker_embeddings: Arc::new(RwLock::new(HashMap::new())),
            next_speaker_id: Arc::new(RwLock::new(1)),
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
        
        let mut best_match: Option<(String, f32)> = None;
        
        for (speaker_id, speaker_embedding) in embeddings.iter() {
            let similarity = Self::cosine_similarity(embedding, speaker_embedding);
            
            if let Some((_, best_sim)) = best_match {
                if similarity > best_sim {
                    best_match = Some((speaker_id.clone(), similarity));
                }
            } else {
                best_match = Some((speaker_id.clone(), similarity));
            }
        }
        
        best_match
    }
}

#[async_trait]
impl SpeakerIdentifier for EmbeddingBasedSpeakerIdentifier {
    async fn identify_speaker(
        &self,
        audio_segment: &[AudioFrame],
        _boundary_timestamp_ms: u64,
    ) -> EngineResult<SpeakerIdentificationResult> {
        // 1. 提取当前音频片段的 embedding（用于说话者识别和音色特征）
        let extract_result = self.extract_embedding(audio_segment).await?;
        
        // 2. 如果音频太短，使用默认声音（基于性别）
        if extract_result.embedding.is_none() {
            // 音频太短，无法提取 embedding，使用默认声音
            // 根据估计的性别分配不同的默认说话者 ID
            let estimated_gender = extract_result.estimated_gender.as_deref().unwrap_or("unknown");
            let default_speaker_id = match estimated_gender.to_lowercase().as_str() {
                "male" | "m" => "default_male".to_string(),
                "female" | "f" => "default_female".to_string(),
                _ => "default_speaker".to_string(),  // 未知性别使用通用默认
            };
            
            eprintln!("[SpeakerIdentifier] Using default speaker ID: {} (estimated gender: {})", 
                     default_speaker_id, estimated_gender);
            
            // 不保存参考音频（因为音频太短，不适合作为参考）
            return Ok(SpeakerIdentificationResult {
                speaker_id: default_speaker_id,
                is_new_speaker: false,  // 默认说话者不算新说话者
                confidence: 0.5,  // 默认置信度较低
                voice_embedding: None,  // 没有 embedding
                reference_audio: None,  // 音频太短，不适合作为参考
            });
        }
        
        let embedding = extract_result.embedding.unwrap();
        
        // 3. 查找最相似的已有说话者
        let most_similar = self.find_most_similar_speaker(&embedding).await;
        
        // 4. 判断是否为新说话者
        let (speaker_id, is_new_speaker, confidence) = if let Some((existing_id, similarity)) = most_similar {
            if similarity >= self.similarity_threshold {
                // 相似度足够高，认为是同一说话者
                (existing_id, false, similarity)
            } else {
                // 相似度不够，认为是新说话者
                let new_id = self.generate_speaker_id().await;
                let mut embeddings = self.speaker_embeddings.write().await;
                embeddings.insert(new_id.clone(), embedding.clone());
                (new_id, true, 1.0 - similarity)  // 置信度基于不相似度
            }
        } else {
            // 没有已有说话者，创建第一个
            let new_id = self.generate_speaker_id().await;
            let mut embeddings = self.speaker_embeddings.write().await;
            embeddings.insert(new_id.clone(), embedding.clone());
            (new_id, true, 0.9)  // 第一个说话者，置信度较高
        };
        
        // 5. 保存参考音频（用于 zero-shot TTS）
        let reference_audio = if !audio_segment.is_empty() {
            // 合并音频帧作为参考音频
            let mut merged = Vec::new();
            for frame in audio_segment {
                merged.extend_from_slice(&frame.data);
            }
            Some(merged)
        } else {
            None
        };
        
        Ok(SpeakerIdentificationResult {
            speaker_id,
            is_new_speaker,
            confidence,
            voice_embedding: Some(embedding),  // 返回提取的 embedding 用于 Voice Cloning
            reference_audio,
        })
    }
    
    async fn reset(&self) -> EngineResult<()> {
        let mut embeddings = self.speaker_embeddings.write().await;
        let mut counter = self.next_speaker_id.write().await;
        
        embeddings.clear();
        *counter = 1;
        
        Ok(())
    }
    
    fn get_info(&self) -> String {
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
        ).unwrap();
        
        let result = identifier.identify_speaker(&[create_test_frame(0)], 0).await.unwrap();
        assert_eq!(result.speaker_id, "speaker_1");
        assert!(result.is_new_speaker);
    }
    
    #[tokio::test]
    #[ignore]  // 需要 HTTP 服务运行
    async fn test_reset() {
        let identifier = EmbeddingBasedSpeakerIdentifier::new(
            Some("http://127.0.0.1:5003".to_string()),
            0.7,
        ).unwrap();
        
        identifier.identify_speaker(&[create_test_frame(0)], 0).await.unwrap();
        identifier.reset().await.unwrap();
        
        let result = identifier.identify_speaker(&[create_test_frame(1000)], 1000).await.unwrap();
        assert_eq!(result.speaker_id, "speaker_1");
        assert!(result.is_new_speaker);
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

