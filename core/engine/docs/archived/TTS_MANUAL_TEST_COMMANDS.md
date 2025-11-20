# TTS 模块手动测试命令

**目的**: 逐步测试 TTS 模块的各个功能

---

## 📋 测试前准备

### 1. 确认编译通过

```powershell
cd D:\Programs\github\lingua\core\engine
cargo check --lib
```

**预期结果**: `Finished dev [unoptimized + debuginfo] target(s)`

---

## 🔍 测试步骤

### 步骤 1: 测试模型加载

```powershell
cd D:\Programs\github\lingua\core\engine
cargo test --lib test_tts_model_load -- --nocapture
```

**预期结果**:
- 如果模型文件存在: `✅ FastSpeech2TtsEngine loaded successfully`
- 如果模型文件不存在: `Skipping test: TTS model directory not found`

**如果卡住**: 可能是模型文件路径问题，检查 `models/tts/` 目录是否存在

---

### 步骤 2: 测试文本预处理器

```powershell
cd D:\Programs\github\lingua\core\engine
cargo test --lib test_text_processor_load -- --nocapture
```

**预期结果**:
- `✅ Chinese TextProcessor loaded successfully`
- `✅ English TextProcessor loaded successfully`
- `Phone map size: XXX`

**如果失败**: 检查 `models/tts/fastspeech2-lite/phone_id_map.txt` 是否存在

---

### 步骤 3: 测试文本规范化

```powershell
cd D:\Programs\github\lingua\core\engine
cargo test --lib test_text_normalization -- --nocapture
```

**预期结果**:
- 显示多个文本规范化结果
- 每个测试用例都应该通过

---

### 步骤 4: 测试音素 ID 映射

```powershell
cd D:\Programs\github\lingua\core\engine
cargo test --lib test_phoneme_to_id_mapping -- --nocapture
```

**预期结果**:
- 显示多个音素到 ID 的映射结果
- 每个音素都应该有对应的 ID

---

### 步骤 5: 测试中文 TTS 合成（需要模型文件）

```powershell
cd D:\Programs\github\lingua\core\engine
cargo test --lib test_tts_synthesize_chinese -- --nocapture
```

**预期结果**:
- 如果模型文件存在: `✅ TTS synthesis successful` + 音频长度信息
- 如果模型文件不存在: `Skipping test: TTS model directory not found`

**如果失败**: 
- 检查模型文件是否存在
- 检查文本预处理是否成功
- 检查 ONNX 推理是否成功

---

### 步骤 6: 测试英文 TTS 合成（需要模型文件）

```powershell
cd D:\Programs\github\lingua\core\engine
cargo test --lib test_tts_synthesize_english -- --nocapture
```

**预期结果**: 同步骤 5

---

### 步骤 7: 测试空文本处理

```powershell
cd D:\Programs\github\lingua\core\engine
cargo test --lib test_tts_empty_text -- --nocapture
```

**预期结果**:
- `✅ Empty text handled correctly`
- 返回空音频 chunk

---

## 🔧 如果测试卡住

### 方案 1: 使用超时运行单个测试

```powershell
# 设置超时（PowerShell 7+）
$job = Start-Job -ScriptBlock { 
    Set-Location D:\Programs\github\lingua\core\engine
    cargo test --lib test_tts_model_load -- --nocapture
}
if (Wait-Job $job -Timeout 30) {
    Receive-Job $job
} else {
    Write-Host "Test timeout after 30 seconds"
    Stop-Job $job
}
Remove-Job $job
```

### 方案 2: 直接运行测试二进制文件

```powershell
cd D:\Programs\github\lingua\core\engine
cargo build --tests
.\target\debug\deps\core_engine-*.exe test_tts_model_load --nocapture
```

### 方案 3: 创建最小测试脚本

创建一个简单的 Rust 测试文件，只测试最基本的功能：

```rust
// tests/tts_simple_test.rs
#[test]
fn test_tts_stub() {
    use core_engine::tts_streaming::{TtsStub, TtsRequest, TtsStreaming};
    use tokio::runtime::Runtime;
    
    let rt = Runtime::new().unwrap();
    let stub = TtsStub::new();
    let request = TtsRequest {
        text: "test".to_string(),
        voice: "default".to_string(),
        locale: "en".to_string(),
    };
    
    let result = rt.block_on(stub.synthesize(request));
    assert!(result.is_ok());
    println!("✅ TtsStub test passed");
}
```

然后运行：
```powershell
cargo test --test tts_simple_test -- --nocapture
```

---

## 📊 测试结果记录表

| 测试步骤 | 命令 | 结果 | 耗时 | 备注 |
|---------|------|------|------|------|
| 1. 模型加载 | `test_tts_model_load` | ✅/❌ | ___ 秒 | |
| 2. 文本预处理器 | `test_text_processor_load` | ✅/❌ | ___ 秒 | |
| 3. 文本规范化 | `test_text_normalization` | ✅/❌ | ___ 秒 | |
| 4. 音素映射 | `test_phoneme_to_id_mapping` | ✅/❌ | ___ 秒 | |
| 5. 中文 TTS | `test_tts_synthesize_chinese` | ✅/❌ | ___ 秒 | |
| 6. 英文 TTS | `test_tts_synthesize_english` | ✅/❌ | ___ 秒 | |
| 7. 空文本 | `test_tts_empty_text` | ✅/❌ | ___ 秒 | |

---

## 🎯 快速测试流程

### 最小测试（不依赖模型文件）

```powershell
# 1. 测试 TtsStub（不依赖模型）
cargo test --lib tts_stub -- --nocapture

# 2. 测试 TextProcessor（只需要 phone_id_map.txt）
cargo test --lib test_text_processor_load -- --nocapture
```

### 完整测试（需要所有模型文件）

```powershell
# 运行所有 TTS 测试
cargo test --lib tts -- --nocapture

# 或逐个运行
cargo test --lib test_tts_model_load -- --nocapture
cargo test --lib test_tts_synthesize_chinese -- --nocapture
cargo test --lib test_tts_synthesize_english -- --nocapture
```

---

## 🚨 常见问题

### 问题 1: 测试卡住

**可能原因**:
- 模型文件很大，加载需要时间
- ONNX Runtime 初始化需要时间
- 防病毒软件扫描

**解决方案**:
- 使用超时运行测试
- 先测试不依赖模型的测试（TtsStub）
- 检查模型文件是否存在

### 问题 2: 模型文件不存在

**检查方法**:
```powershell
Test-Path D:\Programs\github\lingua\core\engine\models\tts\fastspeech2-lite\fastspeech2_csmsc_streaming.onnx
Test-Path D:\Programs\github\lingua\core\engine\models\tts\hifigan-lite\hifigan_csmsc.onnx
```

**解决方案**:
- 下载模型文件
- 或跳过需要模型的测试

### 问题 3: 文本预处理失败

**可能原因**:
- `phone_id_map.txt` 格式不正确
- 文本规范化逻辑有问题

**解决方案**:
- 检查 `phone_id_map.txt` 格式
- 查看测试输出中的错误信息

---

**最后更新**: 2024-12-19

