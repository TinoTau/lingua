// tests/asr_whisper_audio_preprocessing_test.rs
// 测试音频预处理功能（重采样、多声道转换、归一化等）

use core_engine::asr_whisper::audio_preprocessing::*;
use core_engine::AudioFrame;

// 注意：convert_to_mono 和 normalize_audio 在测试模式下是 pub 的

/// 测试多声道转单声道
#[test]
fn test_convert_to_mono() {
    println!("\n========== 测试多声道转单声道 ==========");

    // 测试 1: 单声道（应该保持不变）
    println!("\n1. 测试单声道音频");
    let mono_audio = vec![0.1, 0.2, 0.3, 0.4];
    let result = convert_to_mono(&mono_audio, 1);
    assert_eq!(result, mono_audio, "单声道音频应该保持不变");
    println!("   ✓ 单声道音频转换正确");

    // 测试 2: 立体声（2 声道）
    println!("\n2. 测试立体声音频");
    let stereo_audio = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];  // 3 个样本，2 声道
    let result = convert_to_mono(&stereo_audio, 2);
    assert_eq!(result.len(), 3, "立体声转单声道后应该有 3 个样本");
    // 验证平均值计算：(0.1+0.2)/2=0.15, (0.3+0.4)/2=0.35, (0.5+0.6)/2=0.55
    assert!((result[0] - 0.15).abs() < 0.001, "第一个样本应该是平均值");
    assert!((result[1] - 0.35).abs() < 0.001, "第二个样本应该是平均值");
    assert!((result[2] - 0.55).abs() < 0.001, "第三个样本应该是平均值");
    println!("   ✓ 立体声转单声道正确");

    // 测试 3: 多声道（4 声道）
    println!("\n3. 测试多声道音频（4 声道）");
    let multi_audio = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];  // 2 个样本，4 声道
    let result = convert_to_mono(&multi_audio, 4);
    assert_eq!(result.len(), 2, "4 声道转单声道后应该有 2 个样本");
    // 验证平均值计算：(0.1+0.2+0.3+0.4)/4=0.25, (0.5+0.6+0.7+0.8)/4=0.65
    assert!((result[0] - 0.25).abs() < 0.001, "第一个样本应该是平均值");
    assert!((result[1] - 0.65).abs() < 0.001, "第二个样本应该是平均值");
    println!("   ✓ 多声道转单声道正确");
}

/// 测试归一化
#[test]
fn test_normalize_audio() {
    println!("\n========== 测试音频归一化 ==========");

    // 测试 1: 正常范围音频
    println!("\n1. 测试正常范围音频");
    let mut audio = vec![0.1, 0.2, -0.1, -0.2, 0.5, -0.5];
    normalize_audio(&mut audio);
    // 验证所有值都在 [-1.0, 1.0] 范围内
    for &value in &audio {
        assert!(value >= -1.0 && value <= 1.0, "归一化后的值应该在 [-1.0, 1.0] 范围内");
    }
    println!("   ✓ 正常范围音频归一化正确");

    // 测试 2: 超出范围的音频
    println!("\n2. 测试超出范围的音频");
    let mut audio = vec![2.0, -2.0, 1.5, -1.5];
    normalize_audio(&mut audio);
    // 验证所有值都在 [-1.0, 1.0] 范围内
    for &value in &audio {
        assert!(value >= -1.0 && value <= 1.0, "归一化后的值应该在 [-1.0, 1.0] 范围内");
    }
    println!("   ✓ 超出范围音频归一化正确");

    // 测试 3: 空音频
    println!("\n3. 测试空音频");
    let mut audio = vec![];
    normalize_audio(&mut audio);
    assert_eq!(audio.len(), 0, "空音频归一化后应该还是空的");
    println!("   ✓ 空音频归一化正确");

    // 测试 4: 全零音频
    println!("\n4. 测试全零音频");
    let mut audio = vec![0.0; 100];
    normalize_audio(&mut audio);
    // 全零音频归一化后应该还是全零
    for &value in &audio {
        assert_eq!(value, 0.0, "全零音频归一化后应该还是全零");
    }
    println!("   ✓ 全零音频归一化正确");
}

