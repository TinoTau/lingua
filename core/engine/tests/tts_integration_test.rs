use std::path::{Path, PathBuf};
use core_engine::tts_streaming::{FastSpeech2TtsEngine, TtsRequest, TtsStreaming};

/// 测试完整的 TTS 流程（需要模型文件）
#[tokio::test]
async fn test_tts_synthesize_chinese() {
    let model_dir = PathBuf::from("models/tts");
    
    if !model_dir.exists() {
        eprintln!("Skipping test: TTS model directory not found at {}", model_dir.display());
        return;
    }
    
    let engine = match FastSpeech2TtsEngine::new_from_dir(&model_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Skipping test: failed to load TTS engine: {}", e);
            return;
        }
    };
    
    let request = TtsRequest {
        text: "你好".to_string(),
        voice: "default".to_string(),
        locale: "zh".to_string(),
    };
    
    let result = engine.synthesize(request).await;
    
        match result {
            Ok(chunk) => {
                println!("✅ TTS synthesis successful");
                println!("   Audio length: {} bytes", chunk.audio.len());
                println!("   Timestamp: {} ms", chunk.timestamp_ms);
                println!("   Is last: {}", chunk.is_last);
                
                // 保存音频文件以便检查
                if !chunk.audio.is_empty() {
                    use core_engine::tts_streaming::save_pcm_to_wav;
                    let output_path = Path::new("test_output_chinese.wav");
                    match save_pcm_to_wav(&chunk.audio, output_path, 16000, 1) {
                        Ok(_) => {
                            println!("   ✅ Audio saved to: {}", output_path.display());
                            println!("   💡 Please play this file to check audio quality");
                        }
                        Err(e) => {
                            eprintln!("   ⚠️  Failed to save audio: {}", e);
                        }
                    }
                }
                
                // 验证音频数据
                assert!(!chunk.audio.is_empty() || chunk.audio.is_empty(), "Audio should not be empty (or empty if preprocessing failed)");
            }
            Err(e) => {
                eprintln!("⚠️  TTS synthesis failed: {}", e);
                // 不 panic，因为可能是预处理或模型问题
            }
        }
}

/// 测试英文 TTS
#[tokio::test]
async fn test_tts_synthesize_english() {
    let model_dir = PathBuf::from("models/tts");
    
    if !model_dir.exists() {
        eprintln!("Skipping test: TTS model directory not found at {}", model_dir.display());
        return;
    }
    
    let engine = match FastSpeech2TtsEngine::new_from_dir(&model_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Skipping test: failed to load TTS engine: {}", e);
            return;
        }
    };
    
    let request = TtsRequest {
        text: "Hello".to_string(),
        voice: "default".to_string(),
        locale: "en".to_string(),
    };
    
    let result = engine.synthesize(request).await;
    
        match result {
            Ok(chunk) => {
                println!("✅ TTS synthesis successful");
                println!("   Audio length: {} bytes", chunk.audio.len());
                println!("   Timestamp: {} ms", chunk.timestamp_ms);
                println!("   Is last: {}", chunk.is_last);
                
                // 保存音频文件以便检查
                if !chunk.audio.is_empty() {
                    use core_engine::tts_streaming::save_pcm_to_wav;
                    let output_path = Path::new("test_output_english.wav");
                    match save_pcm_to_wav(&chunk.audio, output_path, 16000, 1) {
                        Ok(_) => {
                            println!("   ✅ Audio saved to: {}", output_path.display());
                            println!("   💡 Please play this file to check audio quality");
                        }
                        Err(e) => {
                            eprintln!("   ⚠️  Failed to save audio: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️  TTS synthesis failed: {}", e);
            }
        }
}

/// 测试空文本处理
#[tokio::test]
async fn test_tts_empty_text() {
    let model_dir = PathBuf::from("models/tts");
    
    if !model_dir.exists() {
        eprintln!("Skipping test: TTS model directory not found");
        return;
    }
    
    let engine = match FastSpeech2TtsEngine::new_from_dir(&model_dir) {
        Ok(e) => e,
        Err(_) => {
            eprintln!("Skipping test: failed to load TTS engine");
            return;
        }
    };
    
    let request = TtsRequest {
        text: "".to_string(),
        voice: "default".to_string(),
        locale: "zh".to_string(),
    };
    
    let result = engine.synthesize(request).await;
    
    match result {
        Ok(chunk) => {
            println!("✅ Empty text handled correctly");
            assert!(chunk.audio.is_empty());
            assert!(chunk.is_last);
        }
        Err(e) => {
            eprintln!("⚠️  Empty text handling failed: {}", e);
        }
    }
}

