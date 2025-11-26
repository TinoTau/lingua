//! Piper HTTP TTS 客户端实现
//! 
//! 通过 HTTP 请求调用 WSL2 中运行的 Piper TTS 服务

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use crate::error::{EngineError, EngineResult};
use crate::tts_streaming::{TtsRequest, TtsStreamChunk, TtsStreaming};

/// Piper HTTP 服务配置
#[derive(Debug, Clone)]
pub struct PiperHttpConfig {
    /// HTTP 服务端点（例如：http://127.0.0.1:5005/tts）
    pub endpoint: String,
    /// 默认语音名称（例如：zh_CN-huayan-medium）
    pub default_voice: String,
    /// 请求超时时间（毫秒）
    pub timeout_ms: u64,
}

impl Default for PiperHttpConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:5005/tts".to_string(),
            default_voice: "zh_CN-huayan-medium".to_string(),
            timeout_ms: 8000,
        }
    }
}

/// Piper HTTP TTS 客户端
pub struct PiperHttpTts {
    client: reqwest::Client,
    config: PiperHttpConfig,
}

impl PiperHttpTts {
    /// 创建新的 Piper HTTP TTS 客户端
    pub fn new(config: PiperHttpConfig) -> EngineResult<Self> {
        let timeout = Duration::from_millis(config.timeout_ms);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| EngineError::new(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// 使用默认配置创建客户端
    pub fn with_default_config() -> EngineResult<Self> {
        Self::new(PiperHttpConfig::default())
    }
}

/// Piper HTTP 服务请求体
#[derive(Debug, Serialize)]
struct PiperHttpRequest {
    text: String,
    voice: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

/// Piper HTTP 服务响应（WAV 音频数据）
/// 注意：实际响应是二进制 WAV 数据，不是 JSON

#[async_trait]
impl TtsStreaming for PiperHttpTts {
    async fn synthesize(&self, request: TtsRequest) -> EngineResult<TtsStreamChunk> {
        // 确定使用的语音
        let voice = if request.voice.is_empty() {
            // 根据 locale 选择默认 voice
            let locale_lower = request.locale.to_lowercase();
            eprintln!("[Piper TTS] Request locale: '{}' (lowercase: '{}')", request.locale, locale_lower);
            if locale_lower.starts_with("en") {
                // 英文语音
                let en_voice = "en_US-lessac-medium";  // 或者根据实际可用的英文 voice 调整
                eprintln!("[Piper TTS] Using English voice: {}", en_voice);
                en_voice
            } else if locale_lower.starts_with("zh") {
                // 中文语音
                eprintln!("[Piper TTS] Using Chinese voice: {}", self.config.default_voice);
                &self.config.default_voice
            } else {
                // 默认使用配置的 voice
                eprintln!("[Piper TTS] Using default voice: {}", self.config.default_voice);
                &self.config.default_voice
            }
        } else {
            eprintln!("[Piper TTS] Using specified voice: {}", request.voice);
            &request.voice
        };

        // 构造请求体
        let http_request = PiperHttpRequest {
            text: request.text.clone(),
            voice: voice.to_string(),
            language: if request.locale.is_empty() {
                None
            } else {
                Some(request.locale.clone())
            },
        };

        // 发送 HTTP POST 请求
        let response = self
            .client
            .post(&self.config.endpoint)
            .json(&http_request)
            .send()
            .await
            .map_err(|e| {
                EngineError::new(format!(
                    "Failed to send HTTP request to Piper service: {}",
                    e
                ))
            })?;

        // 检查 HTTP 状态码
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(EngineError::new(format!(
                "Piper HTTP service returned error: {} {}",
                status, error_text
            )));
        }

        // 读取音频数据（WAV 格式）
        let audio_data = response
            .bytes()
            .await
            .map_err(|e| {
                EngineError::new(format!(
                    "Failed to read audio data from Piper service: {}",
                    e
                ))
            })?
            .to_vec();

        if audio_data.is_empty() {
            return Err(EngineError::new(
                "Piper service returned empty audio data".to_string(),
            ));
        }

        // 返回音频块
        Ok(TtsStreamChunk {
            audio: audio_data,
            timestamp_ms: 0, // Piper HTTP 服务不提供时间戳，设为 0
            is_last: true,   // HTTP 请求返回完整音频，标记为最后一块
        })
    }

    async fn close(&self) -> EngineResult<()> {
        // HTTP 客户端无需特殊清理
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piper_http_config_default() {
        let config = PiperHttpConfig::default();
        assert_eq!(config.endpoint, "http://127.0.0.1:5005/tts");
        assert_eq!(config.default_voice, "zh_CN-huayan-medium");
        assert_eq!(config.timeout_ms, 8000);
    }

    #[test]
    fn test_piper_http_config_custom() {
        let config = PiperHttpConfig {
            endpoint: "http://example.com:8080/tts".to_string(),
            default_voice: "zh_CN-xiaoyan-medium".to_string(),
            timeout_ms: 10000,
        };
        assert_eq!(config.endpoint, "http://example.com:8080/tts");
        assert_eq!(config.default_voice, "zh_CN-xiaoyan-medium");
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn test_piper_http_new() {
        let config = PiperHttpConfig::default();
        let result = PiperHttpTts::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_piper_http_with_default_config() {
        let result = PiperHttpTts::with_default_config();
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // 需要 WSL2 中运行 Piper 服务
    async fn test_piper_http_synthesize() {
        let config = PiperHttpConfig::default();
        let tts = PiperHttpTts::new(config).unwrap();

        let request = TtsRequest {
            text: "你好，欢迎使用语音翻译系统。".to_string(),
            voice: "zh_CN-huayan-medium".to_string(),
            locale: "zh-CN".to_string(),
        };

        let result = tts.synthesize(request).await;
        assert!(result.is_ok(), "TTS synthesis should succeed");

        let chunk = result.unwrap();
        assert!(!chunk.audio.is_empty(), "Audio data should not be empty");
        assert!(chunk.is_last, "Should be marked as last chunk");
        assert!(chunk.audio.len() > 1024, "Audio should be larger than 1024 bytes");

        // 验证是 WAV 格式（前 4 字节应该是 "RIFF"）
        assert!(chunk.audio.len() >= 4, "Audio should have at least 4 bytes");
        let header = String::from_utf8_lossy(&chunk.audio[0..4]);
        assert_eq!(header, "RIFF", "Audio should be WAV format (RIFF header)");
    }

    #[tokio::test]
    #[ignore] // 需要 WSL2 中运行 Piper 服务
    async fn test_piper_http_synthesize_with_default_voice() {
        let config = PiperHttpConfig::default();
        let tts = PiperHttpTts::new(config).unwrap();

        // 使用空 voice，应该使用默认语音
        let request = TtsRequest {
            text: "测试默认语音。".to_string(),
            voice: "".to_string(), // 空字符串，应该使用默认语音
            locale: "zh-CN".to_string(),
        };

        let result = tts.synthesize(request).await;
        assert!(result.is_ok(), "TTS synthesis with default voice should succeed");

        let chunk = result.unwrap();
        assert!(!chunk.audio.is_empty(), "Audio data should not be empty");
        assert!(chunk.audio.len() > 1024, "Audio should be larger than 1024 bytes");
    }

    #[tokio::test]
    #[ignore] // 需要 WSL2 中运行 Piper 服务
    async fn test_piper_http_synthesize_empty_text() {
        let config = PiperHttpConfig::default();
        let tts = PiperHttpTts::new(config).unwrap();

        let request = TtsRequest {
            text: "".to_string(),
            voice: "zh_CN-huayan-medium".to_string(),
            locale: "zh-CN".to_string(),
        };

        // 空文本可能会导致错误或返回空音频
        let result = tts.synthesize(request).await;
        // 根据实际行为，可能是 Ok 或 Err
        // 这里只验证不会 panic
        match result {
            Ok(chunk) => {
                // 如果返回成功，音频可能为空或很小
                println!("Empty text returned audio of size: {}", chunk.audio.len());
            }
            Err(e) => {
                println!("Empty text returned error: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // 需要 WSL2 中运行 Piper 服务
    async fn test_piper_http_close() {
        let config = PiperHttpConfig::default();
        let tts = PiperHttpTts::new(config).unwrap();

        // close 方法应该总是成功
        let result = tts.close().await;
        assert!(result.is_ok(), "Close should always succeed");
    }
}

