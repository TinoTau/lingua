// tests/nmt_quick_test.rs
// 快速测试脚本：测试 NMT 的核心功能

use std::path::PathBuf;
use core_engine::nmt_incremental::MarianNmtOnnx;
use core_engine::onnx_utils;

/// 快速测试：模型加载和基本翻译
#[test]
fn test_quick_nmt() {
    println!("\n========== Quick NMT Test ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    // 1. 初始化
    println!("Step 1: Initializing ONNX Runtime...");
    onnx_utils::init_onnx_runtime()
        .expect("Failed to initialize ONNX Runtime");
    println!("✓ ONNX Runtime initialized");

    // 2. 加载模型
    println!("\nStep 2: Loading model...");
    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load model");
    println!("✓ Model loaded");

    // 3. 测试翻译
    println!("\nStep 3: Testing translation...");
    let test_cases = vec![
        "Hello",
        "Hello world",
    ];

    for source_text in test_cases {
        println!("\n  Translating: '{}'", source_text);
        match nmt.translate(source_text) {
            Ok(translated) => {
                println!("  ✓ Result: '{}'", translated);
                assert!(!translated.is_empty(), "Translation should not be empty");
            }
            Err(e) => {
                println!("  ✗ Failed: {}", e);
                panic!("Translation failed: {}", e);
            }
        }
    }

    println!("\n========== Quick test completed! ==========");
}

