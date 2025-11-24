//! 测试 M2M100 HTTP NMT 客户端
//! 
//! 使用方法：
//!   cargo run --example test_nmt_http_client
//! 
//! 前提条件：
//!   Python M2M100 NMT 服务已启动（http://127.0.0.1:5008）

use std::sync::Arc;
use core_engine::nmt_client::{LocalM2m100HttpClient, NmtClientAdapter};
use core_engine::nmt_incremental::{NmtIncremental, TranslationRequest};
use core_engine::types::PartialTranscript;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== M2M100 HTTP NMT 客户端测试 ===\n");
    
    // 1. 创建 HTTP 客户端
    println!("[1/3] 创建 HTTP 客户端...");
    let client = Arc::new(LocalM2m100HttpClient::new("http://127.0.0.1:5008"));
    println!("  ✅ 客户端创建成功\n");
    
    // 2. 创建适配器
    println!("[2/3] 创建 NMT 适配器...");
    let nmt = Arc::new(NmtClientAdapter::new(client));
    nmt.initialize().await?;
    println!("  ✅ 适配器初始化成功\n");
    
    // 3. 测试翻译
    println!("[3/3] 测试翻译...");
    
    // 测试 1: 中文到英文
    println!("\n测试 1: 中文 → 英文");
    let request1 = TranslationRequest {
        transcript: PartialTranscript {
            text: "你好，欢迎参加测试。".to_string(),
            confidence: 1.0,
            is_final: true,
        },
        target_language: "en".to_string(),
        wait_k: None,
    };
    
    let response1 = nmt.translate(request1).await?;
    println!("  原文: 你好，欢迎参加测试。");
    println!("  翻译: {}", response1.translated_text);
    println!("  稳定: {}\n", response1.is_stable);
    
    // 测试 2: 英文到中文
    println!("测试 2: 英文 → 中文");
    let request2 = TranslationRequest {
        transcript: PartialTranscript {
            text: "Hello, welcome to the test.".to_string(),
            confidence: 1.0,
            is_final: true,
        },
        target_language: "zh".to_string(),
        wait_k: None,
    };
    
    let response2 = nmt.translate(request2).await?;
    println!("  原文: Hello, welcome to the test.");
    println!("  翻译: {}", response2.translated_text);
    println!("  稳定: {}\n", response2.is_stable);
    
    // 清理
    nmt.finalize().await?;
    
    println!("✅ 所有测试通过！");
    
    Ok(())
}

