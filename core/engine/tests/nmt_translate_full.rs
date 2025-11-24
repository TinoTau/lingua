// tests/nmt_translate_full.rs
// 测试完整的翻译流程：tokenizer 编码 -> ONNX 推理 -> tokenizer 解码
// 
// ⚠️ 已废弃：此测试使用 ONNX decoder，已不再使用。
// 当前系统已切换为 Python NMT 服务（HTTP 调用）。

use std::path::PathBuf;
use core_engine::nmt_incremental::MarianNmtOnnx;

/// 测试完整的翻译流程
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_full_translation_pipeline() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    // 加载模型
    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load MarianNmtOnnx model");

    // 测试翻译
    let test_cases = vec![
        "Hello",
        "Hello world",
        "How are you",
    ];

    for source_text in test_cases {
        println!("\n=== Translating: '{}' ===", source_text);
        
        match nmt.translate(source_text) {
            Ok(translated) => {
                println!("[OK] Translation successful: '{}' -> '{}'", source_text, translated);
                // 验证翻译结果不为空
                assert!(!translated.is_empty(), "Translation should not be empty");
                // 验证翻译结果不是重复的 token（之前的问题）
                let words: Vec<&str> = translated.split_whitespace().collect();
                if words.len() > 1 {
                    // 如果有多个词，检查是否都是相同的
                    let first_word = words[0];
                    let all_same = words.iter().all(|&w| w == first_word);
                    if all_same && words.len() > 3 {
                        println!("[WARN] Translation may have issues: all words are the same");
                    }
                }
            }
            Err(e) => {
                println!("[ERROR] Translation failed: {}", e);
                eprintln!("Error details: {:?}", e);
                panic!("Translation failed: {}", e);
            }
        }
    }
}

