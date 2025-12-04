# 虚拟环境安装操作手册

## 概述

本手册将指导您创建两个独立的虚拟环境：
1. **Windows 环境**：conda 环境 `lingua-py310`（用于 Speaker Embedding 服务）
2. **WSL 环境**：Python venv `venv-wsl`（用于 YourTTS 服务）

---

## 第一部分：Windows 环境（conda）

### 前置要求

- ✅ 已安装 Anaconda 或 Miniconda
- ✅ Windows 10/11 x64
- ✅ NVIDIA GPU 驱动（如果使用 GPU）

### 步骤 1：检查 conda 是否可用

打开 **Anaconda Prompt** 或 **PowerShell**，运行：

```powershell
conda --version
```

**预期输出**：`conda 23.x.x` 或类似版本号

如果提示命令不存在，请先安装 [Anaconda](https://www.anaconda.com/download) 或 [Miniconda](https://docs.conda.io/en/latest/miniconda.html)。

### 步骤 2：创建 conda 环境

```powershell
# 创建名为 lingua-py310 的环境，Python 版本 3.10
conda create -n lingua-py310 python=3.10 -y
```

**预期输出**：
```
Collecting package metadata (current_repodata.json): done
Solving environment: done
...
Preparing transaction: done
Verifying transaction: done
Executing transaction: done
#
# To activate this environment, use
#
#     $ conda activate lingua-py310
```

### 步骤 3：激活环境

```powershell
conda activate lingua-py310
```

**预期输出**：提示符变为 `(lingua-py310) PS D:\...>`

### 步骤 4：验证 Python 版本

```powershell
python --version
```

**预期输出**：`Python 3.10.x`

### 步骤 5：安装 PyTorch（GPU 版）

#### 方式 A：使用 conda（推荐）

```powershell
# 确保在 lingua-py310 环境中
conda activate lingua-py310

# 安装 PyTorch + CUDA 12.1
conda install pytorch pytorch-cuda=12.1 -c pytorch -c nvidia -y
```

#### 方式 B：使用 pip（如果 conda 安装失败）

```powershell
conda activate lingua-py310

# 安装 PyTorch + CUDA 12.1
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
```

**安装时间**：可能需要 10-30 分钟（取决于网络速度）

### 步骤 6：安装其他依赖

```powershell
# 确保在 lingua-py310 环境中
conda activate lingua-py310

# 安装基础依赖
pip install numpy soundfile flask

# 安装 torchaudio（如果 conda 没装）
pip install torchaudio

# 安装 SpeechBrain
pip install speechbrain
```

### 步骤 7：验证安装

```powershell
conda activate lingua-py310

# 验证 PyTorch 和 CUDA
python -c "import torch; print('PyTorch 版本:', torch.__version__); print('CUDA 可用:', torch.cuda.is_available()); print('CUDA 版本:', torch.version.cuda if torch.cuda.is_available() else 'N/A'); print('GPU 名称:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"
```

**预期输出**（如果 GPU 可用）：
```
PyTorch 版本: 2.x.x+cu121
CUDA 可用: True
CUDA 版本: 12.1
GPU 名称: NVIDIA GeForce RTX xxxx
```

**预期输出**（如果 GPU 不可用）：
```
PyTorch 版本: 2.x.x+cu121
CUDA 可用: False
CUDA 版本: N/A
GPU 名称: N/A
```

### 步骤 8：验证 SpeechBrain

```powershell
conda activate lingua-py310

python -c "import speechbrain; print('SpeechBrain 版本:', speechbrain.__version__)"
```

**预期输出**：`SpeechBrain 版本: 0.x.x`

### 步骤 9：测试服务启动

```powershell
conda activate lingua-py310

# 进入项目目录
cd D:\Programs\github\lingua

# 测试服务启动（按 Ctrl+C 停止）
python core\engine\scripts\speaker_embedding_service.py --gpu
```

**预期输出**：
```
✅ Using GPU: <你的显卡名称>
✅ Speaker Embedding model loaded successfully
🚀 Starting server on http://127.0.0.1:5003
```

---

## 第二部分：WSL 环境（Python venv）

### 前置要求

- ✅ 已安装 WSL 2
- ✅ 已安装 Ubuntu 22.04（推荐）或 Ubuntu 20.04
- ✅ NVIDIA GPU 驱动（如果使用 GPU）

### 步骤 1：进入 WSL

在 **Windows PowerShell** 中运行：

```powershell
wsl
```

或在 **WSL 终端**中直接操作。

### 步骤 2：检查 Python 版本

```bash
python3 --version
```

**预期输出**：
- Ubuntu 22.04：`Python 3.10.x` ✅
- Ubuntu 20.04：`Python 3.8.x`（需要安装 3.10）
- Ubuntu 24.04：`Python 3.12.x`（需要安装 3.10）

### 步骤 3：安装 Python 3.10（如果需要）

如果系统默认不是 Python 3.10：

```bash
# 更新包列表
sudo apt update

# 安装 Python 3.10
sudo apt install -y python3.10 python3.10-venv python3.10-dev python3-pip

# 验证安装
python3.10 --version
```

**预期输出**：`Python 3.10.x`

### 步骤 4：进入项目目录

```bash
# 进入项目目录（WSL 路径）
cd /mnt/d/Programs/github/lingua

# 验证目录
pwd
```

**预期输出**：`/mnt/d/Programs/github/lingua`

### 步骤 5：创建虚拟环境

```bash
# 使用 Python 3.10 创建虚拟环境
python3.10 -m venv venv-wsl

# 如果系统默认是 3.10，也可以直接使用
# python3 -m venv venv-wsl
```

**预期输出**：无错误，创建 `venv-wsl` 目录

### 步骤 6：激活虚拟环境

```bash
source venv-wsl/bin/activate
```

**预期输出**：提示符变为 `(venv-wsl) tinot@Tino-Lenovo:/mnt/d/Programs/github/lingua$`

### 步骤 7：升级 pip

```bash
# 确保在虚拟环境中
source venv-wsl/bin/activate

# 升级 pip
pip install --upgrade pip
```

**预期输出**：`Successfully installed pip-x.x.x`

### 步骤 8：安装 PyTorch（GPU 版）

```bash
# 确保在虚拟环境中
source venv-wsl/bin/activate

# 安装 PyTorch + CUDA 12.1
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
```

**安装时间**：可能需要 10-30 分钟（取决于网络速度）

### 步骤 9：安装其他依赖

```bash
# 确保在虚拟环境中
source venv-wsl/bin/activate

# 安装基础依赖
pip install numpy soundfile flask

# 安装 ONNX（可选，用于模型导出）
pip install onnx onnxruntime

# 安装 TTS 库（YourTTS）
pip install TTS

# 安装 Piper TTS 依赖（如果使用）
pip install fastapi uvicorn pydantic
```

### 步骤 10：验证安装

```bash
source venv-wsl/bin/activate

# 验证 PyTorch 和 CUDA
python3 -c "import torch; print('PyTorch 版本:', torch.__version__); print('CUDA 可用:', torch.cuda.is_available()); print('CUDA 版本:', torch.version.cuda if torch.cuda.is_available() else 'N/A'); print('GPU 名称:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"
```

**预期输出**（如果 GPU 可用）：
```
PyTorch 版本: 2.x.x+cu121
CUDA 可用: True
CUDA 版本: 12.1
GPU 名称: NVIDIA GeForce RTX xxxx
```

### 步骤 11：验证 TTS 库

```bash
source venv-wsl/bin/activate

python3 -c "from TTS.api import TTS; print('✅ TTS 库安装成功')"
```

**预期输出**：`✅ TTS 库安装成功`

### 步骤 12：测试服务启动

```bash
source venv-wsl/bin/activate

# 测试 YourTTS 服务启动（按 Ctrl+C 停止）
python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
```

**预期输出**：
```
✅ Using GPU: <你的显卡名称>
✅ YourTTS model loaded successfully
🚀 Starting server on http://0.0.0.0:5004
```

---

## 第三部分：使用自动化脚本（可选）

### Windows 环境自动化脚本

```powershell
# 在 PowerShell 中运行
.\core\engine\scripts\setup_windows_env.ps1
```

脚本会自动完成：
- 创建 conda 环境
- 安装 PyTorch
- 安装所有依赖
- 验证安装

### WSL 环境自动化脚本

```bash
# 在 WSL 中运行
bash core/engine/scripts/setup_wsl_env.sh
```

脚本会自动完成：
- 检查 Python 版本
- 创建虚拟环境
- 安装所有依赖
- 验证安装

---

## 第四部分：日常使用

### Windows 环境激活

每次使用前：

```powershell
# 激活 conda 环境
conda activate lingua-py310

# 运行服务
python core\engine\scripts\speaker_embedding_service.py --gpu
```

### WSL 环境激活

每次使用前：

```bash
# 进入项目目录
cd /mnt/d/Programs/github/lingua

# 激活虚拟环境
source venv-wsl/bin/activate

# 运行服务
python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0
```

### 退出环境

**Windows（conda）**：
```powershell
conda deactivate
```

**WSL（venv）**：
```bash
deactivate
```

---

## 第五部分：故障排除

### Windows 环境问题

#### 问题 1：conda 命令不存在

**解决**：
1. 安装 [Anaconda](https://www.anaconda.com/download) 或 [Miniconda](https://docs.conda.io/en/latest/miniconda.html)
2. 重启 PowerShell
3. 或使用 Anaconda Prompt

#### 问题 2：环境创建失败

**解决**：
```powershell
# 清理 conda 缓存
conda clean --all

# 重新创建环境
conda create -n lingua-py310 python=3.10 -y
```

#### 问题 3：PyTorch GPU 不可用

**解决**：
1. 检查 NVIDIA 驱动：`nvidia-smi`
2. 检查 CUDA 版本：`nvcc --version`
3. 重新安装匹配的 PyTorch 版本

#### 问题 4：SpeechBrain 安装失败

**解决**：
```powershell
conda activate lingua-py310

# 先安装依赖
pip install torch torchaudio

# 再安装 SpeechBrain
pip install speechbrain
```

### WSL 环境问题

#### 问题 1：Python 3.10 安装失败

**解决**：
```bash
# 添加 deadsnakes PPA（Ubuntu 20.04）
sudo add-apt-repository ppa:deadsnakes/ppa
sudo apt update
sudo apt install python3.10 python3.10-venv python3.10-dev
```

#### 问题 2：TTS 库安装失败

**解决**：
```bash
# 确保 Python 版本正确
python3 --version  # 应该是 3.10.x

# 清理 pip 缓存
pip cache purge

# 重新安装
pip install TTS
```

#### 问题 3：WSL GPU 不可用

**解决**：
1. 检查 WSL GPU 支持：`wsl nvidia-smi`
2. 需要 WSL 2 + NVIDIA 驱动 510+ 或更高
3. 安装 NVIDIA Container Toolkit（如果使用 Docker）

#### 问题 4：虚拟环境激活失败

**解决**：
```bash
# 检查虚拟环境是否存在
ls -la venv-wsl

# 重新创建虚拟环境
rm -rf venv-wsl
python3.10 -m venv venv-wsl
source venv-wsl/bin/activate
```

---

## 第六部分：验证清单

### Windows 环境检查

- [ ] conda 环境 `lingua-py310` 创建成功
- [ ] Python 版本为 3.10.x
- [ ] PyTorch 安装成功
- [ ] CUDA 可用（如果使用 GPU）
- [ ] SpeechBrain 安装成功
- [ ] Speaker Embedding 服务能启动

### WSL 环境检查

- [ ] 虚拟环境 `venv-wsl` 创建成功
- [ ] Python 版本为 3.10.x
- [ ] PyTorch 安装成功
- [ ] CUDA 可用（如果使用 GPU）
- [ ] TTS 库安装成功
- [ ] YourTTS 服务能启动

---

## 快速参考

### Windows 环境命令

```powershell
# 激活环境
conda activate lingua-py310

# 查看环境列表
conda env list

# 删除环境（如果需要）
conda env remove -n lingua-py310

# 导出环境配置
conda env export > lingua-py310.yaml
```

### WSL 环境命令

```bash
# 激活环境
source venv-wsl/bin/activate

# 查看已安装的包
pip list

# 导出依赖列表
pip freeze > requirements-wsl.txt

# 删除虚拟环境（如果需要）
rm -rf venv-wsl
```

---

## 完成！

完成以上所有步骤后，您的虚拟环境就配置完成了！

**下一步**：
- 参考 `SERVICE_STARTUP_GUIDE.md` 启动服务
- 参考 `ENVIRONMENT_SETUP_GUIDE.md` 进行完整的环境配置

