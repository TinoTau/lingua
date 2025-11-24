//! TTS 音频增强模块单元测试

use core_engine::tts_audio_enhancement::{AudioEnhancer, AudioEnhancementConfig};

#[test]
fn test_audio_enhancement_config_default() {
    let config = AudioEnhancementConfig::default();
    assert!(config.enable_fade);
    assert_eq!(config.fade_duration_ms, 20);
    assert!(config.enable_pause);
    assert_eq!(config.pause_duration_ms, 100);
    assert_eq!(config.sample_rate, 22050);
    assert_eq!(config.channels, 1);
}

#[tokio::test]
async fn test_enhance_audio_empty() {
    let enhancer = AudioEnhancer::default();
    
    // 空音频应该返回错误或空结果
    let result = enhancer.enhance_audio(&[], true, true, true).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_enhance_audio_invalid_wav() {
    let enhancer = AudioEnhancer::default();
    
    // 无效的 WAV 数据
    let invalid_data = vec![0u8; 10];
    let result = enhancer.enhance_audio(&invalid_data, true, true, true).await;
    assert!(result.is_err());
}

// 注意：完整的 WAV 文件测试需要实际的 WAV 数据
// 这里只测试配置和基本逻辑

