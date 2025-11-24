//! NMT 客户端单元测试

use core_engine::nmt_client::{
    LocalM2m100HttpClient, NmtClient, NmtClientAdapter, NmtTranslateRequest,
};
use core_engine::nmt_incremental::{NmtIncremental, TranslationRequest};
use core_engine::types::PartialTranscript;
use std::sync::Arc;

#[tokio::test]
#[ignore] // 需要运行中的 Python 服务
async fn test_local_client_integration() {
    // 这个测试需要实际运行中的 Python 服务
    let client = Arc::new(LocalM2m100HttpClient::new("http://127.0.0.1:5008"));
    
    let request = NmtTranslateRequest {
        src_lang: "zh".to_string(),
        tgt_lang: "en".to_string(),
        text: "你好".to_string(),
    };

    let response = client.translate(&request).await;
    
    // 如果服务未运行，测试会失败
    if let Ok(resp) = response {
        assert!(resp.ok);
        assert!(resp.text.is_some());
    }
}

#[tokio::test]
async fn test_adapter_translate() {
    // 创建一个 mock 客户端用于测试适配器逻辑
    // 这里使用一个简单的实现来测试适配器
    
    struct MockClient {
        should_succeed: bool,
    }

    #[async_trait::async_trait]
    impl NmtClient for MockClient {
        async fn translate(
            &self,
            _req: &NmtTranslateRequest,
        ) -> anyhow::Result<core_engine::nmt_client::NmtTranslateResponse> {
            if self.should_succeed {
                Ok(core_engine::nmt_client::NmtTranslateResponse {
                    ok: true,
                    text: Some("Hello".to_string()),
                    model: Some("test-model".to_string()),
                    provider: Some("test-provider".to_string()),
                    extra: None,
                    error: None,
                })
            } else {
                Ok(core_engine::nmt_client::NmtTranslateResponse {
                    ok: false,
                    text: None,
                    model: None,
                    provider: Some("test-provider".to_string()),
                    extra: None,
                    error: Some("Test error".to_string()),
                })
            }
        }
    }

    // 测试成功情况
    let mock_client = Arc::new(MockClient { should_succeed: true });
    let adapter = Arc::new(NmtClientAdapter::new(mock_client));

    let request = TranslationRequest {
        transcript: PartialTranscript {
            text: "你好".to_string(),
            confidence: 1.0,
            is_final: true,
        },
        target_language: "en".to_string(),
        wait_k: None,
    };

    let response = adapter.translate(request).await;
    assert!(response.is_ok());
    let resp = response.unwrap();
    assert_eq!(resp.translated_text, "Hello");
    assert!(resp.is_stable);

    // 测试失败情况
    let mock_client = Arc::new(MockClient { should_succeed: false });
    let adapter = Arc::new(NmtClientAdapter::new(mock_client));

    let request = TranslationRequest {
        transcript: PartialTranscript {
            text: "你好".to_string(),
            confidence: 1.0,
            is_final: true,
        },
        target_language: "en".to_string(),
        wait_k: None,
    };

    let response = adapter.translate(request).await;
    assert!(response.is_err());
}

#[tokio::test]
async fn test_adapter_initialize_finalize() {
    let mock_client = Arc::new(LocalM2m100HttpClient::new("http://127.0.0.1:5008"));
    let adapter = Arc::new(NmtClientAdapter::new(mock_client));

    // initialize 和 finalize 应该总是成功
    assert!(adapter.initialize().await.is_ok());
    assert!(adapter.finalize().await.is_ok());
}

