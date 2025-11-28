# Lingua Core Runtime 一键启动与服务设计说明

**最后更新**: 2025-11-28

## 1. 概述
Lingua Core Runtime 是提供端到端实时语音翻译（Speech-to-Speech, S2S）的核心服务集群。
各前端形态（Chrome 插件、Electron、移动端、PWA）均通过统一 API 与核心服务通信，实现真正的"一套核心，多种壳"。

**GPU 加速支持**：
- ✅ ASR (Whisper) - CUDA GPU 加速已启用
- ✅ NMT (M2M100) - PyTorch CUDA GPU 加速已启用
- ❌ TTS - 暂未启用 GPU 加速

## 2. 核心服务组成

### 2.1 CoreEngine Service（Rust）
- **职责**：
  - Whisper ASR（内置，支持 CUDA GPU 加速）
  - 音频分段、停顿检测
  - 调用 NMT 服务执行翻译
  - 调用 TTS 服务生成语音
  - 对外暴露统一 S2S API
- **GPU 支持**：✅ CUDA 12.4（已启用）
- **端口**：9000
- **接口**：
  - POST /s2s
  - WS /stream
  - GET /health

### 2.2 NMT Service（Python + M2M100）
- **职责**：提供翻译能力，支持 en↔zh 等语言对
- **GPU 支持**：✅ PyTorch CUDA 12.1（已启用）
- **端口**：5008
- **可替换**：可替换为线上翻译服务（已预留接口）
- **接口**：
  - POST /v1/translate
  - GET /health

### 2.3 TTS Service（Piper HTTP）
- **职责**：提供语音合成能力
- **GPU 支持**：❌ 暂未启用
- **端口**：5005
- **接口**：
  - POST /tts
  - GET /health

## 3. 配置文件：lingua_core_config.toml

```toml
[nmt]
url = "http://127.0.0.1:5008"

[tts]
url = "http://127.0.0.1:5005/tts"

[engine]
port = 9000
whisper_model_path = "models/asr/whisper-base"
```

## 4. 一键启动和停止脚本

### 4.1 Windows 一键启动（推荐）

项目提供了两个启动脚本，推荐使用**多窗口模式**，方便查看各服务的日志：

#### 方法 1：多窗口模式（推荐）⭐

```powershell
.\start_all_services_simple.ps1
```

**特点**：
- 每个服务在独立的 PowerShell 窗口中运行
- 可以实时查看每个服务的日志输出
- 方便调试和监控
- 关闭服务：直接关闭对应的窗口，或运行 `.\stop_all_services.ps1`

#### 方法 2：单窗口模式

```powershell
.\start_all_services.ps1
```

**特点**：
- NMT 服务在后台运行
- CoreEngine 在前台运行
- 按 `Ctrl+C` 会自动停止所有服务

### 4.2 一键停止所有服务

```powershell
.\stop_all_services.ps1
```

**功能**：
- 停止端口 9000 上的 CoreEngine
- 停止端口 5008 上的 NMT 服务
- 停止端口 5005 上的 TTS 服务（如果在运行）
- 停止所有 PowerShell 后台作业
- 停止 WSL 中的 TTS 服务（如果存在）
- 验证所有端口是否已释放

### 4.3 服务启动流程

1. **设置 CUDA 环境变量**（自动）
   - 检测 CUDA Toolkit 12.4 安装路径
   - 设置必要的环境变量供 CoreEngine 使用

2. **启动 NMT 服务**
   - 激活 Python 虚拟环境
   - 启动 FastAPI 服务（端口 5008）
   - 自动检测并使用 GPU（如果 PyTorch CUDA 已安装）

3. **启动 CoreEngine**
   - 加载 Whisper ASR 模型（GPU 加速）
   - 启动 HTTP 服务器（端口 9000）
   - 等待 NMT 和 TTS 服务就绪

### 4.4 验证服务状态

#### 检查服务是否运行

```powershell
# 检查端口占用情况
Get-NetTCPConnection -LocalPort 9000,5008,5005 -ErrorAction SilentlyContinue

# 检查服务健康状态
Invoke-WebRequest -Uri "http://127.0.0.1:5008/health"  # NMT 服务
Invoke-WebRequest -Uri "http://0.0.0.0:9000/health"    # CoreEngine
```

#### 验证 GPU 使用

**NMT 服务窗口**应该显示：
```
[NMT Service] Device: cuda
[NMT Service] ✓ CUDA available: True
[NMT Service] ✓ GPU name: NVIDIA GeForce RTX 4060 Laptop GPU
```

**CoreEngine 窗口**应该显示：
```
whisper_init_with_params_no_state: use gpu    = 1
ggml_cuda_init: found 1 CUDA devices: Device 0: NVIDIA GeForce RTX 4060 Laptop GPU
register_backend: registered backend CUDA (1 devices)
```

