# WSL 集成指南

## 当前架构

### TTS 服务架构

1. **Piper TTS**：在 WSL2 中运行，通过 HTTP 提供服务
   - 端点：`http://127.0.0.1:5005/tts`
   - 客户端：`PiperHttpTts`（直接连接 localhost）

2. **YourTTS**：需要适配 WSL 架构
   - 当前配置：`http://127.0.0.1:5004`
   - 如果服务在 WSL 中运行，需要确保端口映射正确

### Speaker Embedding 服务

- 当前配置：`http://127.0.0.1:5003`
- 可以在 Windows 或 WSL 中运行
- 如果使用 GPU，建议在 Windows 中运行（GPU 支持更好）

## WSL 集成方案

### 方案 1：YourTTS 在 WSL 中运行（推荐，与 Piper 一致）

**优势**：
- 与现有 Piper TTS 架构一致
- Linux 环境对 Python 依赖更友好
- GPU 支持通过 WSL2 的 GPU 直通

**配置**：
- YourTTS 服务在 WSL 中运行
- 端口映射：WSL 端口 5004 → Windows localhost:5004
- Rust 客户端连接 `http://127.0.0.1:5004`

### 方案 2：YourTTS 在 Windows 中运行

**优势**：
- 直接使用 Windows GPU（如果可用）
- 不需要 WSL 配置

**配置**：
- YourTTS 服务在 Windows 中运行
- 直接监听 `127.0.0.1:5004`
- Rust 客户端连接 `http://127.0.0.1:5004`

## 推荐配置

### Speaker Embedding 服务
- **运行位置**：Windows（GPU 支持更好）
- **端口**：5003
- **端点**：`http://127.0.0.1:5003`

### YourTTS 服务
- **运行位置**：WSL2（与 Piper TTS 一致）
- **端口**：5004
- **端点**：`http://127.0.0.1:5004`（WSL 端口自动映射到 Windows）

## WSL 启动脚本

### 在 WSL 中启动 YourTTS 服务

创建 `core/engine/scripts/start_yourtts_wsl.sh`：

```bash
#!/bin/bash
# 在 WSL 中启动 YourTTS 服务

# 获取脚本目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"

# 切换到项目目录
cd "$PROJECT_ROOT"

# 启动 YourTTS 服务（GPU 模式）
python core/engine/scripts/yourtts_service.py --gpu --port 5004 --host 0.0.0.0

# 注意：host 设置为 0.0.0.0 以允许从 Windows 访问
```

### 从 Windows 启动 WSL 服务

创建 `core/engine/scripts/start_yourtts_wsl.ps1`：

```powershell
# 在 WSL 中启动 YourTTS 服务

Write-Host "Starting YourTTS service in WSL..." -ForegroundColor Green

# 获取项目根目录
$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

# 在 WSL 中启动服务
wsl bash -c "cd /mnt/d/Programs/github/lingua && python core/engine/scripts/yourtts_service.py --gpu --port 5004 --host 0.0.0.0"

# 注意：路径需要转换为 WSL 路径格式
```

## 端口映射

WSL2 会自动将 WSL 中的端口映射到 Windows localhost，所以：
- WSL 中监听 `0.0.0.0:5004` → Windows 可以通过 `127.0.0.1:5004` 访问
- 无需额外配置端口转发

## GPU 支持

### WSL2 GPU 支持

WSL2 支持 GPU 直通，YourTTS 在 WSL 中可以使用 GPU：

1. **检查 WSL GPU 支持**：
   ```bash
   wsl nvidia-smi
   ```

2. **在 WSL 中启动服务（GPU 模式）**：
   ```bash
   python core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
   ```

### Windows GPU 支持

如果 Speaker Embedding 在 Windows 中运行，直接使用 GPU：
```bash
python core/engine/scripts/speaker_embedding_service.py --gpu
```

## 验证配置

### 1. 检查服务运行

**Windows 中检查**：
```powershell
# 检查端口是否监听
netstat -an | findstr :5003  # Speaker Embedding
netstat -an | findstr :5004  # YourTTS
```

**WSL 中检查**：
```bash
# 检查端口是否监听
netstat -tuln | grep 5004
```

### 2. 健康检查

```bash
# 从 Windows 检查
curl http://127.0.0.1:5003/health  # Speaker Embedding
curl http://127.0.0.1:5004/health  # YourTTS

# 从 WSL 检查
curl http://localhost:5004/health  # YourTTS
```

## 完整启动流程

### 推荐配置

1. **Speaker Embedding 服务**（Windows，GPU）：
   ```powershell
   python core/engine/scripts/speaker_embedding_service.py --gpu
   ```

2. **YourTTS 服务**（WSL，GPU）：
   ```bash
   # 在 WSL 中
   python core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
   ```

3. **Piper TTS 服务**（WSL，已运行）：
   - 继续使用现有配置

### 统一启动脚本

创建 `start_all_services.ps1`：

```powershell
# 启动所有服务

Write-Host "Starting all services..." -ForegroundColor Green

# 1. Speaker Embedding (Windows, GPU)
Start-Process python -ArgumentList "core/engine/scripts/speaker_embedding_service.py", "--gpu" -WindowStyle Normal

# 2. YourTTS (WSL, GPU)
Start-Process wsl -ArgumentList "bash", "-c", "cd /mnt/d/Programs/github/lingua && python core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0" -WindowStyle Normal

Write-Host "Services started!" -ForegroundColor Green
```

