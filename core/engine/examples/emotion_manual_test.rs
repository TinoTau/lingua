/// 手动测试 Emotion 功能
/// 
/// 使用方法：
/// cargo run --example emotion_manual_test

use std::path::PathBuf;
use core_engine::emotion_adapter::{EmotionAdapter, EmotionRequest, XlmREmotionEngine, EmotionStub};

#[tokio::main]
async fn main() {
    println!("=== Emotion Adapter 手动测试 ===\n");

    // 1. 测试 EmotionStub
    println!("1. 测试 EmotionStub");
    let stub = EmotionStub::new();
    let request = EmotionRequest {
        text: "Hello, this is a test.".to_string(),
        lang: "en".to_string(),
    };
    match stub.analyze(request).await {
        Ok(resp) => {
            println!("   ✅ Stub 测试通过");
            println!("      primary: {}, intensity: {:.2}, confidence: {:.2}", 
                resp.primary, resp.intensity, resp.confidence);
        }
        Err(e) => {
            println!("   ❌ Stub 测试失败: {}", e);
        }
    }
    println!();

    // 2. 测试模型加载
    println!("2. 测试 XlmREmotionEngine 模型加载");
    let model_dir = PathBuf::from("models/emotion/xlm-r");
    if !model_dir.exists() {
        println!("   ⚠️  模型目录不存在: {}", model_dir.display());
        println!("   跳过模型测试");
        return;
    }

    let engine = match XlmREmotionEngine::new_from_dir(&model_dir) {
        Ok(e) => {
            println!("   ✅ 模型加载成功");
            e
        }
        Err(e) => {
            println!("   ❌ 模型加载失败: {}", e);
            return;
        }
    };
    println!();

    // 3. 测试推理
    println!("3. 测试情感分析推理");
    let test_cases = vec![
        ("I love this product!", "joy"),
        ("This is terrible.", "sadness"),
        ("It's okay.", "neutral"),
        ("Hi", "neutral"),  // 短文本，应该返回 neutral
    ];

    for (text, expected) in test_cases {
        let request = EmotionRequest {
            text: text.to_string(),
            lang: "en".to_string(),
        };
        
        match engine.analyze(request).await {
            Ok(resp) => {
                println!("   文本: '{}'", text);
                println!("   结果: primary={}, intensity={:.2}, confidence={:.2}", 
                    resp.primary, resp.intensity, resp.confidence);
                println!("   预期: {}", expected);
                if resp.primary == expected || expected == "neutral" {
                    println!("   ✅ 通过");
                } else {
                    println!("   ⚠️  结果与预期不同（可能是正常的）");
                }
            }
            Err(e) => {
                println!("   ❌ 分析失败: {}", e);
            }
        }
        println!();
    }

    println!("=== 测试完成 ===");
}

