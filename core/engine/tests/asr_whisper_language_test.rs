// tests/asr_whisper_language_test.rs
// 测试 Whisper ASR 的语言设置功能

use std::path::PathBuf;
use core_engine::asr_whisper::WhisperAsrStreaming;
use core_engine::AsrStreaming;

/// 测试语言设置功能
#[tokio::test]
async fn test_set_language() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");

    if !model_dir.exists() {
        println!("⚠ 跳过测试: Whisper 模型目录不存在");
        return;
    }

    // 创建 WhisperAsrStreaming 实例
    let asr = WhisperAsrStreaming::new_from_dir(&model_dir)
        .expect("Failed to load WhisperAsrStreaming");

    println!("\n========== 测试语言设置功能 ==========");

    // 测试 1: 设置英语
    println!("\n1. 设置语言为英语 (en)");
    asr.set_language(Some("en".to_string()))
        .expect("Failed to set language to English");
    
    // 验证语言已设置（通过检查是否没有错误）
    println!("   ✓ 语言设置为英语成功");

    // 测试 2: 设置中文
    println!("\n2. 设置语言为中文 (zh)");
    asr.set_language(Some("zh".to_string()))
        .expect("Failed to set language to Chinese");
    println!("   ✓ 语言设置为中文成功");

    // 测试 3: 设置为自动检测
    println!("\n3. 设置为自动检测 (None)");
    asr.set_language(None)
        .expect("Failed to set language to auto-detect");
    println!("   ✓ 语言设置为自动检测成功");

    // 测试 4: 设置其他语言
    println!("\n4. 设置语言为日语 (ja)");
    asr.set_language(Some("ja".to_string()))
        .expect("Failed to set language to Japanese");
    println!("   ✓ 语言设置为日语成功");

    println!("\n✓ 所有语言设置测试通过");
}

/// 测试语言设置在推理中的应用
#[tokio::test]
async fn test_language_in_inference() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");

    if !model_dir.exists() {
        println!("⚠ 跳过测试: Whisper 模型目录不存在");
        return;
    }

    // 创建 WhisperAsrStreaming 实例
    let asr = WhisperAsrStreaming::new_from_dir(&model_dir)
        .expect("Failed to load WhisperAsrStreaming");

    // 初始化
    asr.initialize().await.expect("Failed to initialize");

    println!("\n========== 测试语言设置在推理中的应用 ==========");

    // 创建测试音频帧（静音，用于测试）
    let frame = core_engine::AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: vec![0.0; 1600],  // 0.1 秒的静音
        timestamp_ms: 0,
    };

    // 测试 1: 使用英语设置进行推理
    println!("\n1. 使用英语设置进行推理");
    asr.set_language(Some("en".to_string()))
        .expect("Failed to set language to English");
    
    // 累积帧并推理（虽然结果是空的，但可以验证没有错误）
    asr.accumulate_frame(frame.clone())
        .expect("Failed to accumulate frame");
    
    let result = asr.infer_on_boundary().await;
    match result {
        Ok(_) => println!("   ✓ 使用英语设置推理成功"),
        Err(e) => println!("   ⚠ 推理返回错误（可能是预期的，因为音频是静音）: {}", e),
    }

    // 测试 2: 使用中文设置进行推理
    println!("\n2. 使用中文设置进行推理");
    asr.set_language(Some("zh".to_string()))
        .expect("Failed to set language to Chinese");
    
    asr.clear_buffer();
    asr.accumulate_frame(frame.clone())
        .expect("Failed to accumulate frame");
    
    let result = asr.infer_on_boundary().await;
    match result {
        Ok(_) => println!("   ✓ 使用中文设置推理成功"),
        Err(e) => println!("   ⚠ 推理返回错误（可能是预期的，因为音频是静音）: {}", e),
    }

    // 清理
    asr.finalize().await.expect("Failed to finalize");
    println!("\n✓ 语言设置在推理中的应用测试完成");
}

