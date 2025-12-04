# Windows 环境配置完成 ✅

## 安装状态

✅ **所有依赖已安装成功**：
- numpy
- soundfile
- flask
- torch (2.5.1+cu121)
- torchaudio
- speechbrain

## 下一步：验证和测试

### 1. 验证 GPU 可用性

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -c "import torch; print('PyTorch:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('CUDA version:', torch.version.cuda if torch.cuda.is_available() else 'N/A'); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"
```

**预期输出**（如果 GPU 可用）：
```
PyTorch: 2.5.1+cu121
CUDA available: True
CUDA version: 12.1
GPU: NVIDIA GeForce RTX xxxx
```

### 2. 测试 Speaker Embedding 服务启动

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" core\engine\scripts\speaker_embedding_service.py --gpu
```

**预期输出**：
```
✅ Using GPU: <你的显卡名称>
✅ Speaker Embedding model loaded successfully
🚀 Starting server on http://127.0.0.1:5003
```

### 3. 健康检查（在另一个 PowerShell 窗口）

```powershell
curl http://127.0.0.1:5003/health
```

**预期输出**：
```json
{"status":"healthy","model_loaded":true}
```

## 简化命令（创建别名）

为了后续使用方便，可以在 PowerShell 中创建函数：

```powershell
# 创建 pip 函数
function pip {
    & "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip $args
}

# 创建 python 函数
function python {
    & "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" $args
}

# 现在可以直接使用
pip list
python --version
python core\engine\scripts\speaker_embedding_service.py --gpu
```

## 日常使用

### 启动服务

```powershell
# 方式 1：使用完整路径
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" core\engine\scripts\speaker_embedding_service.py --gpu

# 方式 2：使用别名（如果创建了）
python core\engine\scripts\speaker_embedding_service.py --gpu
```

### 停止服务

在服务运行的窗口中按 `Ctrl + C`

## 关于 conda activate

**不需要使用 `conda activate`**，因为：
- conda 命令有问题（base 环境损坏）
- 环境已经配置好，可以直接使用完整路径
- 或者使用 Anaconda Prompt（通常更稳定）

## 下一步

1. ✅ Windows 环境配置完成
2. ⏭️ 配置 WSL 环境（YourTTS 服务）
3. ⏭️ 测试完整服务链

参考 `VIRTUAL_ENVIRONMENT_SETUP.md` 的 **第二部分：WSL 环境配置**。