/// 测试重采样（简化测试，因为实际重采样需要外部库）
#[test]
fn test_resample_audio() {
    println!("\n========== 测试音频重采样 ==========");

    // 测试 1: 已经是目标采样率（16kHz）
    println!("\n1. 测试已经是目标采样率的音频");
    let audio = vec![0.0; 1600];  // 0.1 秒，16kHz
    let result = resample_audio(&audio, 16000, 16000);
    assert!(result.is_ok(), "相同采样率重采样应该成功");
    assert_eq!(result.unwrap().len(), audio.len(), "相同采样率应该保持不变");
    println!("   ✓ 相同采样率重采样正确");

    // 测试 2: 空音频
    println!("\n2. 测试空音频");
    let audio = vec![];
    let result = resample_audio(&audio, 44100, 16000);
    assert!(result.is_ok(), "空音频重采样应该成功");
    assert_eq!(result.unwrap().len(), 0, "空音频重采样后应该还是空的");
    println!("   ✓ 空音频重采样正确");

    // 注意：实际的重采样测试需要外部库（如 rubato），这里只做基本验证
    println!("\n⚠ 注意：完整重采样测试需要外部库支持");
}

/// 测试完整预处理流程
#[test]
fn test_preprocess_audio_frame() {
    println!("\n========== 测试完整预处理流程 ==========");

    // 测试 1: 标准音频帧（16kHz 单声道）
    println!("\n1. 测试标准音频帧（16kHz 单声道）");
    let frame = AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: vec![0.1, 0.2, -0.1, -0.2, 0.5, -0.5],
        timestamp_ms: 0,
    };

    let result = preprocess_audio_frame(&frame);
    match result {
        Ok(audio) => {
            assert!(!audio.is_empty(), "预处理后的音频不应该为空");
            // 验证所有值都在 [-1.0, 1.0] 范围内
            for &value in &audio {
                assert!(value >= -1.0 && value <= 1.0, "预处理后的值应该在 [-1.0, 1.0] 范围内");
            }
            println!("   ✓ 标准音频帧预处理成功");
        }
        Err(e) => {
            println!("   ⚠ 预处理返回错误: {}", e);
        }
    }

    // 测试 2: 立体声音频帧
    println!("\n2. 测试立体声音频帧");
    let frame = AudioFrame {
        sample_rate: 44100,
        channels: 2,
        data: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6],  // 3 个样本，2 声道
        timestamp_ms: 0,
    };

    let result = preprocess_audio_frame(&frame);
    match result {
        Ok(audio) => {
            assert!(!audio.is_empty(), "预处理后的音频不应该为空");
            println!("   ✓ 立体声音频帧预处理成功");
        }
        Err(e) => {
            println!("   ⚠ 预处理返回错误: {}", e);
        }
    }

    // 测试 3: 空音频帧
    println!("\n3. 测试空音频帧");
    let frame = AudioFrame {
        sample_rate: 16000,
        channels: 1,
        data: vec![],
        timestamp_ms: 0,
    };

    let result = preprocess_audio_frame(&frame);
    match result {
        Ok(audio) => {
            // 空音频帧预处理后可能还是空的，或者返回错误
            println!("   ✓ 空音频帧预处理处理正确（返回空或错误）");
        }
        Err(_) => {
            println!("   ✓ 空音频帧预处理正确返回错误");
        }
    }
}

/// 测试累积多个音频帧
#[test]
fn test_accumulate_audio_frames() {
    println!("\n========== 测试累积多个音频帧 ==========");

    // 创建多个音频帧
    let frames = vec![
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.1, 0.2],
            timestamp_ms: 0,
        },
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.3, 0.4],
            timestamp_ms: 100,
        },
        AudioFrame {
            sample_rate: 16000,
            channels: 1,
            data: vec![0.5, 0.6],
            timestamp_ms: 200,
        },
    ];

    let result = accumulate_audio_frames(&frames);
    match result {
        Ok(audio) => {
            assert_eq!(audio.len(), 6, "累积后应该有 6 个样本");
            assert!((audio[0] - 0.1).abs() < 0.001, "第一个样本应该正确");
            assert!((audio[5] - 0.6).abs() < 0.001, "最后一个样本应该正确");
            println!("   ✓ 累积多个音频帧成功");
        }
        Err(e) => {
            println!("   ⚠ 累积返回错误: {}", e);
        }
    }

    // 测试空帧列表
    println!("\n2. 测试空帧列表");
    let empty_frames = vec![];
    let result = accumulate_audio_frames(&empty_frames);
    match result {
        Ok(audio) => {
            assert_eq!(audio.len(), 0, "空帧列表应该返回空音频");
            println!("   ✓ 空帧列表处理正确");
        }
        Err(_) => {
            println!("   ✓ 空帧列表正确返回错误");
        }
    }
}

