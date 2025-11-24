/// M2M100 翻译功能测试
/// 
/// 测试 M2M100NmtOnnx 的完整翻译功能

use core_engine::nmt_incremental::{M2M100NmtOnnx, TranslationRequest, NmtIncremental};
use core_engine::types::PartialTranscript;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== M2M100 翻译功能测试 ===\n");

    // 1. 加载模型
    let model_dir = PathBuf::from("models/nmt/m2m100-en-zh");
    println!("[1/4] 加载 M2M100 模型: {}", model_dir.display());
    
    if !model_dir.exists() {
        eprintln!("❌ 模型目录不存在: {}", model_dir.display());
        return Err("模型目录不存在".into());
    }

    let model = M2M100NmtOnnx::new_from_dir(&model_dir)
        .map_err(|e| format!("Failed to load model: {}", e))?;
    println!("✅ 模型加载成功\n");

    // 2. 初始化
    println!("[2/4] 初始化模型...");
    NmtIncremental::initialize(&model).await
        .map_err(|e| format!("Failed to initialize: {}", e))?;
    println!("✅ 初始化成功\n");

    // 3. 测试翻译
    println!("[3/4] 测试翻译功能...");
    let test_cases = vec![
        "Hello world",
        "How are you?",
        "This is a test sentence.",
    ];

    for (i, source_text) in test_cases.iter().enumerate() {
        println!("\n测试用例 {}: \"{}\"", i + 1, source_text);
        
        let transcript = PartialTranscript {
            text: source_text.to_string(),
            confidence: 1.0,
            is_final: true,
        };

        let request = TranslationRequest {
            transcript,
            target_language: "zh".to_string(),
            wait_k: None,
        };

        match NmtIncremental::translate(&model, request).await {
            Ok(response) => {
                println!("  ✅ 翻译成功");
                println!("  原文（en）: \"{}\"", source_text);
                println!("  译文（zh）: \"{}\"", response.translated_text);
                println!("  稳定: {}", response.is_stable);
            }
            Err(e) => {
                eprintln!("  ❌ 翻译失败: {}", e);
            }
        }
    }
    println!();

    // 4. 清理
    println!("[4/4] 清理资源...");
    NmtIncremental::finalize(&model).await
        .map_err(|e| format!("Failed to finalize: {}", e))?;
    println!("✅ 清理完成\n");

    println!("=== 所有测试完成 ===");
    Ok(())
}

