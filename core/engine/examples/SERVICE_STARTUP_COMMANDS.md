# 各服务启动命令

本文档提供所有服务的启动命令，按环境分类。

---

## Windows 环境服务

### 1. CoreEngine 服务（包含 ASR/Whisper）

**端口：** 9000  
**类型：** Rust HTTP 服务器（内置 Whisper ASR）  
**GPU：** 支持（CUDA 12.4）

#### 前提条件

需要先编译 CoreEngine：

```powershell
cd core\engine
cargo build --release --bin core_engine
```

**注意**：CUDA GPU 支持已在 `Cargo.toml` 中默认启用（通过 `whisper-rs` 依赖），无需额外 feature 参数。

#### 启动命令（PowerShell）

```powershell
# 使用启动脚本（推荐）
.\core\engine\scripts\start_core_engine.ps1
```

#### 手动启动

```powershell
# 确保在项目根目录
cd D:\Programs\github\lingua

# 启动 CoreEngine
.\core\engine\target\release\core_engine.exe --config lingua_core_config.toml
```

#### 验证服务

```powershell
# 健康检查
curl http://127.0.0.1:9000/health
```

**注意：** ASR（Whisper）是内置在 CoreEngine 中的，不需要单独启动 ASR 服务。

---

### 2. ASR 服务（Whisper HTTP Server，可选 - 不推荐）

**端口：** 8080（默认）  
**类型：** whisper.cpp HTTP 服务器  
**GPU：** 支持

#### 前提条件

需要先编译 whisper.cpp：

```powershell
cd third_party\whisper.cpp
mkdir build
cd build
cmake ..
cmake --build . --config Release
```

#### 启动命令（PowerShell）

```powershell
# 使用启动脚本
.\core\engine\scripts\start_whisper_server.ps1
```

#### 手动启动

```powershell
# 进入 whisper.cpp 目录
cd third_party\whisper.cpp

# 启动服务器（需要先编译）
.\build\bin\whisper-server.exe --model models\ggml-base.bin --host 127.0.0.1 --port 8080 --language auto
```

#### 验证服务

```powershell
# 健康检查（whisper.cpp server 没有 /health 端点，但可以访问根路径）
curl http://127.0.0.1:8080/
```

**注意：** 如果使用 Rust 的 `WhisperAsrStreaming`（直接集成），则不需要启动此服务。

---

### 3. Speaker Embedding 服务

**端口：** 5003  
**环境：** Windows conda 环境（lingua-py310）  
**GPU：** 支持

#### 启动命令（PowerShell）

```powershell
# 激活 conda 环境
conda activate lingua-py310

# 如果 conda 命令不可用，使用完整路径
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" core\engine\scripts\speaker_embedding_service.py --gpu
```

#### 验证服务

```powershell
curl http://127.0.0.1:5003/health
```

---

### 4. NMT 服务（M2M100）

**端口：** 5008  
**环境：** Windows Python 虚拟环境（services/nmt_m2m100/venv）  
**GPU：** 自动检测

#### 启动命令（PowerShell）

```powershell
# 进入服务目录
cd services\nmt_m2m100

# 激活虚拟环境
.\venv\Scripts\Activate.ps1

# 启动服务
uvicorn nmt_service:app --host 127.0.0.1 --port 5008
```

#### 或者使用 Python 直接运行

```powershell
cd services\nmt_m2m100
.\venv\Scripts\python.exe nmt_service.py
```

#### 验证服务

```powershell
curl http://127.0.0.1:5008/health
```

---

## WSL2 环境服务

### 5. YourTTS 服务

**端口：** 5004  
**环境：** WSL2 Ubuntu 22.04 虚拟环境（venv-wsl）  
**GPU：** 支持

#### 启动命令（在 WSL2 中）

```bash
# 进入项目目录
cd /mnt/d/Programs/github/lingua

# 激活虚拟环境
source venv-wsl/bin/activate

# 启动服务（GPU 模式，允许从 Windows 访问）
python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0 --port 5004
```

