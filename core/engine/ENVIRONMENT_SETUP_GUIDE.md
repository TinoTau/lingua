# 环境配置完整指南

## 概述

本指南将帮助您：
1. **Windows 环境**：创建 Python 3.10 的 conda 环境，安装 Speaker Embedding 服务依赖
2. **WSL 环境**：安装 Ubuntu 22.04，配置 Python 3.10/3.11，安装 TTS 服务依赖
3. **验证**：确保所有服务都能正常运行

---

## 第一部分：Windows 环境配置

### 步骤 1：创建新的 conda 环境（Python 3.10）

```powershell
# 1. 打开 Anaconda Prompt 或 PowerShell

# 2. 创建新环境（如果已存在，先删除）
conda env remove -n lingua-py310 -y
conda create -n lingua-py310 python=3.10 -y

# 3. 激活新环境
conda activate lingua-py310

# 4. 验证 Python 版本
python --version
# 应该显示：Python 3.10.x
```

### 步骤 2：安装 PyTorch（GPU 版）

```powershell
# 确保在 lingua-py310 环境里
conda activate lingua-py310

# 安装 PyTorch + CUDA 12.1（根据你的 CUDA 版本调整）
conda install pytorch pytorch-cuda=12.1 -c pytorch -c nvidia -y

# 或者使用 pip 安装（如果 conda 有问题）
# pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
```

### 步骤 3：安装 Speaker Embedding 服务依赖

```powershell
# 继续在 lingua-py310 环境里
conda activate lingua-py310

# 基础依赖
pip install numpy soundfile flask

# torchaudio（如果 conda 没装）
pip install torchaudio

# Speaker Embedding 依赖
pip install speechbrain
```

### 步骤 4：验证 Windows 环境

```powershell
# 在 lingua-py310 环境里
conda activate lingua-py310

# 验证 PyTorch GPU
python -c "import torch; print('PyTorch:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('CUDA version:', torch.version.cuda if torch.cuda.is_available() else 'N/A'); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"

# 验证 SpeechBrain
python -c "import speechbrain; print('SpeechBrain:', speechbrain.__version__)"

# 测试 Speaker Embedding 服务启动（不运行，只检查导入）
python -c "from core.engine.scripts.speaker_embedding_service import *; print('✅ Speaker Embedding service imports OK')"
```

---

## 第二部分：WSL 环境配置（Ubuntu 22.04）

### 步骤 1：安装 Ubuntu 22.04（如果还没有）

#### 选项 A：安装新的 Ubuntu 22.04 发行版（推荐）

```powershell
# 在 Windows PowerShell 中
# 1. 查看可用的 WSL 发行版
wsl --list --online

# 2. 安装 Ubuntu 22.04
wsl --install -d Ubuntu-22.04

# 3. 设置用户名和密码（首次启动时会提示）
```

#### 选项 B：将现有 Ubuntu 降级（复杂，不推荐）

如果必须降级现有 Ubuntu，需要：
1. 导出当前环境
2. 卸载现有发行版
3. 安装 Ubuntu 22.04
4. 恢复数据

**建议直接安装新的 Ubuntu 22.04 发行版**。

### 步骤 2：在 WSL 中配置 Python 环境

```bash
# 1. 进入 WSL
wsl -d Ubuntu-22.04

# 2. 更新系统包
sudo apt update
sudo apt upgrade -y

# 3. 安装 Python 3.10 和基础工具
sudo apt install -y python3.10 python3.10-venv python3.10-dev python3-pip

# 4. 设置 python3 指向 3.10（如果默认不是）
sudo update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.10 1

# 5. 验证 Python 版本
python3 --version
# 应该显示：Python 3.10.x
```

### 步骤 3：创建虚拟环境

```bash
# 1. 进入项目目录（在 WSL 中）
cd /mnt/d/Programs/github/lingua

# 2. 创建虚拟环境
python3.10 -m venv venv-wsl

# 3. 激活虚拟环境
source venv-wsl/bin/activate

# 4. 升级 pip
pip install --upgrade pip
```

### 步骤 4：安装 TTS 服务依赖

```bash
# 确保在虚拟环境中
source venv-wsl/bin/activate

# 基础依赖
pip install numpy soundfile flask

# PyTorch（GPU 版，CUDA 12.1）
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121

# ONNX（如果需要）
pip install onnx onnxruntime

# TTS 库（YourTTS）
pip install TTS

# Piper TTS 依赖（如果使用）
pip install fastapi uvicorn pydantic
```

### 步骤 5：验证 WSL 环境

```bash
# 在 venv-wsl 环境里
source venv-wsl/bin/activate

# 验证 PyTorch GPU
python3 -c "import torch; print('PyTorch:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('CUDA version:', torch.version.cuda if torch.cuda.is_available() else 'N/A'); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"

# 验证 TTS
python3 -c "from TTS.api import TTS; print('✅ TTS library OK')"

# 验证服务脚本
python3 -c "import sys; sys.path.insert(0, '/mnt/d/Programs/github/lingua'); from core.engine.scripts.yourtts_service import *; print('✅ YourTTS service imports OK')"
```

