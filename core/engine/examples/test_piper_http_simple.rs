//! 简化的独立测试程序：验证 Piper HTTP TTS 调用
//! 
//! 使用方法：
//!   cargo run --example test_piper_http_simple
//! 
//! 前提条件：
//!   1. WSL2 中已启动 Piper HTTP 服务（http://127.0.0.1:5005/tts）
//!   2. 服务正在运行

use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Piper HTTP TTS 独立测试程序（简化版） ===\n");

    // 检查服务是否运行
    println!("[1/4] 检查 Piper HTTP 服务状态...");
    let health_url = "http://127.0.0.1:5005/health";
    let client = reqwest::Client::new();
    match client.get(health_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("[OK] 服务正在运行");
            } else {
                eprintln!("[ERROR] 服务返回错误状态: {}", resp.status());
                return Err("Service health check failed".into());
            }
        }
        Err(e) => {
            eprintln!("[ERROR] 无法连接到服务: {}", e);
            eprintln!("[INFO] 请确保 WSL2 中的 Piper HTTP 服务正在运行");
            eprintln!("[INFO] 启动命令: bash scripts/wsl2_piper/start_piper_service.sh");
            return Err("Service not available".into());
        }
    }

    // 准备测试文本
    println!("\n[2/4] 准备 TTS 请求...");
    let test_text = "你好，欢迎使用 Lingua 语音翻译系统。";
    let request_body = serde_json::json!({
        "text": test_text,
        "voice": "zh_CN-huayan-medium"
    });
    println!("  文本: {}", test_text);
    println!("  语音: zh_CN-huayan-medium");
    println!("  端点: http://127.0.0.1:5005/tts");

    // 发送 TTS 请求
    println!("\n[3/4] 发送 TTS 请求并生成音频...");
    let start_time = std::time::Instant::now();
    
    let response = client
        .post("http://127.0.0.1:5005/tts")
        .json(&request_body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("[ERROR] HTTP 请求失败: {} {}", status, error_text);
        return Err(format!("HTTP request failed: {}", status).into());
    }
    
    let audio_data = response.bytes().await?.to_vec();
    let elapsed = start_time.elapsed();
    
    println!("[OK] 音频生成成功");
    println!("  耗时: {:?}", elapsed);
    println!("  音频大小: {} 字节", audio_data.len());

    if audio_data.is_empty() {
        eprintln!("[ERROR] 音频数据为空");
        return Err("Empty audio data".into());
    }

    // 验证 WAV 格式
    if audio_data.len() >= 4 {
        let header = String::from_utf8_lossy(&audio_data[0..4]);
        if header == "RIFF" {
            println!("  格式: WAV (RIFF)");
        } else {
            println!("  警告: 文件头不是 RIFF，可能不是有效的 WAV 文件");
        }
    }

    // 保存到文件（使用项目根目录）
    println!("\n[4/4] 保存音频文件...");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| manifest_dir.as_path());
    let output_file = project_root.join("test_output").join("test_piper_rust.wav");
    
    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&output_file, &audio_data)?;
    println!("[OK] 音频文件已保存");
    println!("  文件路径: {}", output_file.display());
    println!("  文件大小: {} 字节", fs::metadata(&output_file)?.len());

    // 验证文件大小
    if audio_data.len() > 1024 {
        println!("[OK] 音频文件大小 > 1024 字节，符合预期");
    } else {
        println!("[WARN] 音频文件大小 <= 1024 字节，可能有问题");
    }

    println!("\n=== 测试完成 ===");
    println!("\n下一步：");
    println!("  1. 播放音频文件验证语音质量: {}", output_file.display());
    println!("  2. 如果正常，可以继续集成到 Rust 代码中");

    Ok(())
}

