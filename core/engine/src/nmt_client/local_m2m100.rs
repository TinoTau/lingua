//! 本地 M2M100 HTTP 客户端
//!
//! 连接到本地运行的 Python M2M100 服务。

use super::types::{NmtClient, NmtTranslateRequest, NmtTranslateResponse};
use async_trait::async_trait;
use reqwest::Client;

/// 本地 M2M100 HTTP 客户端
#[derive(Clone)]
pub struct LocalM2m100HttpClient {
    base_url: String,
    http: Client,
}

impl LocalM2m100HttpClient {
    /// 创建新的客户端
    ///
    /// # Arguments
    /// * `url` - 服务基础 URL，例如 "http://127.0.0.1:5008"
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            base_url: url.into(),
            http: Client::new(),
        }
    }
}

#[async_trait]
impl NmtClient for LocalM2m100HttpClient {
    async fn translate(
        &self,
        req: &NmtTranslateRequest,
    ) -> anyhow::Result<NmtTranslateResponse> {
        let url = format!("{}/v1/translate", self.base_url);
        
        eprintln!("[NMT Client] Sending request to {}: text='{}', src_lang='{}', tgt_lang='{}'", 
            url, req.text, req.src_lang, req.tgt_lang);
        
        let response = self
            .http
            .post(&url)
            .json(req)
            .send()
            .await?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            eprintln!("[NMT Client] HTTP error: {} - {}", status, error_text);
            return Err(anyhow::anyhow!(
                "HTTP error: {} - {}",
                status,
                error_text
            ));
        }
        
        let body: NmtTranslateResponse = response.json().await?;
        if let Some(ref text) = body.text {
            eprintln!("[NMT Client] Received response: '{}'", text);
        } else if let Some(ref error) = body.error {
            eprintln!("[NMT Client] Received error response: '{}'", error);
        } else {
            eprintln!("[NMT Client] Received response: ok={}", body.ok);
        }
        Ok(body)
    }
}

// 单元测试在 tests/nmt_client_test.rs 中
