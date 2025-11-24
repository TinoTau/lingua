//! 直接测试 Piper TTS 服务
//! 
//! 使用方法：
//!   cargo run --example test_piper_tts_direct
//! 
//! 前提条件：
//!   Piper TTS 服务已启动（http://127.0.0.1:5005/tts）

use core_engine::tts_streaming::{TtsRequest, TtsStreaming, PiperHttpTts, PiperHttpConfig};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Piper TTS 直接测试 ===\n");
    
    // 1. 创建 TTS 客户端
    println!("[1/4] 创建 Piper TTS 客户端...");
    let config = PiperHttpConfig::default();
    let tts = PiperHttpTts::new(config)?;
    println!("  ✅ 客户端创建成功\n");
    
    // 2. 测试中文文本 + 中文语音
    println!("[2/4] 测试：中文文本 + 中文语音");
    let request1 = TtsRequest {
        text: "你好，欢迎参加测试。".to_string(),
        voice: "zh_CN-huayan-medium".to_string(),
        locale: "zh".to_string(),
    };
    
    match tts.synthesize(request1).await {
        Ok(result) => {
            println!("  ✅ 成功，音频长度: {} 字节", result.audio.len());
            let file1 = "test_output/test_tts_zh_text_zh_voice.wav";
            fs::write(file1, &result.audio)?;
            println!("  💾 已保存: {}\n", file1);
        },
        Err(e) => {
            println!("  ❌ 失败: {}\n", e);
        }
    }
    
    // 3. 测试英文文本 + 中文语音（当前代码的问题场景）
    println!("[3/4] 测试：英文文本 + 中文语音（问题场景）");
    let request2 = TtsRequest {
        text: "Hello, welcome to the test.".to_string(),
        voice: "zh_CN-huayan-medium".to_string(),
        locale: "zh".to_string(),
    };
    
    match tts.synthesize(request2).await {
        Ok(result) => {
            println!("  ✅ 成功，音频长度: {} 字节", result.audio.len());
            let file2 = "test_output/test_tts_en_text_zh_voice.wav";
            fs::write(file2, &result.audio)?;
            println!("  💾 已保存: {}\n", file2);
            println!("  ⚠️  注意：中文语音模型读英文文本，可能无法正确发音\n");
        },
        Err(e) => {
            println!("  ❌ 失败: {}\n", e);
        }
    }
    
    // 4. 测试英文文本 + 英文语音（如果可用）
    println!("[4/4] 测试：英文文本 + 英文语音（如果可用）");
    let request3 = TtsRequest {
        text: "Hello, welcome to the test.".to_string(),
        voice: "en_US-lessac-medium".to_string(),
        locale: "en".to_string(),
    };
    
    match tts.synthesize(request3).await {
        Ok(result) => {
            println!("  ✅ 成功，音频长度: {} 字节", result.audio.len());
            let file3 = "test_output/test_tts_en_text_en_voice.wav";
            fs::write(file3, &result.audio)?;
            println!("  💾 已保存: {}\n", file3);
        },
        Err(e) => {
            println!("  ❌ 失败: {}", e);
            println!("  ⚠️  英文语音模型不可用，这是正常的\n");
        }
    }
    
    println!("✅ 测试完成！");
    println!("\n请播放以下文件对比：");
    println!("  1. test_output/test_tts_zh_text_zh_voice.wav - 中文文本+中文语音（应该正常）");
    println!("  2. test_output/test_tts_en_text_zh_voice.wav - 英文文本+中文语音（问题场景）");
    println!("  3. test_output/test_tts_en_text_en_voice.wav - 英文文本+英文语音（如果可用）");
    
    tts.close().await?;
    
    Ok(())
}

