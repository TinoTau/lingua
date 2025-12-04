# 独立服务测试脚本

本目录包含用于独立测试各个服务的测试脚本，可以在 Windows 环境中运行。

## 测试脚本列表

### 1. VAD 测试 (`test_vad_standalone.rs`)

测试语音活动检测（VAD）服务。

**运行方法：**
```bash
cargo run --example test_vad_standalone
```

**测试内容：**
- TimeBasedVad：基于时间的 VAD（3秒间隔）
- SileroVad：基于 ONNX 的 VAD（如果模型可用）

**前提条件：**
- 无需额外服务
- SileroVad 需要模型文件：`core/engine/models/vad/silero/silero_vad.onnx`

---

### 2. ASR 测试 (`test_asr_standalone.rs`)

测试自动语音识别（ASR）服务（Whisper）。

**运行方法：**
```bash
cargo run --example test_asr_standalone
```

**测试内容：**
- 中文音频识别（`test_output/chinese.wav`）
- 英文音频识别（`test_output/english.wav`）

**前提条件：**
- Whisper 模型已下载到：`core/engine/models/asr/whisper/`
- 测试音频文件位于：`test_output/chinese.wav` 或 `test_output/english.wav`

---

### 3. NMT 测试 (`test_nmt_standalone.rs`)

测试神经机器翻译（NMT）服务（M2M100 HTTP）。

**运行方法：**
```bash
cargo run --example test_nmt_standalone
```

**测试内容：**
- 中文到英文翻译
- 英文到中文翻译
- 多语言翻译测试

**前提条件：**
- Python M2M100 NMT 服务已启动
- 服务地址：`http://127.0.0.1:5008`
- 健康检查端点：`http://127.0.0.1:5008/health`

**启动 NMT 服务：**
```bash
# 在 Windows PowerShell 中
cd services/nmt_m2m100
python nmt_service.py
```

---

### 4. TTS 测试 (`test_tts_standalone.rs`)

测试文本转语音（TTS）服务。

**运行方法：**
```bash
cargo run --example test_tts_standalone
```

**测试内容：**
- Piper HTTP TTS（如果服务可用）
- YourTTS HTTP（如果服务可用）
- 参考音频测试（音色克隆）

**前提条件：**

**Piper HTTP TTS：**
- 服务地址：`http://127.0.0.1:5005`
- 健康检查端点：`http://127.0.0.1:5005/health`
- 通常在 WSL2 中运行

**YourTTS HTTP：**
- 服务地址：`http://127.0.0.1:5004`
- 健康检查端点：`http://127.0.0.1:5004/health`
- 在 WSL2 中运行（Ubuntu 22.04）

**启动 TTS 服务：**

**Piper HTTP（WSL2）：**
```bash
# 在 WSL2 中
cd /mnt/d/Programs/github/lingua
bash scripts/wsl2_piper/start_piper_service.sh
```

**YourTTS HTTP（WSL2）：**
```bash
# 在 WSL2 中
cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate
python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
```

---

## 测试输出

所有测试脚本的输出文件保存在 `test_output/` 目录：

- `test_piper_1.wav`, `test_piper_2.wav` - Piper TTS 测试输出
- `test_yourtts_no_ref.wav` - YourTTS 无参考音频测试
- `test_yourtts_with_ref.wav` - YourTTS 参考音频测试（如果可用）

---

## 环境要求

### Windows 环境
- Rust 工具链
- Cargo
- 已编译的 `core_engine` 库

### 服务依赖
- **VAD**：无需额外服务（SileroVad 需要 ONNX 模型）
- **ASR**：Whisper 模型文件
- **NMT**：Python M2M100 HTTP 服务（Windows）
- **TTS**：Piper HTTP 或 YourTTS HTTP 服务（WSL2）

---

## 故障排除

### 服务连接失败

如果测试脚本报告服务不可用：

1. **检查服务是否运行：**
   ```bash
   # Windows PowerShell
   curl http://127.0.0.1:5008/health  # NMT
   curl http://127.0.0.1:5004/health  # YourTTS
   curl http://127.0.0.1:5005/health  # Piper
   ```

2. **检查端口是否被占用：**
   ```powershell
   netstat -ano | findstr :5004
   netstat -ano | findstr :5005
   netstat -ano | findstr :5008
   ```

3. **检查 WSL2 服务：**
   ```bash
   # 在 WSL2 中
   wsl -d "Ubuntu-22.04" -- bash -c "curl http://127.0.0.1:5004/health"
   ```

### 模型文件缺失

如果测试报告模型文件不存在：

1. **检查模型路径：**
   - ASR: `core/engine/models/asr/whisper/`
   - VAD: `core/engine/models/vad/silero/silero_vad.onnx`

2. **下载模型：**
   - 参考 `docs/models/` 目录中的模型下载指南

### 音频文件缺失

如果测试报告音频文件不存在：

1. **检查测试音频：**
   - `test_output/chinese.wav`
   - `test_output/english.wav`

2. **创建测试音频：**
   - 可以使用任何 WAV 格式的音频文件
   - 建议使用 16kHz, 16-bit, mono 格式

---

## 快速测试所有服务

```bash
# 1. 测试 VAD
cargo run --example test_vad_standalone

# 2. 测试 ASR（需要 Whisper 模型）
cargo run --example test_asr_standalone

# 3. 测试 NMT（需要 NMT 服务运行）
cargo run --example test_nmt_standalone

# 4. 测试 TTS（需要 TTS 服务运行）
cargo run --example test_tts_standalone
```

---

## 注意事项

1. **服务启动顺序：**
   - 先启动所需的服务（NMT、TTS）
   - 再运行测试脚本

2. **WSL2 服务：**
   - WSL2 中的服务需要设置 `--host 0.0.0.0` 以允许从 Windows 访问
   - Windows 客户端连接 `127.0.0.1` 即可（WSL2 自动端口映射）

3. **GPU 支持：**
   - 如果服务支持 GPU，使用 `--gpu` 参数启动
   - 测试脚本会自动检测服务是否使用 GPU

4. **测试时间：**
   - VAD 测试：< 1 秒
   - ASR 测试：取决于音频长度（通常 5-30 秒）
   - NMT 测试：< 5 秒
   - TTS 测试：取决于文本长度（通常 5-15 秒）

