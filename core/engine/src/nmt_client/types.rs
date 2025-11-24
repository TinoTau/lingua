//! NMT 客户端类型定义

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// NMT 翻译请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NmtTranslateRequest {
    pub src_lang: String,
    pub tgt_lang: String,
    pub text: String,
}

/// NMT 翻译响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NmtTranslateResponse {
    pub ok: bool,
    pub text: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub extra: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// NMT 客户端 trait
#[async_trait]
pub trait NmtClient: Send + Sync {
    /// 执行翻译
    async fn translate(
        &self,
        req: &NmtTranslateRequest,
    ) -> anyhow::Result<NmtTranslateResponse>;
}

