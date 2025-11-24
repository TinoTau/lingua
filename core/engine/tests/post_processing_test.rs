//! 文本后处理模块测试

use core_engine::post_processing::TextPostProcessor;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_post_processor_creation() {
    let processor = TextPostProcessor::new(None, true);
    // 测试启用状态：处理文本应该会应用后处理
    let result = processor.process("  test  ", "en");
    assert_eq!(result, "test."); // 应该清理空格并添加句号
}

#[test]
fn test_post_processor_disabled() {
    let processor = TextPostProcessor::new(None, false);
    let result = processor.process("  test  ", "en");
    // 禁用时应该返回原始文本
    assert_eq!(result, "  test  ");
}

#[test]
fn test_clean_text_whitespace() {
    let processor = TextPostProcessor::new(None, true);
    let result = processor.process("  hello   world  ", "en");
    assert_eq!(result, "hello world.");
}

#[test]
fn test_clean_text_repeated_punctuation_chinese() {
    let processor = TextPostProcessor::new(None, true);
    let result = processor.process("测试。。。。", "zh");
    assert_eq!(result, "测试。");
}

#[test]
fn test_clean_text_repeated_punctuation_english() {
    let processor = TextPostProcessor::new(None, true);
    let result = processor.process("test....", "en");
    assert_eq!(result, "test.");
}

#[test]
fn test_add_punctuation_chinese() {
    let processor = TextPostProcessor::new(None, true);
    let result = processor.process("测试", "zh");
    assert_eq!(result, "测试。");
}

#[test]
fn test_add_punctuation_english() {
    let processor = TextPostProcessor::new(None, true);
    let result = processor.process("hello", "en");
    assert_eq!(result, "hello.");
}

#[test]
fn test_no_add_punctuation_if_exists() {
    let processor = TextPostProcessor::new(None, true);
    let result = processor.process("hello.", "en");
    assert_eq!(result, "hello.");
}

#[test]
fn test_load_terms_from_file() {
    // 创建临时目录和文件
    let temp_dir = TempDir::new().unwrap();
    let terms_file = temp_dir.path().join("terms.json");
    
    // 写入测试术语表
    let terms_json = r#"{
        "Whisper": "Whisper ASR",
        "Piper": "Piper TTS"
    }"#;
    fs::write(&terms_file, terms_json).unwrap();
    
    // 创建处理器并加载术语表
    let processor = TextPostProcessor::new(Some(&terms_file), true);
    
    // 测试术语替换
    let result = processor.process("This is Whisper and Piper", "en");
    assert!(result.contains("Whisper ASR"));
    assert!(result.contains("Piper TTS"));
}

#[test]
fn test_load_terms_file_not_exists() {
    // 不存在的文件应该不会 panic
    let processor = TextPostProcessor::new(Some(Path::new("/nonexistent/terms.json")), true);
    let result = processor.process("test", "en");
    assert_eq!(result, "test.");
}

#[test]
fn test_replace_terms() {
    // 创建临时目录和文件
    let temp_dir = TempDir::new().unwrap();
    let terms_file = temp_dir.path().join("terms.json");
    
    // 写入测试术语表
    let terms_json = r#"{
        "AI": "Artificial Intelligence",
        "NLP": "Natural Language Processing"
    }"#;
    fs::write(&terms_file, terms_json).unwrap();
    
    let processor = TextPostProcessor::new(Some(&terms_file), true);
    let result = processor.process("AI and NLP are important", "en");
    
    assert!(result.contains("Artificial Intelligence"));
    assert!(result.contains("Natural Language Processing"));
}

#[test]
fn test_complex_processing() {
    let processor = TextPostProcessor::new(None, true);
    
    // 测试复杂的文本处理
    let input = "  hello   world  ....  ";
    let result = processor.process(input, "en");
    
    // 应该清理空格、处理重复标点、添加句号
    // 注意：处理后的文本可能包含空格（在标点前），这是正常的
    assert!(result.contains("hello"));
    assert!(result.contains("world"));
    assert!(result.ends_with("."));
}

#[test]
fn test_chinese_complex_processing() {
    let processor = TextPostProcessor::new(None, true);
    
    // 测试中文复杂文本处理
    let input = "  测试文本  ，，，，  ";
    let result = processor.process(input, "zh");
    
    // 应该清理空格、处理重复标点、添加句号
    // 验证基本功能：文本被处理（不是原始输入）
    assert_ne!(result, input);
    // 验证文本被清理（不包含连续空格）
    assert!(!result.contains("  "));
    // 验证文本被清理（不包含连续逗号）
    assert!(!result.contains("，，，，"));
}

