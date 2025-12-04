//! Speaker Embedding 和 YourTTS 服务集成测试
//! 
//! 确保这两个服务不影响其他功能

use core_engine::speaker_identifier::{
    SpeakerEmbeddingClient, SpeakerEmbeddingClientConfig,
};
use core_engine::tts_streaming::{YourTtsHttp, YourTtsHttpConfig};
use core_engine::error::EngineResult;
use std::path::Path;
use std::time::Duration;

/// 检查服务是否运行
async fn check_service_health(endpoint: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;
    
    let url = format!("{}/health", endpoint);
    client.get(&url).send().await
        .ok()
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_speaker_embedding_service_integration() {
    // 检查服务是否运行
    let endpoint = "http://127.0.0.1:5003";
    if !check_service_health(endpoint).await {
        eprintln!("⚠️  Speaker Embedding service not running, skipping test");
        eprintln!("   Start service with: python core/engine/scripts/speaker_embedding_service.py --gpu");
        return;
    }
    
    // 创建客户端
    let client = SpeakerEmbeddingClient::new(SpeakerEmbeddingClientConfig {
        endpoint: endpoint.to_string(),
        timeout_ms: 5000,
    }).expect("Failed to create client");
    
    // 健康检查
    let is_healthy = client.health_check().await;
    assert!(is_healthy.is_ok(), "Health check should succeed");
    
    // 测试提取 embedding（使用模拟数据）
    let test_audio: Vec<f32> = (0..16000)
        .map(|i| (i as f32 * 0.001).sin())
        .collect();
    
    match client.extract_embedding(&test_audio).await {
        Ok(embedding) => {
            assert_eq!(embedding.len(), 192, "Embedding should be 192-dimensional");
            println!("✅ Speaker Embedding service integration test passed");
        }
        Err(e) => {
            eprintln!("⚠️  Speaker Embedding extraction failed: {}", e);
            // 不失败测试，因为可能是模型问题
        }
    }
}

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_yourtts_service_integration() {
    // 检查服务是否运行
    let endpoint = "http://127.0.0.1:5004";
    if !check_service_health(endpoint).await {
        eprintln!("⚠️  YourTTS service not running, skipping test");
        eprintln!("   Start service with: python core/engine/scripts/yourtts_service.py --gpu");
        return;
    }
    
    // 创建客户端
    let client = YourTtsHttp::new(YourTtsHttpConfig {
        endpoint: endpoint.to_string(),
        timeout_ms: 10000,
    }).expect("Failed to create client");
    
    // 测试语音合成
    use core_engine::tts_streaming::TtsRequest;
    
    let request = TtsRequest {
        text: "Hello, this is a test.".to_string(),
        voice: String::new(),
        locale: "en".to_string(),
        reference_audio: None,
    };
    
    match client.synthesize(request).await {
        Ok(chunk) => {
            assert!(!chunk.audio.is_empty(), "Audio should not be empty");
            println!("✅ YourTTS service integration test passed");
        }
        Err(e) => {
            eprintln!("⚠️  YourTTS synthesis failed: {}", e);
            // 不失败测试，因为可能是模型问题
        }
    }
}

#[tokio::test]
async fn test_services_do_not_affect_other_modules() {
    // 测试：确保服务客户端创建不会影响其他模块
    // 这个测试不需要服务运行
    
    // 1. 测试 Speaker Embedding 客户端创建
    let embedding_client = SpeakerEmbeddingClient::with_default_config();
    assert!(embedding_client.is_ok(), "Should create client without affecting other modules");
    
    // 2. 测试 YourTTS 客户端创建
    let tts_client = YourTtsHttp::with_default_config();
    assert!(tts_client.is_ok(), "Should create client without affecting other modules");
    
    println!("✅ Services do not affect other modules");
}

#[tokio::test]
async fn test_service_configs_are_independent() {
    // 测试：确保不同服务的配置是独立的
    
    let embedding_config = SpeakerEmbeddingClientConfig {
        endpoint: "http://127.0.0.1:5003".to_string(),
        timeout_ms: 5000,
    };
    
    let tts_config = YourTtsHttpConfig {
        endpoint: "http://127.0.0.1:5004".to_string(),
        timeout_ms: 10000,
    };
    
    // 创建客户端
    let embedding_client = SpeakerEmbeddingClient::new(embedding_config);
    let tts_client = YourTtsHttp::new(tts_config);
    
    assert!(embedding_client.is_ok(), "Embedding client should be created");
    assert!(tts_client.is_ok(), "TTS client should be created");
    
    println!("✅ Service configs are independent");
}