#### 从 Windows 启动（PowerShell）

```powershell
# 使用启动脚本
wsl -d "Ubuntu-22.04" -- bash -c "cd /mnt/d/Programs/github/lingua && source venv-wsl-py310/bin/activate && python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0 --port 5004"
```

#### 验证服务

```powershell
# 在 Windows PowerShell 中
curl http://127.0.0.1:5004/health
```

---

### 6. Piper TTS 服务

---

## Web 前端服务

### 7. Web 前端服务器

**端口：** 8080  
**类型：** 静态文件服务器（PWA）  
**环境：** Windows（Python 或 Node.js）

#### 启动命令（PowerShell）

```powershell
# 使用启动脚本（推荐）
cd clients\web_pwa
.\start_web_server.ps1
```

#### 或者使用原始脚本

```powershell
cd clients\web_pwa
.\start_server.ps1 -Port 8080
```

#### 手动启动

**使用 Python：**
```powershell
cd clients\web_pwa
python -m http.server 8080 --directory .
```

**使用 Node.js：**
```powershell
cd clients\web_pwa
npx http-server . -p 8080
```

#### 验证服务

```powershell
# 在浏览器中访问
start http://localhost:8080
```

**注意：** Web 前端需要 CoreEngine 服务（端口 9000）运行才能正常工作。

**端口：** 5005  
**环境：** WSL2（piper_env）  
**GPU：** 支持

#### 启动命令（在 WSL2 中）

```bash
# 使用启动脚本
bash scripts/wsl2_piper/start_piper_service.sh
```

#### 手动启动

```bash
# 进入 piper 环境目录
cd ~/piper_env

# 激活虚拟环境
source .venv/bin/activate

# 设置环境变量
export PIPER_MODEL_DIR="$HOME/piper_models"
export PIPER_DEFAULT_VOICE="zh_CN-huayan-medium"
export PIPER_USE_GPU=true

# 启动服务
python scripts/wsl2_piper/piper_http_server.py --host 0.0.0.0 --port 5005 --model-dir "$PIPER_MODEL_DIR"
```

#### 验证服务

```powershell
# 在 Windows PowerShell 中
curl http://127.0.0.1:5005/health
```

---

## 快速启动所有服务

### Windows PowerShell 脚本

创建 `start_all_services.ps1`：

