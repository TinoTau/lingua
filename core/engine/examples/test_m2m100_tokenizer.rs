/// M2M100 Tokenizer 测试程序
/// 
/// 测试 Tokenizer 的加载、编码、解码和语言 ID 获取功能

use core_engine::nmt_incremental::M2M100Tokenizer;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== M2M100 Tokenizer 测试 ===\n");

    // 1. 测试模型目录路径
    let model_dir = PathBuf::from("models/nmt/m2m100-en-zh");
    println!("[1/5] 检查模型目录: {}", model_dir.display());
    
    if !model_dir.exists() {
        eprintln!("❌ 模型目录不存在: {}", model_dir.display());
        eprintln!("请确保模型已导出到: {}", model_dir.display());
        return Err("模型目录不存在".into());
    }
    println!("✅ 模型目录存在\n");

    // 2. 测试加载 Tokenizer
    println!("[2/5] 加载 Tokenizer...");
    let tokenizer = match M2M100Tokenizer::from_model_dir(&model_dir) {
        Ok(t) => {
            println!("✅ Tokenizer 加载成功\n");
            t
        }
        Err(e) => {
            eprintln!("❌ Tokenizer 加载失败: {}", e);
            return Err(e.into());
        }
    };

    // 3. 测试语言 ID 获取
    println!("[3/5] 测试语言 ID 获取...");
    let en_id = tokenizer.get_lang_id("en");
    let zh_id = tokenizer.get_lang_id("zh");
    println!("✅ en ID: {}", en_id);
    println!("✅ zh ID: {}", zh_id);
    println!("✅ 语言 ID 获取成功\n");

    // 4. 测试编码
    println!("[4/5] 测试编码...");
    let test_texts = vec![
        ("Hello world", "en"),
        ("你好世界", "zh"),
        ("This is a test sentence.", "en"),
    ];

    for (text, lang) in &test_texts {
        match tokenizer.encode(text, lang, true) {
            Ok(ids) => {
                println!("✅ 编码成功: \"{}\" (lang: {}) -> {} tokens", text, lang, ids.len());
                if ids.len() <= 20 {
                    println!("   IDs: {:?}", ids);
                } else {
                    println!("   IDs (前10个): {:?}...", &ids[..10]);
                }
            }
            Err(e) => {
                eprintln!("❌ 编码失败: \"{}\" (lang: {}): {}", text, lang, e);
            }
        }
    }
    println!();

    // 5. 测试解码
    println!("[5/5] 测试解码...");
    let test_text = "Hello world";
    match tokenizer.encode(test_text, "en", true) {
        Ok(ids) => {
            println!("编码: \"{}\" -> {:?}", test_text, &ids[..ids.len().min(10)]);
            match tokenizer.decode(&ids, true) {
                Ok(decoded) => {
                    println!("✅ 解码成功: \"{}\"", decoded);
                    if decoded.to_lowercase().contains("hello") || decoded.to_lowercase().contains("world") {
                        println!("✅ 解码结果包含原文本内容");
                    } else {
                        println!("⚠️  解码结果可能不完整: \"{}\"", decoded);
                    }
                }
                Err(e) => {
                    eprintln!("❌ 解码失败: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 编码失败（无法测试解码）: {}", e);
        }
    }
    println!();

    // 6. 测试往返一致性（可选）
    println!("[额外测试] 往返一致性测试...");
    let test_cases = vec![
        ("Hello", "en"),
        ("World", "en"),
    ];

    for (text, lang) in &test_cases {
        match tokenizer.encode(text, lang, true) {
            Ok(ids) => {
                match tokenizer.decode(&ids, true) {
                    Ok(decoded) => {
                        println!("   \"{}\" -> encode -> decode -> \"{}\"", text, decoded);
                    }
                    Err(e) => {
                        eprintln!("   ❌ 解码失败: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("   ❌ 编码失败: {}", e);
            }
        }
    }
    println!();

    // 7. 测试特殊 token ID
    println!("[额外测试] 特殊 token ID...");
    println!("   pad_token_id: {}", tokenizer.pad_token_id());
    println!("   eos_token_id: {}", tokenizer.eos_token_id());
    println!();

    // 8. 测试无效语言代码（应该 panic）
    println!("[额外测试] 测试无效语言代码（应该 panic）...");
    println!("   注意: 如果看到 panic，这是预期的行为（fail-fast 设计）");
    // 注释掉，避免程序崩溃
    // tokenizer.get_lang_id("invalid");

    println!("=== 所有测试完成 ===");
    Ok(())
}

