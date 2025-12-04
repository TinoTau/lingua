//! 独立测试程序：验证 NMT 服务（M2M100 HTTP）
//! 
//! 使用方法：
//!   cargo run --example test_nmt_standalone
//! 
//! 前提条件：
//!   Python M2M100 NMT 服务已启动（http://127.0.0.1:5008）

use std::sync::Arc;
use core_engine::nmt_client::{LocalM2m100HttpClient, NmtClientAdapter};
use core_engine::nmt_incremental::{NmtIncremental, TranslationRequest};
use core_engine::types::PartialTranscript;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== NMT 独立测试程序 ===\n");

    // 检查服务是否运行
    println!("[1/4] 检查 NMT 服务状态...");
    let service_url = "http://127.0.0.1:5008";
    let health_url = format!("{}/health", service_url);
    let client = reqwest::Client::new();
    
    match client.get(&health_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("[OK] 服务正在运行: {}", service_url);
            } else {
                eprintln!("[ERROR] 服务返回错误状态: {}", resp.status());
                return Err("Service health check failed".into());
            }
        }
        Err(e) => {
            eprintln!("[ERROR] 无法连接到服务: {}", e);
            eprintln!("[INFO] 请确保 NMT 服务正在运行: {}", service_url);
            return Err("Service not available".into());
        }
    }

    // 创建 HTTP 客户端
    println!("\n[2/4] 创建 NMT HTTP 客户端...");
    let http_client = Arc::new(LocalM2m100HttpClient::new(service_url));
    println!("  ✅ 客户端创建成功");

    // 创建适配器
    println!("\n[3/4] 创建 NMT 适配器...");
    let nmt = Arc::new(NmtClientAdapter::new(http_client));
    nmt.initialize().await?;
    println!("  ✅ 适配器初始化成功");

    // 测试翻译
    println!("\n[4/4] 测试翻译功能...");

    // 测试用例
    let test_cases = vec![
        (
            "你好，欢迎参加测试。",
            "zh",
            "en",
            "Chinese to English",
        ),
        (
            "Hello, welcome to the test.",
            "en",
            "zh",
            "English to Chinese",
        ),
        (
            "这是一个多语言翻译系统的测试。",
            "zh",
            "en",
            "Chinese to English (longer text)",
        ),
        (
            "The quick brown fox jumps over the lazy dog.",
            "en",
            "zh",
            "English to Chinese (pangram)",
        ),
    ];

    for (i, (text, source_lang, target_lang, description)) in test_cases.iter().enumerate() {
        println!("\n  测试 {}: {}", i + 1, description);
        println!("    原文 ({}): {}", source_lang, text);
        
        let request = TranslationRequest {
            transcript: PartialTranscript {
                text: text.to_string(),
                confidence: 1.0,
                is_final: true,
            },
            target_language: target_lang.to_string(),
            wait_k: None,
        };

        let start_time = std::time::Instant::now();
        let response = nmt.translate(request).await?;
        let elapsed = start_time.elapsed();

        println!("    翻译 ({}): {}", target_lang, response.translated_text);
        println!("    稳定: {}", response.is_stable);
        if let Some(ref speaker_id) = response.speaker_id {
            println!("    说话者ID: {}", speaker_id);
        }
        println!("    耗时: {:?}", elapsed);
    }

    // 清理
    nmt.finalize().await?;

    println!("\n=== 测试完成 ===");
    Ok(())
}

