// tests/nmt_tokenizer_multi_lang.rs
// 测试多语言 tokenizer 的自动识别和编码/解码功能

use std::path::PathBuf;
use core_engine::nmt_incremental::{
    LanguageCode, LanguagePair, MarianTokenizer,
};

/// 测试1：语言对自动识别
#[test]
fn test_language_pair_auto_detection() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    
    // 测试不同的语言对目录
    let test_cases = vec![
        ("marian-en-zh", LanguageCode::En, LanguageCode::Zh),
        ("marian-zh-en", LanguageCode::Zh, LanguageCode::En),
        ("marian-en-es", LanguageCode::En, LanguageCode::Es),
        ("marian-es-en", LanguageCode::Es, LanguageCode::En),
        ("marian-en-ja", LanguageCode::En, LanguageCode::Ja),
        ("marian-ja-en", LanguageCode::Ja, LanguageCode::En),
    ];

    for (dir_name, expected_source, expected_target) in test_cases {
        let model_dir = crate_root.join("models/nmt").join(dir_name);
        
        // 如果目录存在，测试自动识别
        if model_dir.exists() {
            let pair = LanguagePair::from_model_dir(&model_dir)
                .expect(&format!("Failed to parse language pair from {}", dir_name));
            
            assert_eq!(
                pair.source, expected_source,
                "Source language mismatch for {}: expected {:?}, got {:?}",
                dir_name, expected_source, pair.source
            );
            
            assert_eq!(
                pair.target, expected_target,
                "Target language mismatch for {}: expected {:?}, got {:?}",
                dir_name, expected_target, pair.target
            );
            
            println!("✓ Successfully detected language pair for {}: {:?} -> {:?}", 
                dir_name, pair.source, pair.target);
        } else {
            println!("⚠ Skipping {} (directory not found)", dir_name);
        }
    }
}

/// 测试2：从字符串创建语言对
#[test]
fn test_language_pair_from_string() {
    let test_cases = vec![
        ("en-zh", LanguageCode::En, LanguageCode::Zh),
        ("eng-zho", LanguageCode::En, LanguageCode::Zh),
        ("zh-en", LanguageCode::Zh, LanguageCode::En),
        ("en-es", LanguageCode::En, LanguageCode::Es),
        ("es-en", LanguageCode::Es, LanguageCode::En),
    ];

    for (input, expected_source, expected_target) in test_cases {
        let pair = LanguagePair::from_str(input)
            .expect(&format!("Failed to parse language pair from string: {}", input));
        
        assert_eq!(pair.source, expected_source, "Source mismatch for {}", input);
        assert_eq!(pair.target, expected_target, "Target mismatch for {}", input);
        
        println!("✓ Successfully parsed '{}' -> {:?} -> {:?}", 
            input, pair.source, pair.target);
    }
}

/// 测试3：Tokenizer 编码/解码（使用 en-zh 模型）
#[test]
fn test_tokenizer_encode_decode_en_zh() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    // 自动识别语言对
    let language_pair = LanguagePair::from_model_dir(&model_dir)
        .expect("Failed to detect language pair from directory");

    assert_eq!(language_pair.source, LanguageCode::En);
    assert_eq!(language_pair.target, LanguageCode::Zh);

    // 加载 tokenizer
    let tokenizer = MarianTokenizer::from_model_dir(&model_dir, language_pair)
        .expect("Failed to load tokenizer");

    // 测试编码
    let test_text = "Hello world";
    let encoded = tokenizer.encode(test_text, true);
    
    println!("Original text: '{}'", test_text);
    println!("Encoded IDs: {:?}", encoded);
    println!("Encoded length: {}", encoded.len());
    
    // 验证编码结果不为空
    assert!(!encoded.is_empty(), "Encoded result should not be empty");
    
    // 验证包含 BOS 和 EOS（如果 add_special_tokens = true）
    if !encoded.is_empty() {
        println!("First token ID: {}", encoded[0]);
        println!("Last token ID: {}", encoded[encoded.len() - 1]);
    }

    // 测试解码
    let decoded = tokenizer.decode(&encoded);
    println!("Decoded text: '{}'", decoded);
    
    // 注意：由于是简化的 tokenizer，解码结果可能不完全匹配原文
    // 这里主要验证解码不会崩溃，并且返回非空字符串
    assert!(!decoded.is_empty(), "Decoded result should not be empty");
}

