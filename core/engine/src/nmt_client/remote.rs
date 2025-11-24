//! 远程 NMT HTTP 客户端
//!
//! 连接到远程翻译 API 服务（未来扩展）。

use super::types::{NmtClient, NmtTranslateRequest, NmtTranslateResponse};
use async_trait::async_trait;
use reqwest::Client;

/// 远程 NMT HTTP 客户端
#[derive(Clone)]
pub struct RemoteNmtHttpClient {
    base_url: String,
    api_key: Option<String>,
    http: Client,
}

impl RemoteNmtHttpClient {
    /// 创建新的客户端
    ///
    /// # Arguments
    /// * `url` - 服务基础 URL
    /// * `api_key` - API 密钥（可选）
    pub fn new(url: impl Into<String>, api_key: Option<String>) -> Self {
        Self {
            base_url: url.into(),
            api_key,
            http: Client::new(),
        }
    }
}

#[async_trait]
impl NmtClient for RemoteNmtHttpClient {
    async fn translate(
        &self,
        req: &NmtTranslateRequest,
    ) -> anyhow::Result<NmtTranslateResponse> {
        let url = format!("{}/v1/translate", self.base_url);
        
        let mut request_builder = self.http.post(&url).json(req);
        
        // 如果有 API 密钥，添加到请求头
        if let Some(key) = &self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }
        
        let response = request_builder.send().await?;
        
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

