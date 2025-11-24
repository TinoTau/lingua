// tests/nmt_comprehensive_test.rs
// 全面的 NMT 功能测试脚本
// 测试模型加载、Encoder 推理、Decoder 推理、完整翻译流程等
// 
// ⚠️ 已废弃：此测试使用 ONNX decoder，已不再使用。
// 当前系统已切换为 Python NMT 服务（HTTP 调用）。

use std::path::PathBuf;
use core_engine::nmt_incremental::{MarianNmtOnnx, LanguagePair, LanguageCode};
use core_engine::onnx_utils;

/// 测试 1: 模型加载和初始化
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_01_model_loading() {
    println!("\n========== Test 1: Model Loading ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found at {:?}", model_dir);
        return;
    }

    // 初始化 ONNX Runtime
    onnx_utils::init_onnx_runtime()
        .expect("Failed to initialize ONNX Runtime");

    // 加载模型
    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load MarianNmtOnnx model");

    println!("✓ Model loaded successfully");
    println!("  - Decoder start token ID: {}", nmt.decoder_start_token_id);
    println!("  - EOS token ID: {}", nmt.eos_token_id);
    println!("  - Max length: {}", nmt.max_length);
}

/// 测试 2: Encoder 推理（通过完整翻译流程间接测试）
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_02_encoder_inference() {
    println!("\n========== Test 2: Encoder Inference ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load model");

    // 测试编码
    let source_text = "Hello world";
    let source_ids = nmt.tokenizer.encode(source_text, true);
    println!("Source text: '{}'", source_text);
    println!("Encoded IDs: {:?} (length: {})", source_ids, source_ids.len());

    // 通过完整翻译流程间接测试 encoder（因为 run_encoder 是私有方法）
    let translated = nmt.translate(source_text)
        .expect("Failed to translate (this indirectly tests encoder)");

    println!("✓ Encoder inference successful (tested via full translation)");
    println!("  - Translation result: '{}'", translated);
    assert!(!translated.is_empty(), "Translation should not be empty");
}

/// 测试 3: Decoder 单步推理（通过完整翻译流程间接测试）
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_03_decoder_single_step() {
    println!("\n========== Test 3: Decoder Single Step ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load model");

    // 通过完整翻译流程间接测试 decoder（因为 decoder_step 是私有方法）
    let source_text = "Hello";
    let translated = nmt.translate(source_text)
        .expect("Failed to translate (this indirectly tests decoder)");

    println!("✓ Decoder step successful (tested via full translation)");
    println!("  - Translation result: '{}'", translated);
    assert!(!translated.is_empty(), "Translation should not be empty");
}

/// 测试 4: 完整翻译流程（短句）
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_04_full_translation_short() {
    println!("\n========== Test 4: Full Translation (Short Sentences) ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load model");

    let test_cases = vec![
        "Hello",
        "Hi",
        "Yes",
        "No",
    ];

    for source_text in test_cases {
        println!("\n--- Translating: '{}' ---", source_text);
        
        match nmt.translate(source_text) {
            Ok(translated) => {
                println!("✓ Translation: '{}' -> '{}'", source_text, translated);
                assert!(!translated.is_empty(), "Translation should not be empty");
            }
            Err(e) => {
                println!("✗ Translation failed: {}", e);
                panic!("Translation failed for '{}': {}", source_text, e);
            }
        }
    }
}

/// 测试 5: 完整翻译流程（中等长度句子）
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_05_full_translation_medium() {
    println!("\n========== Test 5: Full Translation (Medium Sentences) ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load model");

    let test_cases = vec![
        "Hello world",
        "How are you",
        "Thank you very much",
        "I love you",
    ];

    for source_text in test_cases {
        println!("\n--- Translating: '{}' ---", source_text);
        
        match nmt.translate(source_text) {
            Ok(translated) => {
                println!("✓ Translation: '{}' -> '{}'", source_text, translated);
                assert!(!translated.is_empty(), "Translation should not be empty");
            }
            Err(e) => {
                println!("✗ Translation failed: {}", e);
                panic!("Translation failed for '{}': {}", source_text, e);
            }
        }
    }
}

/// 测试 6: Tokenizer 编码/解码往返测试
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_06_tokenizer_roundtrip() {
    println!("\n========== Test 6: Tokenizer Roundtrip ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load model");

    let test_cases = vec![
        "Hello",
        "Hello world",
        "How are you",
    ];

    for text in test_cases {
        println!("\n--- Testing: '{}' ---", text);
        
        // 编码
        let encoded = nmt.tokenizer.encode(text, true);
        println!("  Encoded: {:?}", encoded);
        
        // 解码
        let decoded = nmt.tokenizer.decode(&encoded);
        println!("  Decoded: '{}'", decoded);
        
        // 注意：由于 tokenizer 的特性，往返可能不完全一致（子词切分）
        // 所以这里只验证解码不为空
        assert!(!decoded.is_empty(), "Decoded text should not be empty");
    }
}

