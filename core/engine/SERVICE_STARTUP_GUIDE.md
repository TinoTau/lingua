# 服务启动指南

## 概述

Speaker Embedding 和 YourTTS 服务需要单独启动，支持 CPU 和 GPU 模式。

## 启动服务

### 方式 1：手动启动（推荐用于开发）

#### 启动 Speaker Embedding 服务

**CPU 模式**：
```bash
python core/engine/scripts/speaker_embedding_service.py
```

**GPU 模式**（如果可用）：
```bash
python core/engine/scripts/speaker_embedding_service.py --gpu
```

**自定义端口和地址**：
```bash
python core/engine/scripts/speaker_embedding_service.py --gpu --port 5003 --host 127.0.0.1
```

**默认配置**：
- 端口：5003
- 地址：127.0.0.1
- 端点：http://127.0.0.1:5003

#### 启动 YourTTS 服务

**CPU 模式**：
```bash
python core/engine/scripts/yourtts_service.py
```

**GPU 模式**（推荐，如果可用）：
```bash
python core/engine/scripts/yourtts_service.py --gpu
```

**自定义端口和地址**：
```bash
python core/engine/scripts/yourtts_service.py --gpu --port 5004 --host 127.0.0.1
```

**默认配置**：
- 端口：5004
- 地址：127.0.0.1
- 端点：http://127.0.0.1:5004

### 方式 2：使用启动脚本（Windows）

创建 `start_services.ps1`：

```powershell
# 启动 Speaker Embedding 服务（GPU 模式）
Start-Process python -ArgumentList "core/engine/scripts/speaker_embedding_service.py", "--gpu" -WindowStyle Normal

# 等待 5 秒
Start-Sleep -Seconds 5

# 启动 YourTTS 服务（GPU 模式）
Start-Process python -ArgumentList "core/engine/scripts/yourtts_service.py", "--gpu" -WindowStyle Normal

Write-Host "Services started. Check the windows for status."
```

运行：
```powershell
.\start_services.ps1
```

### 方式 3：使用启动脚本（Linux/Mac）

创建 `start_services.sh`：

```bash
#!/bin/bash

# 启动 Speaker Embedding 服务（GPU 模式）
python core/engine/scripts/speaker_embedding_service.py --gpu &
SPEAKER_EMBEDDING_PID=$!

# 等待 5 秒
sleep 5

# 启动 YourTTS 服务（GPU 模式）
python core/engine/scripts/yourtts_service.py --gpu &
YOURTTS_PID=$!

echo "Services started:"
echo "  Speaker Embedding: PID $SPEAKER_EMBEDDING_PID (port 5003)"
echo "  YourTTS: PID $YOURTTS_PID (port 5004)"
echo ""
echo "To stop services, run:"
echo "  kill $SPEAKER_EMBEDDING_PID $YOURTTS_PID"
```

运行：
```bash
chmod +x start_services.sh
./start_services.sh
```

## 验证服务运行

### 健康检查

**Speaker Embedding 服务**：
```bash
curl http://127.0.0.1:5003/health
```

**YourTTS 服务**：
```bash
curl http://127.0.0.1:5004/health
```

### 使用 Python 测试

```python
import requests

# 检查 Speaker Embedding 服务
response = requests.get("http://127.0.0.1:5003/health")
print("Speaker Embedding:", response.json())

# 检查 YourTTS 服务
response = requests.get("http://127.0.0.1:5004/health")
print("YourTTS:", response.json())
```

## GPU 模式检查

### 检查 CUDA 是否可用

```python
import torch
print(f"CUDA available: {torch.cuda.is_available()}")
if torch.cuda.is_available():
    print(f"CUDA device: {torch.cuda.get_device_name(0)}")
```

### 检查服务是否使用 GPU

启动服务时，如果 GPU 可用，会显示：
```
✅ Using GPU: NVIDIA GeForce RTX 3090
```

如果 GPU 不可用，会显示：
```
⚠️  GPU requested but not available, using CPU
ℹ️  Using CPU
```

## 常见问题

### 1. 端口被占用

**错误**：
```
OSError: [Errno 48] Address already in use
```

**解决**：
- 检查端口是否被占用：`netstat -an | grep 5003`（Linux/Mac）或 `netstat -an | findstr 5003`（Windows）
- 使用其他端口：`--port 5005`
- 停止占用端口的进程

### 2. 模型未找到

**错误**：
```
FileNotFoundError: Model not found at ...
```

**解决**：
- 确保模型文件在正确位置：
  - Speaker Embedding: `core/engine/models/speaker_embedding/cache`
  - YourTTS: `core/engine/models/tts/your_tts`
- 检查模型文件是否存在

### 3. GPU 不可用

**现象**：
- 服务启动但使用 CPU
- 性能较慢

**解决**：
- 检查 CUDA 是否安装：`nvidia-smi`
- 检查 PyTorch 是否支持 CUDA：`python -c "import torch; print(torch.cuda.is_available())"`
- 如果没有 GPU，使用 CPU 模式（不添加 `--gpu` 参数）

### 4. 内存不足

**错误**：
```
RuntimeError: CUDA out of memory
```

**解决**：
- 使用 CPU 模式：不添加 `--gpu` 参数
- 减少批处理大小
- 关闭其他占用 GPU 的程序

## 性能优化

### GPU 模式

- **Speaker Embedding**：GPU 模式可以提升 5-10 倍性能
- **YourTTS**：GPU 模式可以提升 10-20 倍性能

### CPU 模式

- 如果 GPU 不可用，服务会自动使用 CPU
- CPU 模式性能较慢，但功能完整

## 集成测试

运行集成测试前，确保服务已启动：

```bash
# 1. 启动服务（两个终端）
python core/engine/scripts/speaker_embedding_service.py --gpu
python core/engine/scripts/yourtts_service.py --gpu

# 2. 运行集成测试
cargo test --test speaker_services_integration_test -- --ignored
```

## 停止服务

### 手动停止

- 在运行服务的终端按 `Ctrl+C`

### 查找并停止进程

**Windows**：
```powershell
# 查找进程
Get-Process python | Where-Object {$_.CommandLine -like "*speaker_embedding*"}

# 停止进程
Stop-Process -Name python -Force
```

**Linux/Mac**：
```bash
# 查找进程
ps aux | grep speaker_embedding_service

# 停止进程
kill <PID>
```

