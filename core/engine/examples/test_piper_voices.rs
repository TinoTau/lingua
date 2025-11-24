//! 测试 Piper TTS 可用的语音模型
//! 
//! 使用方法：
//!   cargo run --example test_piper_voices
//! 
//! 前提条件：
//!   Piper TTS 服务已启动（http://127.0.0.1:5005/tts）

use core_engine::tts_streaming::{TtsRequest, TtsStreaming, PiperHttpTts, PiperHttpConfig};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试 Piper TTS 可用语音模型 ===\n");
    
    // 创建 TTS 客户端
    let config = PiperHttpConfig::default();
    let tts = PiperHttpTts::new(config)?;
    
    // 测试不同的英文语音模型名称
    let english_voices = vec![
        "en_US-lessac-medium",
        "en_US-lessac-low",
        "en_US-amy-medium",
        "en_US-libritts-high",
        "en_US-joe-medium",
        "en_US-kathleen-low",
        "en_US-ryan-medium",
        "en_US-ryan-low",
        "en_US-amy-low",
        "en_US-libritts-medium",
    ];
    
    let test_text = "Hello, welcome to the test.";
    
    println!("测试文本: \"{}\"\n", test_text);
    println!("尝试不同的英文语音模型：\n");
    
    let mut success_count = 0;
    
    for voice in english_voices {
        print!("测试 {} ... ", voice);
        
        let request = TtsRequest {
            text: test_text.to_string(),
            voice: voice.to_string(),
            locale: "en".to_string(),
        };
        
        match tts.synthesize(request).await {
            Ok(result) => {
                println!("✅ 成功！音频长度: {} 字节", result.audio.len());
                
                // 保存成功的音频文件
                let filename = format!("test_output/test_voice_{}.wav", voice.replace("/", "_").replace("-", "_"));
                fs::write(&filename, &result.audio)?;
                println!("  💾 已保存: {}\n", filename);
                success_count += 1;
                
                // 找到第一个可用的就停止
                break;
            },
            Err(e) => {
                println!("❌ 失败: {}\n", e);
            }
        }
    }
    
    if success_count == 0 {
        println!("\n⚠️  所有英文语音模型都不可用！");
        println!("请检查 Piper TTS 服务配置，确保已安装英文语音模型。");
    } else {
        println!("\n✅ 找到可用的英文语音模型！");
    }
    
    tts.close().await?;
    
    Ok(())
}

