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
        
        let response = self
            .http
            .post(&url)
            .json(req)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }
        
        let body: NmtTranslateResponse = response.json().await?;
        Ok(body)
    }
}

// 单元测试在 tests/nmt_client_test.rs 中