#### 使用 nvidia-smi 监控 GPU

```powershell
# 实时监控 GPU 使用情况
nvidia-smi -l 1
```

发送请求后，应该看到 GPU 使用率上升。

### 4.5 Linux / macOS 启动脚本（参考）

```bash
#!/bin/bash

# 启动 NMT 服务
cd services/nmt_m2m100
source venv/bin/activate
nohup uvicorn nmt_service:app --host 127.0.0.1 --port 5008 &

# 启动 CoreEngine
cd ../..
export CUDA_PATH=/usr/local/cuda
./core/engine/target/release/core_engine --config lingua_core_config.toml
```

## 5. 对前端暴露的 API

### 5.1 整句翻译（同步 S2S）
POST /s2s

输入：
```json
{
  "audio": "<base64>",
  "src_lang": "zh",
  "tgt_lang": "en"
}
```

### 5.2 流式实时翻译（WebSocket）
- WS /stream/start
- WS /stream/audio
- WS /stream/output
- WS /stream/stop

## 6. 前端（壳）接入方式
所有壳无需关心 Whisper/NMT/TTS 模型，只需：
- 采集麦克风音频
- 推送到 CoreEngine
- 播放返回音频
- 显示字幕（可选）

## 7. 项目结构

```
/lingua
  /core
    /engine                    # CoreEngine (Rust)
      /target
        /release
          core_engine.exe      # 编译后的可执行文件
      Cargo.toml               # 包含 CUDA feature 配置
  /services
    /nmt_m2m100               # NMT 服务 (Python)
      nmt_service.py          # FastAPI 服务
      /venv                   # Python 虚拟环境
      requirements.txt        # 包含 PyTorch CUDA
  /docs
    /operational              # 操作文档
      ASR_GPU_配置完成.md
      PyTorch_CUDA_安装指南.md
      GPU改造进度总结.md
  start_all_services.ps1      # 单窗口启动脚本
  start_all_services_simple.ps1  # 多窗口启动脚本（推荐）
  stop_all_services.ps1       # 一键停止脚本
  lingua_core_config.toml     # 配置文件
```

## 8. GPU 配置要求

### 8.1 ASR (Whisper) GPU 支持

**前置条件**：
- ✅ CUDA Toolkit 12.4 已安装
- ✅ Visual Studio 2022 Community 已安装
- ✅ CUDA 工具集已配置
- ✅ `Cargo.toml` 已配置 CUDA feature

**验证**：启动 CoreEngine 后，日志应显示 `use gpu = 1`

**参考文档**：`docs/operational/ASR_GPU_配置完成.md`

### 8.2 NMT (M2M100) GPU 支持

**前置条件**：
- ✅ PyTorch CUDA 版本已安装（2.5.1+cu121）
- ✅ CUDA Toolkit 12.4 已安装
- ✅ 虚拟环境中已安装 CUDA 版本的 PyTorch

**验证**：启动 NMT 服务后，日志应显示 `Device: cuda`

**参考文档**：`docs/operational/PyTorch_CUDA_安装指南.md`

### 8.3 性能预期

| 组件 | CPU 耗时 | GPU 耗时 | 提升倍数 |
|------|---------|---------|---------|
| ASR | 6-7 秒 | 1-2 秒 | 3-4x |
| NMT | 3-4 秒 | 0.5-1 秒 | 3-4x |
| **总计** | **13-15 秒** | **4.5-7 秒** | **2-3x** |

## 9. 故障排查

### 9.1 服务启动失败

1. **检查端口占用**：
   ```powershell
   Get-NetTCPConnection -LocalPort 9000,5008,5005
   ```

2. **检查 CUDA 环境**：
   ```powershell
   nvcc --version
   $env:CUDA_PATH
   ```

3. **检查 PyTorch CUDA**：
   ```powershell
   cd services\nmt_m2m100
   .\venv\Scripts\Activate.ps1
   python -c "import torch; print(torch.cuda.is_available())"
   ```

### 9.2 GPU 未启用

- **ASR**：参考 `docs/operational/ASR_GPU_编译故障排查.md`
- **NMT**：检查 PyTorch CUDA 安装，参考 `docs/operational/PyTorch_CUDA_安装指南.md`

### 9.3 相关文档

- `docs/operational/GPU改造进度总结.md` - GPU 配置总体进度
- `docs/operational/ASR_GPU_配置完成.md` - ASR GPU 配置详细步骤
- `docs/operational/PyTorch_CUDA_安装指南.md` - NMT GPU 配置详细步骤
- `docs/operational/ASR_GPU_编译故障排查.md` - 常见问题解决方案
