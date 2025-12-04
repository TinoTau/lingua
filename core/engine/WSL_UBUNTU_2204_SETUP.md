# Ubuntu 22.04 环境配置步骤

## 当前状态

✅ Ubuntu 22.04 LTS 已安装  
✅ 用户名：tinot  
✅ 系统已启动

## 接下来的步骤

### 步骤 1：更新系统包

在 Ubuntu 22.04 终端中运行：

```bash
# 更新包列表
sudo apt update

# 升级系统包（可选，但推荐）
sudo apt upgrade -y
```

### 步骤 2：安装基础工具

```bash
# 安装常用工具
sudo apt install -y curl wget git build-essential

# 安装 Python 开发工具
sudo apt install -y python3-pip python3-venv python3-dev
```

### 步骤 3：验证 Python 版本

```bash
python3 --version
```

**预期输出**：`Python 3.10.x`

### 步骤 4：进入项目目录

```bash
# WSL 中的 Windows 路径映射
cd /mnt/d/Programs/github/lingua

# 验证目录
pwd
ls -la
```

### 步骤 5：创建虚拟环境

```bash
# 创建虚拟环境
python3 -m venv venv-wsl

# 激活虚拟环境
source venv-wsl/bin/activate
```

**预期输出**：提示符变为 `(venv-wsl) tinot@...`

### 步骤 6：升级 pip

```bash
pip install --upgrade pip
```

### 步骤 7：安装依赖

```bash
# 基础依赖
pip install numpy soundfile flask

# PyTorch (GPU, CUDA 12.1)
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121

# 其他依赖
pip install onnx onnxruntime TTS fastapi uvicorn pydantic
```

### 步骤 8：验证安装

```bash
# 验证 PyTorch
python3 -c "import torch; print('PyTorch:', torch.__version__); print('CUDA:', torch.cuda.is_available())"

# 验证 TTS
python3 -c "from TTS.api import TTS; print('✅ TTS OK')"
```

---

## 快速执行（使用自动化脚本）

或者直接运行自动化脚本：

```bash
cd /mnt/d/Programs/github/lingua
bash core/engine/scripts/setup_wsl_env.sh
```

脚本会自动完成所有配置步骤。

