# 完整 S2S 流测试命令

## 快速开始

### 方法 1: 使用 PowerShell 脚本（推荐）

```powershell
# 在项目根目录
.\scripts\test_s2s_full_real.ps1 <input_wav_file>
```

**示例**:
```powershell
.\scripts\test_s2s_full_real.ps1 test_input\chinese_audio.wav
```

### 方法 2: 直接使用 Cargo 命令

```powershell
# 切换到 core/engine 目录
cd core\engine

# 运行测试
cargo run --example test_s2s_full_real -- <input_wav_file>
```

**示例**:
```powershell
cd core\engine
cargo run --example test_s2s_full_real -- ..\test_input\chinese_audio.wav
```

## 前提条件检查

### 1. 检查 Piper HTTP 服务

```powershell
# 在 PowerShell 中
Invoke-WebRequest -Uri "http://127.0.0.1:5005/health"
```

如果服务未运行，在 WSL2 中启动：

```bash
# 在 WSL2 中
cd /mnt/d/Programs/github/lingua
bash scripts/wsl2_piper/start_piper_service.sh
```

### 2. 检查模型文件

```powershell
# 检查 Whisper ASR 模型
Test-Path core\engine\models\asr\whisper-base

# 检查 Marian NMT 模型
Test-Path core\engine\models\nmt\marian-zh-en
```

### 3. 准备测试音频文件

准备一个中文语音的 WAV 文件：
- 格式：WAV
- 采样率：建议 16kHz（会自动处理）
- 声道：单声道或立体声（会自动转换）

## 完整测试流程

### 步骤 1: 启动 Piper HTTP 服务（如果未运行）

```bash
# 在 WSL2 中
cd /mnt/d/Programs/github/lingua
bash scripts/wsl2_piper/start_piper_service.sh
```

### 步骤 2: 运行测试

```powershell
# 在 PowerShell 中（项目根目录）
.\scripts\test_s2s_full_real.ps1 test_input\chinese_audio.wav
```

### 步骤 3: 查看结果

测试完成后，检查输出文件：

```powershell
# 查看输出文件
Get-Item test_output\s2s_full_real_test.wav | Select-Object FullName, Length
```

## 测试输出

测试成功后会输出：

1. **源文本（中文）**: ASR 识别的结果
2. **目标文本（英文）**: NMT 翻译的结果
3. **音频文件**: `test_output/s2s_full_real_test.wav`

## 故障排除

### 错误: "Service not available"

**解决**: 启动 Piper HTTP 服务（见步骤 1）

### 错误: "Whisper ASR model directory not found"

**解决**: 下载 Whisper 模型到 `core/engine/models/asr/whisper-base/`

### 错误: "Marian NMT model directory not found"

**解决**: 导出 Marian NMT 模型到 `core/engine/models/nmt/marian-zh-en/`

### 错误: "Input file not found"

**解决**: 检查输入文件路径是否正确

## 示例命令序列

```powershell
# 1. 检查服务
Invoke-WebRequest -Uri "http://127.0.0.1:5005/health"

# 2. 运行测试
.\scripts\test_s2s_full_real.ps1 test_input\chinese_audio.wav

# 3. 查看结果
Get-Item test_output\s2s_full_real_test.wav
```

## 注意事项

1. **音频文件路径**: 可以使用相对路径或绝对路径
2. **WSL2 服务**: 确保 WSL2 中的 Piper 服务正在运行
3. **模型文件**: 确保所有必需的模型文件都已下载
4. **输出目录**: 输出文件会保存到 `test_output/` 目录

