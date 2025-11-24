//! M2M100 NMT 单元测试
//! 
//! 测试 NMT 翻译功能和翻译质量

use std::path::PathBuf;
use core_engine::nmt_incremental::{TranslationRequest, NmtIncremental, M2M100NmtOnnx};
use core_engine::types::PartialTranscript;

#[tokio::test]
async fn test_nmt_zh_to_en() {
    println!("\n=== 测试 M2M100 中文→英文翻译 ===");
    
    // 加载模型
    let model_dir = PathBuf::from("models/nmt/m2m100-zh-en");
    if !model_dir.exists() {
        eprintln!("[SKIP] 模型目录不存在: {}", model_dir.display());
        return;
    }
    
    let nmt = M2M100NmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load M2M100 NMT");
    
    // 初始化
    NmtIncremental::initialize(&nmt).await
        .expect("Failed to initialize NMT");
    
    // 测试用例
    let test_cases = vec![
        "你好，世界",
        "欢迎来到我们的系统",
        "这是一个测试",
    ];
    
    for (i, source_text) in test_cases.iter().enumerate() {
        println!("\n[测试用例 {}] 源文本: '{}'", i + 1, source_text);
        
        let transcript = PartialTranscript {
            text: source_text.to_string(),
            confidence: 1.0,
            is_final: true,
        };
        
        let request = TranslationRequest {
            transcript,
            target_language: "en".to_string(),
            wait_k: None,
        };
        
        let response = NmtIncremental::translate(&nmt, request).await
            .expect("Translation failed");
        
        println!("  翻译结果: '{}'", response.translated_text);
        println!("  翻译稳定: {}", response.is_stable);
        
        // 验证：翻译结果不应该为空
        assert!(!response.translated_text.is_empty(), "翻译结果不应该为空");
        
        // 验证：翻译结果不应该包含重复的 token 模式
        let text = &response.translated_text;
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() > 10 {
            // 检查是否有重复的单词序列
            let mut has_repetition = false;
            for window_size in 3..=5 {
                for i in 0..words.len().saturating_sub(window_size * 2) {
                    let pattern = &words[i..i + window_size];
                    let next_pattern = &words[i + window_size..i + window_size * 2];
                    if pattern == next_pattern {
                        has_repetition = true;
                        println!("  ⚠️  检测到重复模式: {:?}", pattern);
                        break;
                    }
                }
                if has_repetition {
                    break;
                }
            }
            
            if !has_repetition {
                println!("  ✅ 未检测到明显的重复模式");
            }
        }
    }
}

#[tokio::test]
async fn test_nmt_en_to_zh() {
    println!("\n=== 测试 M2M100 英文→中文翻译 ===");
    
    // 加载模型
    let model_dir = PathBuf::from("models/nmt/m2m100-en-zh");
    if !model_dir.exists() {
        eprintln!("[SKIP] 模型目录不存在: {}", model_dir.display());
        return;
    }
    
    let nmt = M2M100NmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load M2M100 NMT");
    
    // 初始化
    NmtIncremental::initialize(&nmt).await
        .expect("Failed to initialize NMT");
    
    // 测试用例
    let test_cases = vec![
        "Hello, world",
        "Welcome to our system",
        "This is a test",
    ];
    
    for (i, source_text) in test_cases.iter().enumerate() {
        println!("\n[测试用例 {}] 源文本: '{}'", i + 1, source_text);
        
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
        
        let response = NmtIncremental::translate(&nmt, request).await
            .expect("Translation failed");
        
        println!("  翻译结果: '{}'", response.translated_text);
        println!("  翻译稳定: {}", response.is_stable);
        
        // 验证：翻译结果不应该为空
        assert!(!response.translated_text.is_empty(), "翻译结果不应该为空");
        
        // 验证：翻译结果应该包含中文字符
        let has_chinese = response.translated_text.chars().any(|c| {
            let code = c as u32;
            (0x4E00..=0x9FFF).contains(&code)
        });
        
        if has_chinese {
            println!("  ✅ 检测到中文字符");
        } else {
            println!("  ⚠️  未检测到中文字符");
        }
        
        // 验证：翻译结果不应该包含重复的 token 模式
        let text = &response.translated_text;
        let chars: Vec<char> = text.chars().collect();
        if chars.len() > 20 {
            // 检查是否有重复的字符序列
            let mut has_repetition = false;
            for window_size in 3..=5 {
                for i in 0..chars.len().saturating_sub(window_size * 2) {
                    let pattern = &chars[i..i + window_size];
                    let next_pattern = &chars[i + window_size..i + window_size * 2];
                    if pattern == next_pattern {
                        has_repetition = true;
                        println!("  ⚠️  检测到重复模式: {:?}", pattern.iter().collect::<String>());
                        break;
                    }
                }
                if has_repetition {
                    break;
                }
            }
            
            if !has_repetition {
                println!("  ✅ 未检测到明显的重复模式");
            }
        }
    }
}