---

## 第三部分：更新启动脚本

### Windows 启动脚本更新

更新 `core/engine/scripts/start_all_services.ps1`，确保使用正确的 conda 环境：

```powershell
# 在脚本开头添加
conda activate lingua-py310
```

### WSL 启动脚本更新

更新 `core/engine/scripts/start_yourtts_wsl.sh`，确保使用正确的虚拟环境：

```bash
# 在脚本开头添加
source /mnt/d/Programs/github/lingua/venv-wsl/bin/activate
```

---

## 第四部分：验证完整流程

### 1. 启动 Windows 服务（Speaker Embedding）

```powershell
# 在 PowerShell 中
conda activate lingua-py310
cd D:\Programs\github\lingua
python core\engine\scripts\speaker_embedding_service.py --gpu
```

**预期输出**：
```
✅ Using GPU: <你的显卡名称>
✅ Speaker Embedding model loaded successfully
🚀 Starting server on http://127.0.0.1:5003
```

### 2. 启动 WSL 服务（YourTTS）

```bash
# 在 WSL 中
source /mnt/d/Programs/github/lingua/venv-wsl/bin/activate
cd /mnt/d/Programs/github/lingua
python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
```

**预期输出**：
```
✅ Using GPU: <你的显卡名称>
✅ YourTTS model loaded successfully
🚀 Starting server on http://0.0.0.0:5004
```

### 3. 健康检查

```powershell
# 在 Windows PowerShell 中
# Speaker Embedding
curl http://127.0.0.1:5003/health

# YourTTS（通过 WSL 端口映射）
curl http://127.0.0.1:5004/health
```

---

## 故障排除

### Windows 环境问题

#### 问题 1：conda 环境创建失败

```powershell
# 清理 conda 缓存
conda clean --all

# 重新创建
conda create -n lingua-py310 python=3.10 -y
```

#### 问题 2：PyTorch GPU 不可用

```powershell
# 检查 CUDA 驱动
nvidia-smi

# 检查 PyTorch 安装
python -c "import torch; print(torch.cuda.is_available())"

# 如果不可用，重新安装
conda install pytorch pytorch-cuda=12.1 -c pytorch -c nvidia -y
```

### WSL 环境问题

#### 问题 1：Python 3.10 安装失败

```bash
# 添加 deadsnakes PPA（如果需要）
sudo add-apt-repository ppa:deadsnakes/ppa
sudo apt update
sudo apt install python3.10 python3.10-venv python3.10-dev
```

#### 问题 2：TTS 库安装失败

```bash
# 确保 Python 版本正确
python3 --version  # 应该是 3.10.x

# 清理 pip 缓存
pip cache purge

# 重新安装
pip install TTS
```

#### 问题 3：WSL GPU 不可用

```bash
# 检查 WSL GPU 支持
wsl nvidia-smi

# 如果不可用，检查 Windows 驱动和 WSL 版本
# 需要 WSL 2 + NVIDIA 驱动 510+ 或更高
```

---

## 快速参考

### Windows 环境激活

```powershell
conda activate lingua-py310
```

### WSL 环境激活

```bash
source /mnt/d/Programs/github/lingua/venv-wsl/bin/activate
```

### 服务启动命令

**Windows（Speaker Embedding）**：
```powershell
conda activate lingua-py310
python core\engine\scripts\speaker_embedding_service.py --gpu
```

**WSL（YourTTS）**：
```bash
source /mnt/d/Programs/github/lingua/venv-wsl/bin/activate
python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
```

---

## 依赖清单总结

### Windows 环境（lingua-py310）

- Python 3.10
- PyTorch (GPU, CUDA 12.1)
- torchaudio
- numpy
- soundfile
- flask
- speechbrain

### WSL 环境（venv-wsl）

- Python 3.10
- PyTorch (GPU, CUDA 12.1)
- torchaudio
- numpy
- soundfile
- flask
- TTS
- onnx, onnxruntime（可选）
- fastapi, uvicorn, pydantic（Piper TTS）

---

## 完成检查清单

- [ ] Windows conda 环境 `lingua-py310` 创建成功
- [ ] Windows PyTorch GPU 可用
- [ ] Windows Speaker Embedding 服务能启动
- [ ] WSL Ubuntu 22.04 安装成功
- [ ] WSL Python 3.10 可用
- [ ] WSL 虚拟环境 `venv-wsl` 创建成功
- [ ] WSL PyTorch GPU 可用
- [ ] WSL TTS 库安装成功
- [ ] WSL YourTTS 服务能启动
- [ ] 两个服务的健康检查都通过

完成以上所有步骤后，您的环境就配置完成了！

