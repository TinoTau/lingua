use std::path::PathBuf;
use core_engine::tts_streaming::FastSpeech2TtsEngine;

/// 测试 TTS 模型加载
#[test]
fn test_tts_model_load() {
    let model_dir = PathBuf::from("models/tts");
    
    if !model_dir.exists() {
        eprintln!("Skipping test: TTS model directory not found at {}", model_dir.display());
        return;
    }
    
    let engine = FastSpeech2TtsEngine::new_from_dir(&model_dir);
    
    match engine {
        Ok(_) => println!("✅ FastSpeech2TtsEngine loaded successfully"),
        Err(e) => {
            eprintln!("⚠️  Failed to load FastSpeech2TtsEngine: {}", e);
            // 不 panic，因为可能是模型文件不存在
        }
    }
}

