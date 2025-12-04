use core_engine::speaker_identifier::{
    SpeakerEmbeddingClient, SpeakerEmbeddingClientConfig,
};

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_speaker_embedding_client_health_check() {
    let client = SpeakerEmbeddingClient::with_default_config().unwrap();
    
    // 健康检查
    let is_healthy = client.health_check().await;
    match is_healthy {
        Ok(true) => println!("✅ Service is healthy"),
        Ok(false) => println!("⚠️  Service returned unhealthy status"),
        Err(e) => println!("❌ Health check failed: {}", e),
    }
}

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_speaker_embedding_client_extract() {
    let client = SpeakerEmbeddingClient::with_default_config().unwrap();
    
    // 创建测试音频数据（16kHz，1秒）
    let audio_data: Vec<f32> = (0..16000)
        .map(|i| (i as f32 * 0.001).sin())
        .collect();
    
    match client.extract_embedding(&audio_data).await {
        Ok(embedding) => {
            assert_eq!(embedding.len(), 192, "Embedding should be 192-dimensional");
            println!("✅ Extracted embedding: {} dimensions", embedding.len());
            
            // 验证 embedding 不是全零
            let sum: f32 = embedding.iter().sum();
            assert_ne!(sum, 0.0, "Embedding should not be all zeros");
        }
        Err(e) => {
            panic!("Failed to extract embedding: {}", e);
        }
    }
}

#[tokio::test]
async fn test_speaker_embedding_client_config() {
    let config = SpeakerEmbeddingClientConfig {
        endpoint: "http://127.0.0.1:5003".to_string(),
        timeout_ms: 5000,
    };
    
    let client = SpeakerEmbeddingClient::new(config);
    assert!(client.is_ok(), "Client should be created successfully");
}

#[tokio::test]
async fn test_speaker_embedding_client_default_config() {
    let client = SpeakerEmbeddingClient::with_default_config();
    assert!(client.is_ok(), "Client with default config should be created");
}

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_speaker_embedding_client_empty_audio() {
    let client = SpeakerEmbeddingClient::with_default_config().unwrap();
    
    // 测试空音频
    let empty_audio: Vec<f32> = vec![];
    
    let result = client.extract_embedding(&empty_audio).await;
    assert!(result.is_err(), "Empty audio should return error");
}

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_speaker_embedding_client_short_audio() {
    let client = SpeakerEmbeddingClient::with_default_config().unwrap();
    
    // 测试很短的音频（可能不够提取 embedding）
    let short_audio: Vec<f32> = vec![0.1, 0.2, 0.3];
    
    // 这个可能会失败，取决于模型的最小输入要求
    let result = client.extract_embedding(&short_audio).await;
    // 不强制成功或失败，只记录结果
    match result {
        Ok(embedding) => {
            println!("✅ Short audio extracted: {} dimensions", embedding.len());
        }
        Err(e) => {
            println!("⚠️  Short audio failed (expected): {}", e);
        }
    }
}

