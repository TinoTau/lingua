//! Speaker Identifier 集成测试
//! 
//! 测试两种模式的说话者识别功能

use std::sync::Arc;
use core_engine::*;

fn create_test_frame(timestamp_ms: u64) -> AudioFrame {
    AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: vec![0.0; 512],
        timestamp_ms,
    }
}

#[tokio::test]
async fn test_vad_based_speaker_identification() {
    // 测试基于 VAD 边界的说话者识别
    let identifier = Arc::new(VadBasedSpeakerIdentifier::new(1000, 5000));
    
    println!("=== 测试 VAD 基于边界的说话者识别 ===");
    println!("配置: min_switch=1000ms, max_same=5000ms");
    
    // 第一个边界 - 应该创建 speaker_1
    let result1 = identifier.identify_speaker(&[], 0).await.unwrap();
    println!("边界 0ms: speaker_id={}, is_new={}, confidence={:.2}", 
        result1.speaker_id, result1.is_new_speaker, result1.confidence);
    assert_eq!(result1.speaker_id, "speaker_1");
    assert!(result1.is_new_speaker);
    
    // 500ms 后 - 短间隔，应该是插话（speaker_2）
    let result2 = identifier.identify_speaker(&[], 500).await.unwrap();
    println!("边界 500ms: speaker_id={}, is_new={}, confidence={:.2}", 
        result2.speaker_id, result2.is_new_speaker, result2.confidence);
    assert_eq!(result2.speaker_id, "speaker_2");
    assert!(result2.is_new_speaker);
    
    // 3000ms 后 - 中等间隔，应该是同一说话者（speaker_2）
    let result3 = identifier.identify_speaker(&[], 3000).await.unwrap();
    println!("边界 3000ms: speaker_id={}, is_new={}, confidence={:.2}", 
        result3.speaker_id, result3.is_new_speaker, result3.confidence);
    assert_eq!(result3.speaker_id, "speaker_2");
    assert!(!result3.is_new_speaker);
    
    // 6000ms 后 - 长间隔，应该是新说话者（speaker_3）
    let result4 = identifier.identify_speaker(&[], 6000).await.unwrap();
    println!("边界 6000ms: speaker_id={}, is_new={}, confidence={:.2}", 
        result4.speaker_id, result4.is_new_speaker, result4.confidence);
    assert_eq!(result4.speaker_id, "speaker_3");
    assert!(result4.is_new_speaker);
    
    println!("✅ VAD 基于边界的识别测试通过\n");
}

#[tokio::test]
async fn test_embedding_based_speaker_identification() {
    // 测试基于 Embedding 的说话者识别
    let identifier = Arc::new(EmbeddingBasedSpeakerIdentifier::new(
        "models/speaker_embedding.onnx".to_string(),
        0.7,
    ));
    
    println!("=== 测试 Embedding 基于的说话者识别 ===");
    println!("配置: model=speaker_embedding.onnx, threshold=0.7");
    
    // 第一个边界 - 应该创建 speaker_1
    let frames1 = vec![create_test_frame(0)];
    let result1 = identifier.identify_speaker(&frames1, 0).await.unwrap();
    println!("边界 0ms: speaker_id={}, is_new={}, confidence={:.2}", 
        result1.speaker_id, result1.is_new_speaker, result1.confidence);
    assert_eq!(result1.speaker_id, "speaker_1");
    assert!(result1.is_new_speaker);
    
    // 第二个边界 - 由于 embedding 是占位符，应该创建新说话者
    let frames2 = vec![create_test_frame(3000)];
    let result2 = identifier.identify_speaker(&frames2, 3000).await.unwrap();
    println!("边界 3000ms: speaker_id={}, is_new={}, confidence={:.2}", 
        result2.speaker_id, result2.is_new_speaker, result2.confidence);
    // 注意：由于当前是占位符实现，每次都会创建新说话者
    
    println!("✅ Embedding 基于的识别测试通过（占位符模式）\n");
}

#[tokio::test]
async fn test_speaker_identifier_reset() {
    let identifier = Arc::new(VadBasedSpeakerIdentifier::new(1000, 5000));
    
    println!("=== 测试重置功能 ===");
    
    // 识别几个说话者
    identifier.identify_speaker(&[], 0).await.unwrap();
    identifier.identify_speaker(&[], 500).await.unwrap();
    identifier.identify_speaker(&[], 3000).await.unwrap();
    
    // 重置
    identifier.reset().await.unwrap();
    println!("已重置识别器");
    
    // 重置后应该重新开始
    let result = identifier.identify_speaker(&[], 5000).await.unwrap();
    println!("重置后边界 5000ms: speaker_id={}, is_new={}", 
        result.speaker_id, result.is_new_speaker);
    assert_eq!(result.speaker_id, "speaker_1");
    assert!(result.is_new_speaker);
    
    println!("✅ 重置功能测试通过\n");
}

#[tokio::test]
async fn test_speaker_identifier_info() {
    let vad_identifier = VadBasedSpeakerIdentifier::new(1000, 5000);
    let embedding_identifier = EmbeddingBasedSpeakerIdentifier::new(
        "models/test.onnx".to_string(),
        0.7,
    );
    
    println!("=== 测试信息获取 ===");
    println!("VAD 模式: {}", vad_identifier.get_info());
    println!("Embedding 模式: {}", embedding_identifier.get_info());
    
    assert!(vad_identifier.get_info().contains("VadBasedSpeakerIdentifier"));
    assert!(embedding_identifier.get_info().contains("EmbeddingBasedSpeakerIdentifier"));
    
    println!("✅ 信息获取测试通过\n");
}