/// 测试 7: 多语言对支持（如果可用）
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_07_language_pair_support() {
    println!("\n========== Test 7: Language Pair Support ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let base_dir = crate_root.join("models/nmt");

    // 测试语言对解析
    let pairs = vec![
        ("marian-en-zh", LanguageCode::En, LanguageCode::Zh),
        ("marian-zh-en", LanguageCode::Zh, LanguageCode::En),
    ];

    for (dir_name, expected_source, expected_target) in pairs {
        let model_dir = base_dir.join(dir_name);
        if !model_dir.exists() {
            println!("⚠ Model directory not found: {:?}", model_dir);
            continue;
        }

        match LanguagePair::from_model_dir(&model_dir) {
            Ok(pair) => {
                println!("✓ Language pair parsed: {} -> {:?} -> {:?}", dir_name, pair.source, pair.target);
                assert_eq!(pair.source, expected_source, "Source language mismatch");
                assert_eq!(pair.target, expected_target, "Target language mismatch");
            }
            Err(e) => {
                println!("✗ Failed to parse language pair from {}: {}", dir_name, e);
            }
        }
    }
}

/// 测试 8: 错误处理
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_08_error_handling() {
    println!("\n========== Test 8: Error Handling ==========");
    
    // 测试不存在的模型目录
    let non_existent_dir = PathBuf::from("/non/existent/path");
    match MarianNmtOnnx::new_from_dir(&non_existent_dir) {
        Ok(_) => panic!("Should have failed for non-existent directory"),
        Err(e) => {
            println!("✓ Correctly handled non-existent directory: {}", e);
        }
    }

    // 测试空字符串翻译（应该能处理，但可能返回空结果）
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");
    
    if model_dir.exists() {
        let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
            .expect("Failed to load model");
        
        // 空字符串应该能处理（可能返回空或特殊 token）
        match nmt.translate("") {
            Ok(translated) => {
                println!("✓ Empty string handled: '{}'", translated);
            }
            Err(e) => {
                println!("⚠ Empty string translation failed (acceptable): {}", e);
            }
        }
    }
}

/// 测试 9: 性能基准（简单）
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_09_performance_benchmark() {
    println!("\n========== Test 9: Performance Benchmark ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load model");

    let test_text = "Hello world";
    let iterations = 5;

    println!("Running {} iterations of translation for '{}'", iterations, test_text);
    
    let start = std::time::Instant::now();
    for i in 0..iterations {
        match nmt.translate(test_text) {
            Ok(translated) => {
                println!("  Iteration {}: '{}' -> '{}'", i + 1, test_text, translated);
            }
            Err(e) => {
                panic!("Translation failed at iteration {}: {}", i + 1, e);
            }
        }
    }
    let elapsed = start.elapsed();
    
    let avg_time = elapsed.as_millis() as f64 / iterations as f64;
    println!("✓ Performance benchmark completed");
    println!("  - Total time: {:?}", elapsed);
    println!("  - Average time per translation: {:.2} ms", avg_time);
}

/// 测试 10: 完整功能集成测试
#[test]
#[ignore] // 已废弃：使用 ONNX decoder，不再参与 CI
fn test_10_integration_test() {
    println!("\n========== Test 10: Integration Test ==========");
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    println!("Step 1: Initializing ONNX Runtime...");
    onnx_utils::init_onnx_runtime()
        .expect("Failed to initialize ONNX Runtime");
    println!("✓ ONNX Runtime initialized");

    println!("\nStep 2: Loading model...");
    let nmt = MarianNmtOnnx::new_from_dir(&model_dir)
        .expect("Failed to load model");
    println!("✓ Model loaded");

    println!("\nStep 3: Testing encoder and decoder (via full translation)...");
    let source_text = "Hello world";
    // 由于 run_encoder 和 decoder_step 是私有方法，我们通过完整翻译流程来测试它们
    println!("  (Encoder and decoder are tested indirectly through full translation)");

    println!("\nStep 5: Testing full translation...");
    let translated = nmt.translate(source_text)
        .expect("Failed to translate");
    println!("✓ Full translation successful: '{}' -> '{}'", source_text, translated);

    println!("\n========== All integration tests passed! ==========");
}

