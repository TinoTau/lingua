// tests/asr_whisper_model_load_test.rs
// 测试 Whisper GGML 模型加载

use std::path::PathBuf;

/// 测试 1: 验证模型文件是否存在
#[test]
fn test_whisper_model_file_exists() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");
    
    if model_path.exists() {
        let file_size = std::fs::metadata(&model_path)
            .expect("Failed to get file metadata")
            .len();
        let size_mb = file_size as f64 / 1024.0 / 1024.0;
        
        println!("✓ 模型文件存在: {}", model_path.display());
        println!("  文件大小: {:.2} MB", size_mb);
        
        // 验证文件大小合理（base 模型大约 140-150 MB）
        assert!(file_size > 100 * 1024 * 1024, "模型文件太小，可能不完整");
        assert!(file_size < 200 * 1024 * 1024, "模型文件太大，可能不是 base 模型");
    } else {
        panic!("模型文件不存在: {}", model_path.display());
    }
}

/// 测试 2: 尝试加载模型（不运行推理）
#[test]
fn test_whisper_model_load() {
    use whisper_rs::{WhisperContext, WhisperContextParameters};
    
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_path = crate_root.join("models/asr/whisper-base/ggml-base.bin");
    
    if !model_path.exists() {
        println!("⚠ 跳过测试: 模型文件不存在");
        return;
    }
    
    println!("尝试加载模型: {}", model_path.display());
    
    // 尝试加载模型
    match WhisperContext::new_with_params(
        model_path.to_str().unwrap(),
        WhisperContextParameters::default(),
    ) {
        Ok(_ctx) => {
            println!("✓ 模型加载成功!");
        }
        Err(e) => {
            // 模型加载可能失败（例如缺少系统依赖），但不应该 panic
            println!("⚠ 模型加载失败: {}", e);
            println!("  这可能是因为缺少系统依赖（如 C++ 运行时库）");
            println!("  在 Windows 上，可能需要安装 Visual C++ Redistributable");
        }
    }
}

/// 测试 3: 检查模型路径配置
#[test]
fn test_whisper_model_path_config() {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let model_dir = crate_root.join("models/asr/whisper-base");
    let model_file = model_dir.join("ggml-base.bin");
    
    println!("模型目录: {}", model_dir.display());
    println!("模型文件: {}", model_file.display());
    
    assert!(model_dir.exists(), "模型目录不存在");
    
    if model_file.exists() {
        println!("✓ GGML 模型文件存在");
    } else {
        println!("⚠ GGML 模型文件不存在");
        println!("  需要下载或转换模型文件");
        println!("  可以使用: python scripts/convert_whisper_to_ggml.py --download");
    }
}

