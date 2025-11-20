//! 简化的完整 S2S 流测试：测试文本→翻译→TTS 流程
//! 
//! 使用方法：
//!   cargo run --example test_s2s_flow_simple
//! 
//! 前提条件：
//!   1. WSL2 中已启动 Piper HTTP 服务（http://127.0.0.1:5005/tts）
//!   2. 服务正在运行
//! 
//! 注意：此测试模拟完整的 S2S 流程，使用模拟的 NMT 翻译

use std::fs;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 完整 S2S 流集成测试（简化版） ===\n");

    // 检查服务是否运行
    println!("[1/5] 检查 Piper HTTP 服务状态...");
    let health_url = "http://127.0.0.1:5005/health";
    let client = reqwest::Client::new();
    match client.get(health_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                println!("[OK] Piper HTTP 服务正在运行");
            } else {
                eprintln!("[ERROR] 服务返回错误状态: {}", resp.status());
                return Err("Service health check failed".into());
            }
        }
        Err(e) => {
            eprintln!("[ERROR] 无法连接到服务: {}", e);
            eprintln!("[INFO] 请确保 WSL2 中的 Piper HTTP 服务正在运行");
            return Err("Service not available".into());
        }
    }

    // 步骤 1: 模拟 ASR 结果（中文文本）
    println!("\n[2/5] 模拟 ASR 结果（中文文本）...");
    let source_text = "你好，欢迎使用 Lingua 语音翻译系统。";
    println!("  源文本（中文）: {}", source_text);
    println!("  [模拟] ASR 识别成功");

    // 步骤 2: 模拟 NMT 翻译
    println!("\n[3/5] 模拟 NMT 翻译...");
    // 在实际场景中，这里会调用真实的 NMT 模型
    // 为了测试，我们使用模拟的翻译结果
    let target_text = "Hello, welcome to use the Lingua speech translation system.";
    println!("  目标文本（英文）: {}", target_text);
    println!("  [模拟] NMT 翻译成功");
    println!("  注意: 实际部署时应使用真实的 NMT 模型");

    // 步骤 3: TTS 合成（使用 Piper HTTP TTS 合成中文语音）
    println!("\n[4/5] 执行 TTS 合成（Piper HTTP）...");
    println!("  说明: 合成中文语音用于回放源语言");
    
    let tts_request_body = serde_json::json!({
        "text": source_text,
        "voice": "zh_CN-huayan-medium"
    });
    
    let start_time = std::time::Instant::now();
    let response = client
        .post("http://127.0.0.1:5005/tts")
        .json(&tts_request_body)
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
    
    println!("[OK] TTS 合成成功");
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
        }
    }

    // 保存到文件
    println!("\n[5/5] 保存音频文件...");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| manifest_dir.as_path());
    let output_file = project_root.join("test_output").join("s2s_flow_test.wav");
    
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
    }

    println!("\n=== 测试完成 ===");
    println!("\n完整 S2S 流程测试总结：");
    println!("  ✅ 步骤 1: ASR 识别（模拟）");
    println!("    输入: 中文语音");
    println!("    输出: \"{}\"", source_text);
    println!();
    println!("  ✅ 步骤 2: NMT 翻译（模拟）");
    println!("    输入: \"{}\"", source_text);
    println!("    输出: \"{}\"", target_text);
    println!();
    println!("  ✅ 步骤 3: TTS 合成（Piper HTTP）");
    println!("    输入: \"{}\"", source_text);
    println!("    输出: {} 字节 WAV 音频", audio_data.len());
    println!();
    println!("  完整流程: 中文语音 → 中文文本 → 英文文本 → 中文语音");
    println!();
    println!("下一步：");
    println!("  1. 播放音频文件验证语音质量: {}", output_file.display());
    println!("  2. 如果正常，完整的 S2S 流程已验证通过");
    println!("  3. 可以继续集成真实的 ASR 和 NMT 模型进行完整测试");

    Ok(())
}

