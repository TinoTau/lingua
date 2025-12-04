use core_engine::tts_streaming::{YourTtsHttp, YourTtsHttpConfig, TtsRequest};
use core_engine::error::EngineResult;

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_yourtts_http_synthesize() {
    let client = YourTtsHttp::with_default_config().unwrap();
    
    let request = TtsRequest {
        text: "Hello, this is a test.".to_string(),
        voice: String::new(),
        locale: "en".to_string(),
        reference_audio: None,
    };
    
    match client.synthesize(request).await {
        Ok(chunk) => {
            assert!(!chunk.audio.is_empty(), "Audio should not be empty");
            println!("✅ Synthesized audio: {} bytes", chunk.audio.len());
        }
        Err(e) => {
            panic!("Failed to synthesize: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_yourtts_http_zero_shot() {
    let client = YourTtsHttp::with_default_config().unwrap();
    
    // 创建参考音频（22050 Hz，1秒）
    let reference_audio: Vec<f32> = (0..22050)
        .map(|i| (i as f32 * 0.001).sin())
        .collect();
    
    let request = TtsRequest {
        text: "Hello, this is a zero-shot test.".to_string(),
        voice: String::new(),
        locale: "en".to_string(),
        reference_audio: Some(reference_audio),
    };
    
    match client.synthesize(request).await {
        Ok(chunk) => {
            assert!(!chunk.audio.is_empty(), "Audio should not be empty");
            println!("✅ Zero-shot synthesized audio: {} bytes", chunk.audio.len());
        }
        Err(e) => {
            panic!("Failed to synthesize with zero-shot: {}", e);
        }
    }
}

#[tokio::test]
async fn test_yourtts_http_config() {
    let config = YourTtsHttpConfig {
        endpoint: "http://127.0.0.1:5004".to_string(),
        timeout_ms: 10000,
    };
    
    let client = YourTtsHttp::new(config);
    assert!(client.is_ok(), "Client should be created successfully");
}

#[tokio::test]
async fn test_yourtts_http_default_config() {
    let client = YourTtsHttp::with_default_config();
    assert!(client.is_ok(), "Client with default config should be created");
}

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_yourtts_http_empty_text() {
    let client = YourTtsHttp::with_default_config().unwrap();
    
    let request = TtsRequest {
        text: String::new(),
        voice: String::new(),
        locale: "en".to_string(),
        reference_audio: None,
    };
    
    let result = client.synthesize(request).await;
    assert!(result.is_err(), "Empty text should return error");
}

#[tokio::test]
#[ignore] // 需要服务运行
async fn test_yourtts_http_chinese_text() {
    let client = YourTtsHttp::with_default_config().unwrap();
    
    let request = TtsRequest {
        text: "你好，这是一个测试。".to_string(),
        voice: String::new(),
        locale: "zh".to_string(),
        reference_audio: None,
    };
    
    match client.synthesize(request).await {
        Ok(chunk) => {
            assert!(!chunk.audio.is_empty(), "Audio should not be empty");
            println!("✅ Chinese text synthesized: {} bytes", chunk.audio.len());
        }
        Err(e) => {
            panic!("Failed to synthesize Chinese text: {}", e);
        }
    }
}

