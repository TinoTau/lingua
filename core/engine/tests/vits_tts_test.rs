use std::path::PathBuf;
use core_engine::tts_streaming::{TtsStreaming, TtsRequest, VitsTtsEngine};
use std::sync::Arc;

const TEST_OUTPUT_DIR: &str = r"D:\Programs\github\lingua\test_output";

/// 测试 VITS TTS 引擎加载
#[test]
fn test_vits_tts_engine_load() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/tts/mms-tts-eng");
    
    if !model_dir.exists() {
        eprintln!("⚠️  跳过测试: 模型目录不存在 {}", model_dir.display());
        return;
    }
    
    let engine = VitsTtsEngine::new_from_dir(&model_dir);
    
    match engine {
        Ok(_) => println!("✅ VitsTtsEngine 加载成功"),
        Err(e) => {
            eprintln!("❌ 加载失败: {}", e);
            panic!("Failed to load VitsTtsEngine: {}", e);
        }
    }
}

/// 测试 VITS TTS 英文合成
#[tokio::test]
async fn test_vits_tts_synthesize_english() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/tts/mms-tts-eng");
    
    if !model_dir.exists() {
        eprintln!("⚠️  跳过测试: 模型目录不存在 {}", model_dir.display());
        return;
    }
    
    let engine = match VitsTtsEngine::new_from_dir(&model_dir) {
        Ok(e) => Arc::new(e),
        Err(e) => {
            eprintln!("⚠️  跳过测试: 加载模型失败: {}", e);
            return;
        }
    };
    
    let request = TtsRequest {
        text: "Hello from Lingua. This is a test of the VITS TTS engine.".to_string(),
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
            
            // 保存音频文件用于验证
            let output_dir = PathBuf::from(TEST_OUTPUT_DIR);
            std::fs::create_dir_all(&output_dir).ok();
            let output_path = output_dir.join("vits_tts_test_english.wav");
            
            // 使用 audio_utils 保存为 WAV
            use core_engine::tts_streaming::save_pcm_to_wav;
            if let Err(e) = save_pcm_to_wav(&chunk.audio, &output_path, 16000, 1) {
                eprintln!("⚠️  保存音频文件失败: {}", e);
            } else {
                println!("   ✅ Audio saved to: {}", output_path.display());
                println!("   💡 Please play this file to check audio quality");
            }
            
            // 验证音频数据不为空
            assert!(!chunk.audio.is_empty(), "Audio should not be empty");
            assert!(chunk.audio.len() > 1000, "Audio should have reasonable length (at least 1000 bytes)");
        }
        Err(e) => {
            eprintln!("❌ TTS synthesis failed: {}", e);
            panic!("TTS synthesis failed: {}", e);
        }
    }
}

/// 测试 VITS TTS 短文本合成
#[tokio::test]
async fn test_vits_tts_synthesize_short_text() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/tts/mms-tts-eng");
    
    if !model_dir.exists() {
        eprintln!("⚠️  跳过测试: 模型目录不存在 {}", model_dir.display());
        return;
    }
    
    let engine = match VitsTtsEngine::new_from_dir(&model_dir) {
        Ok(e) => Arc::new(e),
        Err(e) => {
            eprintln!("⚠️  跳过测试: 加载模型失败: {}", e);
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
            println!("✅ Short text synthesis successful");
            println!("   Audio length: {} bytes", chunk.audio.len());
            assert!(!chunk.audio.is_empty());
        }
        Err(e) => {
            eprintln!("❌ Short text synthesis failed: {}", e);
            panic!("Short text synthesis failed: {}", e);
        }
    }
}

/// 测试 VITS TTS 中文合成（vits-zh-aishell3）
#[tokio::test]
async fn test_vits_tts_synthesize_chinese() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let models_root = crate_root.join("models/tts");
    
    // 检查中文模型是否存在
    let model_dir_zh = models_root.join("vits-zh-aishell3");
    if !model_dir_zh.exists() {
        eprintln!("⚠️  跳过测试: 中文模型目录不存在 {}", model_dir_zh.display());
        eprintln!("   请先下载模型: git clone https://huggingface.co/csukuangfj/vits-zh-aishell3 {}", model_dir_zh.display());
        return;
    }
    
    // 使用 new_from_models_root 加载多语言模型
    let engine = match VitsTtsEngine::new_from_models_root(&models_root) {
        Ok(e) => Arc::new(e),
        Err(e) => {
            eprintln!("⚠️  跳过测试: 加载模型失败: {}", e);
            return;
        }
    };
    
    let request = TtsRequest {
        text: "你好，世界。这是一个测试。".to_string(),
        voice: "default".to_string(),
        locale: "zh".to_string(),
    };
    
    let result = engine.synthesize(request).await;
    
    match result {
        Ok(chunk) => {
            println!("✅ 中文 TTS 合成成功");
            println!("   Audio length: {} bytes", chunk.audio.len());
            println!("   Timestamp: {} ms", chunk.timestamp_ms);
            println!("   Is last: {}", chunk.is_last);
            
            // 保存音频文件用于验证
            let output_dir = PathBuf::from(TEST_OUTPUT_DIR);
            std::fs::create_dir_all(&output_dir).ok();
            let output_path = output_dir.join("vits_tts_test_chinese.wav");
            
            // 使用 audio_utils 保存为 WAV
            // vits-zh-aishell3 使用 22050 Hz 采样率
            use core_engine::tts_streaming::save_pcm_to_wav;
            if let Err(e) = save_pcm_to_wav(&chunk.audio, &output_path, 22050, 1) {
                eprintln!("⚠️  保存音频文件失败: {}", e);
            } else {
                println!("   ✅ Audio saved to: {}", output_path.display());
                println!("   💡 Please play this file to check audio quality");
            }
            
            // 验证音频数据不为空
            assert!(!chunk.audio.is_empty(), "Audio should not be empty");
            assert!(chunk.audio.len() > 1000, "Audio should have reasonable length (at least 1000 bytes)");
        }
        Err(e) => {
            eprintln!("❌ 中文 TTS 合成失败: {}", e);
            panic!("Chinese TTS synthesis failed: {}", e);
        }
    }
}