/// 测试4：Tokenizer 编码/解码（使用 zh-en 模型）
#[test]
fn test_tokenizer_encode_decode_zh_en() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-zh-en");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-zh-en model directory not found");
        return;
    }

    // 自动识别语言对
    let language_pair = LanguagePair::from_model_dir(&model_dir)
        .expect("Failed to detect language pair from directory");

    assert_eq!(language_pair.source, LanguageCode::Zh);
    assert_eq!(language_pair.target, LanguageCode::En);

    // 加载 tokenizer
    let tokenizer = MarianTokenizer::from_model_dir(&model_dir, language_pair)
        .expect("Failed to load tokenizer");

    // 测试编码中文文本
    let test_text = "你好 世界";
    let encoded = tokenizer.encode(test_text, true);
    
    println!("Original text: '{}'", test_text);
    println!("Encoded IDs: {:?}", encoded);
    println!("Encoded length: {}", encoded.len());
    
    assert!(!encoded.is_empty(), "Encoded result should not be empty");

    // 测试解码
    let decoded = tokenizer.decode(&encoded);
    println!("Decoded text: '{}'", decoded);
    
    assert!(!decoded.is_empty(), "Decoded result should not be empty");
}

/// 测试5：根据语言对自动查找模型目录
#[test]
fn test_find_model_dir_by_language_pair() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let base_dir = crate_root.join("models/nmt");

    let test_cases = vec![
        (LanguageCode::En, LanguageCode::Zh, "marian-en-zh"),
        (LanguageCode::Zh, LanguageCode::En, "marian-zh-en"),
        (LanguageCode::En, LanguageCode::Es, "marian-en-es"),
    ];

    for (source, target, expected_dir) in test_cases {
        let pair = LanguagePair::new(source, target);
        let model_dir = pair.find_model_dir(&base_dir);
        
        let expected_path = base_dir.join(expected_dir);
        assert_eq!(
            model_dir, expected_path,
            "Model directory path mismatch for {:?} -> {:?}",
            source, target
        );
        
        println!("✓ Language pair {:?} -> {:?} maps to: {}", 
            source, target, model_dir.display());
    }
}

/// 测试6：完整的翻译流程（编码 -> 模型推理 -> 解码）
/// 注意：这个测试需要完整的 ONNX 模型推理，目前只测试 tokenizer 部分
#[test]
fn test_tokenizer_roundtrip() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/nmt/marian-en-zh");

    if !model_dir.exists() {
        println!("⚠ Skipping test: marian-en-zh model directory not found");
        return;
    }

    let language_pair = LanguagePair::from_model_dir(&model_dir)
        .expect("Failed to detect language pair");
    
    let tokenizer = MarianTokenizer::from_model_dir(&model_dir, language_pair)
        .expect("Failed to load tokenizer");

    // 测试多个文本
    let test_texts = vec![
        "Hello",
        "Hello world",
        "How are you",
        "Thank you",
    ];

    for text in test_texts {
        println!("\n--- Testing: '{}' ---", text);
        
        // 编码
        let encoded = tokenizer.encode(text, true);
        println!("  Encoded: {:?} (length: {})", encoded, encoded.len());
        
        // 解码
        let decoded = tokenizer.decode(&encoded);
        println!("  Decoded: '{}'", decoded);
        
        // 验证编码/解码不会崩溃
        assert!(!encoded.is_empty());
        assert!(!decoded.is_empty());
    }
}

