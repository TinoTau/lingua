# TTS 性能问题诊断

## 问题

TTS 耗时 14100ms，明显过慢。

## 可能的原因

### 1. 每次调用都重新加载模型（最可能）

当前实现使用 `subprocess` 调用 `piper` 命令行工具，每次请求都会：
- 启动新的 piper 进程
- 重新加载模型（耗时）
- 执行推理
- 退出进程

**解决方案**：使用 Python API 直接调用，保持模型常驻内存。

### 2. GPU 未真正启用

虽然传递了 `--cuda` 参数，但需要验证：
- Piper 命令行工具是否支持 `--cuda`
- ONNX Runtime 是否真的在使用 GPU
- 检查 Piper 服务的 stderr 输出

### 3. 网络开销

虽然在同一台机器，但通过 WSL2 端口转发可能有延迟。

## 诊断步骤

### 步骤 1：检查 Piper 服务日志

查看 TTS 服务窗口的日志，应该看到：
```
Using GPU acceleration (--cuda)
Executing piper command: ...
```

### 步骤 2：检查是否每次都在加载模型

如果每次请求都看到模型加载信息，说明模型没有常驻内存。

### 步骤 3：检查 GPU 使用情况

在另一个终端运行：
```bash
watch -n 0.5 nvidia-smi
```

然后发送 TTS 请求，观察 GPU 使用率是否上升。

## 解决方案

### 方案 1：使用 Python API（推荐）

修改 `piper_http_server.py`，使用 `PiperVoice` Python API 而不是命令行工具，这样可以：
- 模型只加载一次
- 保持在内存中
- 每次请求只执行推理

### 方案 2：验证 GPU 是否启用

检查 Piper 命令行工具是否真的支持 `--cuda` 参数，以及是否正确使用 GPU。

