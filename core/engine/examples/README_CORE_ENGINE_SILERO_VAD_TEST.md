# CoreEngine + SileroVad 集成测试

## 概述

此测试验证 CoreEngine 与 SileroVad 的集成，确保自然停顿检测功能正常工作。

## 前置条件

1. **NMT 服务**：运行在 `http://127.0.0.1:5008`
2. **TTS 服务**：运行在 `http://127.0.0.1:5005`
3. **Silero VAD 模型**：位于 `models/vad/silero/silero_vad.onnx`

## 启动依赖服务

### 方法 1：使用一键启动脚本（推荐）

```powershell
# 在项目根目录执行
.\start_all_services_with_speaker.ps1
```

这将启动所有必要的服务，包括：
- NMT 服务（端口 5008）
- TTS 服务（端口 5005）
- CoreEngine（端口 9000）

### 方法 2：手动启动服务

#### 启动 NMT 服务

```powershell
cd core\engine\scripts
.\start_nmt.ps1
```

#### 启动 TTS 服务

```powershell
# 在 WSL 中执行
wsl -d Ubuntu-22.04 bash -c 'cd /mnt/d/Programs/github/lingua && source venv/bin/activate && python3 core/engine/scripts/piper_tts_service.py --gpu --host 0.0.0.0 --port 5005'
```

## 运行测试

```powershell
cd core\engine
cargo run --example test_core_engine_silero_vad
```

## 测试内容

1. **服务健康检查**：验证 NMT 和 TTS 服务是否可用
2. **SileroVad 初始化**：加载模型并初始化
3. **CoreEngine 初始化**：使用 SileroVad 构建 CoreEngine
4. **自然停顿检测**：
   - 发送语音帧序列（模拟说话）
   - 发送静音帧序列（模拟停顿）
   - 验证是否检测到自然停顿边界

## 预期结果

测试应该显示：
- ✅ 所有服务健康检查通过
- ✅ SileroVad 初始化成功
- ✅ CoreEngine 初始化成功
- ✅ 检测到自然停顿（边界类型为 `NaturalPause`）

## 故障排除

### NMT 服务不可用

```
❌ NMT 服务不可用: http://127.0.0.1:5008
```

**解决方案**：
1. 检查 NMT 服务是否已启动
2. 验证端口 5008 是否被占用
3. 检查防火墙设置

### TTS 服务不可用

```
❌ TTS 服务不可用: http://127.0.0.1:5005
```

**解决方案**：
1. 检查 TTS 服务是否在 WSL 中运行
2. 验证端口转发是否正常
3. 检查 WSL 网络配置

### Silero VAD 模型文件不存在

```
❌ Silero VAD 模型文件不存在
```

**解决方案**：
1. 确保模型文件位于 `core/engine/models/vad/silero/silero_vad.onnx`
2. 如果不存在，请下载模型：
   ```powershell
   # 参考 ONNX_RUNTIME_VERSION_FIX.md 中的下载说明
   ```

### 自然停顿未检测到

```
⚠️  自然停顿检测: 未触发
```

**可能原因**：
1. 静音帧数量不足（需要至少 600ms 的静音）
2. 阈值配置不合适

**解决方案**：
1. 增加 `min_silence_duration_ms` 配置值
2. 调整 `silence_threshold` 配置值
3. 检查音频帧的采样率和格式

## 配置说明

测试使用的 SileroVad 配置：
- **采样率**：16kHz
- **帧大小**：512 samples (32ms)
- **静音阈值**：0.5
- **最小静音时长**：600ms

这些配置可以在 `lingua_core_config.toml` 中修改：

```toml
[vad]
type = "silero"
model_path = "models/vad/silero/silero_vad.onnx"
silence_threshold = 0.5
min_silence_duration_ms = 600
```

## 下一步

测试通过后，可以在实际应用中使用 CoreEngine，它将自动使用 SileroVad 进行自然停顿检测。

