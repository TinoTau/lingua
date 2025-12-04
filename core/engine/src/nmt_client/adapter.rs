//! NMT 客户端适配器
//!
//! 将 NmtClient 适配为 NmtIncremental trait，以便与现有系统集成。

use async_trait::async_trait;
use std::sync::Arc;
use crate::error::EngineResult;
use crate::nmt_incremental::{NmtIncremental, TranslationRequest, TranslationResponse};
use super::{NmtClient, NmtTranslateRequest};

/// NMT 客户端适配器
///
/// 将 NmtClient 包装为 NmtIncremental trait 的实现
pub struct NmtClientAdapter {
    client: Arc<dyn NmtClient>,
}

impl NmtClientAdapter {
    /// 创建新的适配器
    pub fn new(client: Arc<dyn NmtClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl NmtIncremental for NmtClientAdapter {
    async fn initialize(&self) -> EngineResult<()> {
        // HTTP 客户端不需要初始化
        Ok(())
    }

    async fn translate(&self, request: TranslationRequest) -> EngineResult<TranslationResponse> {
        // 从 TranslationRequest 提取文本和语言信息
        let source_text = request.transcript.text.clone();
        let target_lang = request.target_language.clone();

        // 确定源语言
        // 注意：TranslationRequest 中的 transcript 是 PartialTranscript，不包含 language 字段
        // 所以我们需要从 target_language 推断源语言
        // 这是一个简化处理：如果目标是 en，源是 zh；如果目标是 zh，源是 en
        // 实际应用中，应该从 ASR 结果（StableTranscript）中获取源语言
        let src_lang = if target_lang == "en" {
            "zh"
        } else if target_lang == "zh" {
            "en"
        } else {
            // 默认使用 zh -> en
            "zh"
        };

        // 构造 NmtTranslateRequest
        let req = NmtTranslateRequest {
            src_lang: src_lang.to_string(),
            tgt_lang: target_lang,
            text: source_text,
        };

        // 调用客户端
        let response = self.client.translate(&req).await.map_err(|e| {
            crate::error::EngineError::new(format!("NMT client error: {}", e))
        })?;

        // 检查响应
        if !response.ok {
            return Err(crate::error::EngineError::new(
                response.error.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        // 构造 TranslationResponse
        let translated_text = response.text.ok_or_else(|| {
            crate::error::EngineError::new("No translation text in response".to_string())
        })?;

        Ok(TranslationResponse {
            translated_text,
            speaker_id: None,
            source_text: None,
            source_audio_duration_ms: None,
            quality_metrics: None,
            is_stable: true, // HTTP 服务总是返回稳定结果
        })
    }

    async fn finalize(&self) -> EngineResult<()> {
        // HTTP 客户端不需要清理
        Ok(())
    }
}

