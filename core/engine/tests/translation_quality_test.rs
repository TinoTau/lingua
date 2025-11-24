//! 翻译质量检查模块单元测试

use core_engine::translation_quality::TranslationQualityChecker;

#[test]
fn test_remove_repetitive_sequences() {
    let checker = TranslationQualityChecker::new(true);
    
    // 测试重复单词
    assert_eq!(
        checker.remove_repetitive_sequences("to to to to"),
        "to"
    );
    
    // 测试正常文本
    assert_eq!(
        checker.remove_repetitive_sequences("hello world"),
        "hello world"
    );
    
    // 测试重复但不超过阈值
    assert_eq!(
        checker.remove_repetitive_sequences("hello hello"),
        "hello hello"
    );
}

#[test]
fn test_is_suspicious_quality_english() {
    let checker = TranslationQualityChecker::new(true);
    
    // 非字母字符比例过高
    assert!(checker.is_suspicious_quality("???###$$$", "en"));
    
    // 正常英文
    assert!(!checker.is_suspicious_quality("Hello world", "en"));
    
    // 空文本
    assert!(checker.is_suspicious_quality("", "en"));
}

#[test]
fn test_is_suspicious_quality_chinese() {
    let checker = TranslationQualityChecker::new(true);
    
    // 全标点
    assert!(checker.is_suspicious_quality("？？？", "zh"));
    
    // 正常中文
    assert!(!checker.is_suspicious_quality("你好世界", "zh"));
    
    // 字符数极少
    assert!(checker.is_suspicious_quality("？", "zh"));
}

#[test]
fn test_check_and_fix() {
    let checker = TranslationQualityChecker::new(true);
    
    // 测试重复序列修复
    let result = checker.check_and_fix("Hello", "to to to to", "en");
    assert_eq!(result, "to");
    
    // 测试正常文本
    let result = checker.check_and_fix("Hello", "Hello", "en");
    assert_eq!(result, "Hello");
}

#[test]
fn test_remove_excessive_punctuation() {
    let checker = TranslationQualityChecker::new(true);
    
    // 测试连续标点
    assert_eq!(
        checker.remove_excessive_punctuation("test..."),
        "test."
    );
    
    // 测试中文连续标点
    assert_eq!(
        checker.remove_excessive_punctuation("测试。。。"),
        "测试。"
    );
}

