//! Speaker Embedding HTTP 客户端
//! 
//! 用于调用 Python HTTP 服务提取说话者特征向量

use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::error::{EngineError, EngineResult};

/// Speaker Embedding HTTP 服务配置
#[derive(Debug, Clone)]
pub struct SpeakerEmbeddingClientConfig {
    /// HTTP 服务端点（例如：http://127.0.0.1:5003）
    pub endpoint: String,
    /// 请求超时时间（毫秒）
    pub timeout_ms: u64,
}

impl Default for SpeakerEmbeddingClientConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:5003".to_string(),
            timeout_ms: 5000,
        }
    }
}

/// Speaker Embedding HTTP 客户端
pub struct SpeakerEmbeddingClient {
    client: reqwest::Client,
    config: SpeakerEmbeddingClientConfig,
}

impl SpeakerEmbeddingClient {
    /// 创建新的 Speaker Embedding HTTP 客户端
    pub fn new(config: SpeakerEmbeddingClientConfig) -> EngineResult<Self> {
        let timeout = Duration::from_millis(config.timeout_ms);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| EngineError::new(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// 使用默认配置创建客户端
    pub fn with_default_config() -> EngineResult<Self> {
        Self::new(SpeakerEmbeddingClientConfig::default())
    }

    /// 提取说话者特征向量
    /// 
    /// # Arguments
    /// * `audio` - 音频数据（16kHz 单声道，f32）
    /// 
    /// # Returns
    /// 返回提取结果，包含 embedding（如果可用）和默认声音标记
    pub async fn extract_embedding(&self, audio: &[f32]) -> EngineResult<ExtractEmbeddingResult> {
        use std::time::Instant;
        let start_time = Instant::now();
        
        eprintln!("[SpeakerEmbedding] ===== Extract Embedding Request Started =====");
        eprintln!("[SpeakerEmbedding] Endpoint: {}/extract", self.config.endpoint);
        let duration_sec = audio.len() as f32 / 16000.0;
        let duration_ms = (duration_sec * 1000.0) as u64;
        eprintln!("[SpeakerEmbedding] Input audio: {} samples, {:.2}s ({:.0}ms) at 16kHz", 
                  audio.len(), duration_sec, duration_ms);
        
        let request_body = serde_json::json!({
            "audio": audio
        });

        let request_start = Instant::now();
        eprintln!("[SpeakerEmbedding] Sending HTTP request...");
        
        let response = self.client
            .post(&format!("{}/extract", self.config.endpoint))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                let elapsed = start_time.elapsed().as_millis() as u64;
                eprintln!("[SpeakerEmbedding] ❌ HTTP request failed after {}ms: {}", elapsed, e);
                EngineError::new(format!("HTTP request failed: {}", e))
            })?;

        let request_ms = request_start.elapsed().as_millis() as u64;
        eprintln!("[SpeakerEmbedding] HTTP request completed in {}ms (status: {})", 
                  request_ms, response.status());

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            let elapsed = start_time.elapsed().as_millis() as u64;
            eprintln!("[SpeakerEmbedding] ❌ Request failed with status {} after {}ms: {}", 
                      status, elapsed, error_text);
            return Err(EngineError::new(format!(
                "HTTP request failed with status {}: {}",
                status, error_text
            )));
        }

        let parse_start = Instant::now();
        eprintln!("[SpeakerEmbedding] Parsing response...");
        
        let result: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| {
                let elapsed = start_time.elapsed().as_millis() as u64;
                eprintln!("[SpeakerEmbedding] ❌ Failed to parse response after {}ms: {}", elapsed, e);
                EngineError::new(format!("Failed to parse response: {}", e))
            })?;

        let parse_ms = parse_start.elapsed().as_millis() as u64;
        let total_ms = start_time.elapsed().as_millis() as u64;
        
        eprintln!("[SpeakerEmbedding] Response parsed in {}ms", parse_ms);
        
        // 检查是否需要使用默认声音
        let use_default = result.use_default.unwrap_or(false);
        if use_default {
            let gender = result.estimated_gender.as_deref().unwrap_or("unknown");
            let msg = result.message.as_deref().unwrap_or("Audio too short");
            eprintln!("[SpeakerEmbedding] ⚠ Audio too short, using default voice (estimated gender: {})", gender);
            eprintln!("[SpeakerEmbedding] Message: {}", msg);
            eprintln!("[SpeakerEmbedding] ✅ Extract embedding completed in {}ms (using default voice)", total_ms);
            eprintln!("[SpeakerEmbedding] ================================================");
            
            return Ok(ExtractEmbeddingResult {
                embedding: None,
                use_default: true,
                estimated_gender: result.estimated_gender,
            });
        }
        
        // 正常情况：返回 embedding
        let embedding = result.embedding.ok_or_else(|| {
            EngineError::new("Response missing embedding field")
        })?;
        
        let dimension = result.dimension.unwrap_or(embedding.len());
        eprintln!("[SpeakerEmbedding] Embedding dimension: {} (expected: 192)", dimension);
        eprintln!("[SpeakerEmbedding] ✅ Extract embedding completed in {}ms (request: {}ms, parse: {}ms)", 
                  total_ms, request_ms, parse_ms);
        eprintln!("[SpeakerEmbedding] ================================================");

        // 即使音频足够长，也保存估计的性别信息（用于选择默认音色）
        Ok(ExtractEmbeddingResult {
            embedding: Some(embedding),
            use_default: false,
            estimated_gender: result.estimated_gender,  // 保存性别信息，即使音频足够长
        })
    }

    /// 健康检查
    pub async fn health_check(&self) -> EngineResult<bool> {
        let response = self.client
            .get(&format!("{}/health", self.config.endpoint))
            .send()
            .await
            .map_err(|e| EngineError::new(format!("Health check failed: {}", e)))?;

        Ok(response.status().is_success())
    }
}

/// Speaker Embedding 服务响应
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    #[serde(default)]
    embedding: Option<Vec<f32>>,
    #[serde(default)]
    dimension: Option<usize>,
    #[serde(default)]
    input_samples: Option<usize>,
    #[serde(default)]
    sample_rate: Option<u32>,
    #[serde(default)]
    too_short: Option<bool>,
    #[serde(default)]
    use_default: Option<bool>,
    #[serde(default)]
    estimated_gender: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

/// 提取 Embedding 的结果
#[derive(Debug, Clone)]
pub struct ExtractEmbeddingResult {
    pub embedding: Option<Vec<f32>>,
    pub use_default: bool,
    pub estimated_gender: Option<String>,
}

