# WSL2 Piper TTS 部署和使用指南

本目录包含在 Windows 上通过 WSL2 部署和使用 Piper TTS 的所有脚本和说明。

## 快速开始

### 1. 环境准备（Windows 侧）

以**管理员权限**运行 PowerShell，执行：

```powershell
cd D:\Programs\github\lingua
.\scripts\wsl2_piper\setup_wsl2.ps1
```

此脚本会：
- 检查并安装 WSL2（如果未安装）
- 检查并安装 Ubuntu（如果未安装）
- 验证 WSL2 运行状态

**注意**：如果首次安装 WSL2，需要重启电脑后再次运行此脚本。

### 2. 安装 Piper（WSL2 内）

在 WSL2 Ubuntu 终端中执行：

```bash
cd /mnt/d/Programs/github/lingua
bash scripts/wsl2_piper/install_piper_in_wsl.sh
```

或在 Windows PowerShell 中通过 WSL 执行：

```powershell
wsl bash scripts/wsl2_piper/install_piper_in_wsl.sh
```

此脚本会：
- 更新系统并安装 Python3、pip、git 等依赖
- 创建虚拟环境 `~/piper_env`
- 安装 `piper-tts[http]` 包

### 3. 下载中文模型（WSL2 内）

在 WSL2 Ubuntu 终端中执行：

```bash
bash scripts/wsl2_piper/download_piper_model.sh
```

或在 Windows PowerShell 中通过 WSL 执行：

```powershell
wsl bash scripts/wsl2_piper/download_piper_model.sh
```

此脚本会：
- 创建模型目录 `~/piper_models/zh`
- 下载 `zh_CN-huayan-medium.onnx` 模型文件
- 下载对应的配置文件

### 4. 启动 HTTP 服务（WSL2 内）

在 WSL2 Ubuntu 终端中执行：

```bash
bash scripts/wsl2_piper/start_piper_service.sh
```

或在 Windows PowerShell 中通过 WSL 执行：

```powershell
wsl bash scripts/wsl2_piper/start_piper_service.sh
```

此脚本会：
- 检查并安装 FastAPI、Uvicorn 等 HTTP 服务依赖
- 激活虚拟环境
- 启动自定义的 Piper HTTP 服务包装器，监听 `0.0.0.0:5005`
- 服务将持续运行，直到按 `Ctrl+C` 停止

**注意**：由于 `piper-tts` 包不包含 HTTP 服务器，我们使用自定义的 Python HTTP 服务包装器（`piper_http_server.py`）来调用 `piper` 命令行工具。

**注意**：服务启动后，请保持终端窗口打开。

### 5. 测试服务（Windows 侧）

在 Windows PowerShell 中执行：

```powershell
.\scripts\wsl2_piper\test_piper_service.ps1
```

此脚本会：
- 检查服务是否运行
- 发送测试请求到 `http://127.0.0.1:5005/tts`
- 生成测试音频文件 `test_output\piper_wsl2_test.wav`
- 验证音频文件是否正常生成

### 6. 在 Rust 代码中使用

在 `CoreEngineBuilder` 中使用 Piper HTTP TTS：

```rust
use core_engine::bootstrap::CoreEngineBuilder;
use core_engine::tts_streaming::PiperHttpConfig;

let engine = CoreEngineBuilder::new()
    // ... 其他配置 ...
    .tts_with_default_piper_http()?  // 使用默认配置
    // 或
    .tts_with_piper_http(PiperHttpConfig {
        endpoint: "http://127.0.0.1:5005/tts".to_string(),
        default_voice: "zh_CN-huayan-medium".to_string(),
        timeout_ms: 8000,
    })?
    .build()?;
```

## 文件说明

- `setup_wsl2.ps1` - Windows 侧 WSL2 环境检查和安装脚本
- `install_piper_in_wsl.sh` - WSL2 内 Piper 安装脚本
- `download_piper_model.sh` - WSL2 内模型下载脚本
- `piper_http_server.py` - Piper HTTP 服务包装器（Python FastAPI 实现）
- `start_piper_service.sh` - WSL2 内 HTTP 服务启动脚本
- `test_piper_service.ps1` - Windows 侧服务测试脚本
- `README.md` - 本文件

## 常见问题

### Q: WSL2 安装后需要重启吗？
A: 是的，首次安装 WSL2 后需要重启电脑。

### Q: 如何检查 WSL2 是否运行？
A: 在 PowerShell 中执行 `wsl -l -v`，查看 VERSION 是否为 2。

### Q: 如何停止 Piper HTTP 服务？
A: 在运行服务的终端窗口中按 `Ctrl+C`。

### Q: 服务启动失败怎么办？
A: 
1. 检查虚拟环境是否正确安装：`wsl bash -c "cd ~/piper_env && source .venv/bin/activate && pip list | grep piper"`
2. 检查模型文件是否存在：`wsl bash -c "ls -lh ~/piper_models/zh/"`
3. 查看服务启动日志中的错误信息

### Q: 如何更换其他中文模型？
A: 
1. 修改 `download_piper_model.sh` 中的 `MODEL_NAME` 和 `BASE_URL`
2. 重新运行下载脚本
3. 修改 `start_piper_service.sh` 中的 `DEFAULT_VOICE`
4. 重启服务

### Q: 如何修改服务端口？
A: 修改 `start_piper_service.sh` 中的 `PORT` 变量，并相应更新 Rust 代码中的 `endpoint` 配置。

## 参考文档

- [WSL2_Piper_ZH_TTS_Deployment_Guide.md](../../docs/architecture/WSL2_Piper_ZH_TTS_Deployment_Guide.md) - 完整部署指南
- [Piper 官方仓库](https://github.com/OHF-Voice/piper1-gpl)
- [Piper Python 包](https://pypi.org/project/piper-tts/)
- [Piper 声音样例](https://rhasspy.github.io/piper-samples/)

