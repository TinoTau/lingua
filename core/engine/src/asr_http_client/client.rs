// core/engine/src/asr_http_client/client.rs
// HTTP client implementation for faster-whisper ASR service

use std::time::Duration;
use reqwest::Client;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use crate::error::{EngineError, EngineResult};
use super::types::{AsrHttpRequest, AsrHttpResponse};

/// HTTP client for ASR service
pub struct AsrHttpClient {
    client: Client,
    service_url: String,
    timeout: Duration,
}

impl AsrHttpClient {
    /// Create a new ASR HTTP client
    /// 
    /// # Arguments
    /// * `service_url` - Base URL of the ASR service (e.g., "http://127.0.0.1:6006")
    /// * `timeout_secs` - Request timeout in seconds
    pub fn new(service_url: String, timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            service_url,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Check if the ASR service is healthy
    pub async fn health_check(&self) -> EngineResult<bool> {
        let url = format!("{}/health", self.service_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(true)
                } else {
                    Err(EngineError::new(format!("ASR service health check failed: {}", response.status())))
                }
            }
            Err(e) => Err(EngineError::new(format!("ASR service health check error: {}", e))),
        }
    }

    /// Transcribe audio using the ASR service
    /// 
    /// # Arguments
    /// * `audio_data` - Audio data as bytes (WAV format, 16kHz mono)
    /// * `context_prompt` - Context prompt (previous sentences)
    /// * `language` - Language code (optional, None for auto-detect)
    /// 
    /// # Returns
    /// Returns the transcribed text and segments
    pub async fn transcribe(
        &self,
        audio_data: Vec<u8>,
        context_prompt: String,
        language: Option<String>,
    ) -> EngineResult<AsrHttpResponse> {
        // Encode audio to base64
        let audio_b64 = BASE64.encode(&audio_data);
        
        // Build request
        // Note: vad_filter is set to false because we already use Silero VAD for boundary detection
        // The audio segments sent to ASR should already contain speech only
        let request = AsrHttpRequest {
            audio_b64,
            prompt: context_prompt,
            language,
            task: "transcribe".to_string(),
            beam_size: 5,
            vad_filter: false,  // Disable VAD in faster-whisper (we use Silero VAD for boundaries)
            condition_on_previous_text: true,  // 启用条件生成，提高连续识别准确度
        };
        
        // Send request
        let url = format!("{}/asr", self.service_url);
        
        eprintln!("[ASR] 📤 Sending request to Faster-Whisper service: {} (audio: {} bytes, context: {} chars)", 
                 url, audio_data.len(), request.prompt.len());
        if !request.prompt.is_empty() {
            eprintln!("[ASR] 📚 Context prompt: \"{}\"", request.prompt.chars().take(100).collect::<String>());
        }
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| EngineError::new(format!("ASR HTTP request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            eprintln!("[ASR] ❌ Service returned error {}: {}", status, error_text);
            return Err(EngineError::new(format!("ASR service returned error {}: {}", status, error_text)));
        }
        
        let asr_response: AsrHttpResponse = response
            .json()
            .await
            .map_err(|e| EngineError::new(format!("Failed to parse ASR response: {}", e)))?;
        
        eprintln!("[ASR] ✅ Received response from Faster-Whisper: {} segments, {} chars, language: {:?}, duration: {:.2}s", 
                 asr_response.segments.len(), 
                 asr_response.text.len(),
                 asr_response.language,
                 asr_response.duration);
        
        Ok(asr_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_asr_client_creation() {
        let client = AsrHttpClient::new("http://127.0.0.1:6006".to_string(), 30);
        assert_eq!(client.service_url, "http://127.0.0.1:6006");
    }
}