```powershell
# 启动 Speaker Embedding 服务
Write-Host "Starting Speaker Embedding service..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "conda activate lingua-py310; python core\engine\scripts\speaker_embedding_service.py --gpu"

# 等待服务启动
Start-Sleep -Seconds 3

# 启动 NMT 服务
Write-Host "Starting NMT service..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd services\nmt_m2m100; .\venv\Scripts\Activate.ps1; uvicorn nmt_service:app --host 127.0.0.1 --port 5008"

# 等待服务启动
Start-Sleep -Seconds 3

# 启动 YourTTS 服务（WSL2）
Write-Host "Starting YourTTS service in WSL2..." -ForegroundColor Yellow
wsl -d "Ubuntu-22.04" -- bash -c "cd /mnt/d/Programs/github/lingua && source venv-wsl-py310/bin/activate && python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0 --port 5004" &

# 等待服务启动
Start-Sleep -Seconds 5

# 启动 Piper TTS 服务（WSL2）
Write-Host "Starting Piper TTS service in WSL2..." -ForegroundColor Yellow
wsl -d "Ubuntu-22.04" -- bash -c "cd /mnt/d/Programs/github/lingua && bash scripts/wsl2_piper/start_piper_service.sh" &

Write-Host "`n✅ All services started!" -ForegroundColor Green
Write-Host "   Speaker Embedding: http://127.0.0.1:5003" -ForegroundColor Cyan
Write-Host "   NMT: http://127.0.0.1:5008" -ForegroundColor Cyan
Write-Host "   YourTTS: http://127.0.0.1:5004" -ForegroundColor Cyan
Write-Host "   Piper TTS: http://127.0.0.1:5005" -ForegroundColor Cyan
```

### 验证所有服务

```powershell
# 检查所有服务
Write-Host "Checking services..." -ForegroundColor Cyan
curl http://127.0.0.1:5003/health  # Speaker Embedding
curl http://127.0.0.1:5004/health  # YourTTS
curl http://127.0.0.1:5005/health  # Piper TTS
curl http://127.0.0.1:5008/health  # NMT
```

---

## 服务端口总结

| 服务 | 端口 | 环境 | GPU 支持 | 必需 |
|------|------|------|----------|------|
| **CoreEngine (ASR内置)** | **9000** | **Windows (Rust)** | **✅** | **推荐** |
| **Web Frontend** | **8080** | **Windows (Python/Node)** | **-** | **推荐** |
| ASR (Whisper HTTP) | 8080 | Windows (编译) | ✅ | 可选* |
| Speaker Embedding | 5003 | Windows (conda) | ✅ | 可选 |
| YourTTS | 5004 | WSL2 (venv) | ✅ | 可选 |
| Piper TTS | 5005 | WSL2 (piper_env) | ✅ | 可选 |
| NMT (M2M100) | 5008 | Windows (venv) | 自动检测 | 可选 |

\* **推荐使用 CoreEngine**，它内置了 ASR（Whisper），不需要单独的 ASR HTTP 服务。

\* **推荐使用 CoreEngine**，它内置了 ASR（Whisper），不需要单独的 ASR HTTP 服务。

---

## 注意事项

### 1. 服务启动顺序

建议按以下顺序启动：
1. Speaker Embedding（Windows）
2. NMT（Windows）
3. YourTTS（WSL2）
4. Piper TTS（WSL2）

### 2. WSL2 服务访问

- WSL2 中的服务需要设置 `--host 0.0.0.0` 以允许从 Windows 访问
- Windows 客户端连接 `127.0.0.1` 即可（WSL2 自动端口映射）

### 3. GPU 支持

- 所有服务都支持 GPU 加速
- 使用 `--gpu` 参数启用（如果可用）
- 如果 GPU 不可用，服务会自动回退到 CPU

### 4. 环境激活

- **Windows conda 环境：** `conda activate lingua-py310`
- **Windows venv：** `.\venv\Scripts\Activate.ps1`
- **WSL2 venv：** `source venv-wsl/bin/activate`

### 5. 停止服务

- 在服务运行的终端中按 `Ctrl+C`
- 或关闭服务窗口（如果使用 `Start-Process` 启动）

---

## 故障排除

### 端口被占用

```powershell
# 检查端口占用
netstat -ano | findstr :5003
netstat -ano | findstr :5004
netstat -ano | findstr :5005
netstat -ano | findstr :5008

# 终止进程（替换 PID）
taskkill /PID <PID> /F
```

### 服务无法访问

1. **检查服务是否运行：**
   ```powershell
   curl http://127.0.0.1:5003/health
   ```

2. **检查防火墙设置：**
   - Windows 防火墙可能阻止本地服务
   - WSL2 服务需要确保 `--host 0.0.0.0`

3. **检查 WSL2 网络：**
   ```bash
   # 在 WSL2 中
   curl http://127.0.0.1:5004/health
   ```

### 环境问题

1. **conda 命令不可用：**
   - 使用完整 Python 路径：`& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe"`

2. **虚拟环境未激活：**
   - Windows: `.\venv\Scripts\Activate.ps1`
   - WSL2: `source venv-wsl/bin/activate`

3. **依赖缺失：**
   - 检查 `requirements.txt` 并安装依赖
   - 确保所有模型文件已下载

---

## 测试服务

启动所有服务后，运行测试脚本：

```bash
# 测试 VAD（无需服务）
cargo run --example test_vad_standalone

# 测试 ASR（无需服务，需要模型）
cargo run --example test_asr_standalone

# 测试 NMT（需要服务运行）
cargo run --example test_nmt_standalone

# 测试 TTS（需要服务运行）
cargo run --example test_tts_standalone
```

