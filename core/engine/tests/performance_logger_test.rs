//! 性能日志模块测试

use core_engine::performance_logger::{PerformanceLog, PerformanceLogger};

#[test]
fn test_performance_log_creation() {
    let log = PerformanceLog::new(
        "test-id".to_string(),
        "zh".to_string(),
        "en".to_string(),
        100,
        200,
        150,
        450,
        true,
    );
    
    assert_eq!(log.id, "test-id");
    assert_eq!(log.src_lang, "zh");
    assert_eq!(log.tgt_lang, "en");
    assert_eq!(log.asr_ms, 100);
    assert_eq!(log.nmt_ms, 200);
    assert_eq!(log.tts_ms, 150);
    assert_eq!(log.total_ms, 450);
    assert!(log.ok);
}

#[test]
fn test_performance_log_suspect_translation_short() {
    let mut log = PerformanceLog::new(
        "test-id".to_string(),
        "zh".to_string(),
        "en".to_string(),
        100,
        200,
        150,
        450,
        true,
    );
    
    // 原文长度 > 20，译文长度 < 3，应该被标记为可疑
    let src_text = "这是一个很长的测试文本，用来测试可疑翻译检测功能";
    let tgt_text = "Hi";
    
    log.check_suspect_translation(src_text, tgt_text);
    
    assert_eq!(log.src_text_len, Some(src_text.len()));
    assert_eq!(log.tgt_text_len, Some(tgt_text.len()));
    assert_eq!(log.suspect_translation, Some(true));
}

#[test]
fn test_performance_log_suspect_translation_non_alpha() {
    let mut log = PerformanceLog::new(
        "test-id".to_string(),
        "zh".to_string(),
        "en".to_string(),
        100,
        200,
        150,
        450,
        true,
    );
    
    // 非字母字符比例 > 70%，应该被标记为可疑
    let src_text = "测试";
    let tgt_text = "???###$$$%%%";
    
    log.check_suspect_translation(src_text, tgt_text);
    
    assert_eq!(log.suspect_translation, Some(true));
}

#[test]
fn test_performance_log_normal_translation() {
    let mut log = PerformanceLog::new(
        "test-id".to_string(),
        "zh".to_string(),
        "en".to_string(),
        100,
        200,
        150,
        450,
        true,
    );
    
    // 正常翻译，不应该被标记为可疑
    let src_text = "你好";
    let tgt_text = "Hello";
    
    log.check_suspect_translation(src_text, tgt_text);
    
    assert_eq!(log.suspect_translation, Some(false));
}

#[test]
fn test_performance_log_to_json() {
    let log = PerformanceLog::new(
        "test-id".to_string(),
        "zh".to_string(),
        "en".to_string(),
        100,
        200,
        150,
        450,
        true,
    );
    
    let json = log.to_json();
    assert!(json.contains("test-id"));
    assert!(json.contains("zh"));
    assert!(json.contains("en"));
    assert!(json.contains("100"));
    assert!(json.contains("200"));
    assert!(json.contains("150"));
    assert!(json.contains("450"));
}

#[test]
fn test_performance_logger_enabled() {
    let logger = PerformanceLogger::new(true, true);
    let log = PerformanceLog::new(
        "test-id".to_string(),
        "zh".to_string(),
        "en".to_string(),
        100,
        200,
        150,
        450,
        true,
    );
    
    // 应该能正常记录（不会 panic）
    logger.log(&log);
    assert!(true);
}

#[test]
fn test_performance_logger_disabled() {
    let logger = PerformanceLogger::new(false, false);
    let log = PerformanceLog::new(
        "test-id".to_string(),
        "zh".to_string(),
        "en".to_string(),
        100,
        200,
        150,
        450,
        true,
    );
    
    // 禁用时应该不输出（不会 panic）
    logger.log(&log);
    assert!(true);
}

#[test]
fn test_performance_logger_suspect_warning() {
    let logger = PerformanceLogger::new(true, true);
    let mut log = PerformanceLog::new(
        "test-id".to_string(),
        "zh".to_string(),
        "en".to_string(),
        100,
        200,
        150,
        450,
        true,
    );
    
    // 标记为可疑翻译
    log.check_suspect_translation("这是一个很长的测试文本", "Hi");
    
    // 应该能正常记录可疑翻译警告（不会 panic）
    logger.log(&log);
    assert!(true);
}

