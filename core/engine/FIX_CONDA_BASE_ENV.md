# 修复 Conda Base 环境问题

## 问题描述

conda 命令无法执行，错误信息：
```
Unable to create process using 'D:\Program Files\Anaconda\python.exe'
```

但环境已激活（Python 3.10.19 可用）。

## 临时解决方案：直接使用 Python 安装

由于环境已激活，可以直接使用 Python 的完整路径：

### 安装依赖

```powershell
# 使用 Python 的 -m pip 方式安装
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install numpy soundfile flask

# 安装 torchaudio
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install torchaudio

# 安装 SpeechBrain
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install speechbrain
```

### 安装 PyTorch（GPU）

```powershell
# 使用 pip 安装 PyTorch（因为 conda 命令不可用）
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
```

### 验证安装

```powershell
# 验证 PyTorch
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -c "import torch; print('PyTorch:', torch.__version__); print('CUDA:', torch.cuda.is_available())"

# 验证其他包
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -c "import numpy, soundfile, flask, speechbrain; print('All packages OK')"
```

## 永久解决方案：修复 Conda Base 环境

### 方案 1：使用 Anaconda Prompt

1. 关闭当前 PowerShell
2. 打开 **Anaconda Prompt**（从开始菜单）
3. 在 Anaconda Prompt 中运行：

```powershell
# 检查 base 环境
conda info

# 如果 base 环境有问题，重新安装 conda
conda update conda -y
```

### 方案 2：重新安装 Anaconda（最后手段）

如果 Anaconda Prompt 也有问题，可能需要：
1. 备份重要环境配置
2. 重新安装 Anaconda
3. 重新创建环境

### 方案 3：使用完整路径创建别名（临时）

在 PowerShell 中创建函数：

```powershell
# 创建 pip 别名
function pip {
    & "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip $args
}

# 创建 python 别名（如果需要）
function python {
    & "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" $args
}

# 现在可以直接使用
pip install numpy
python --version
```

## 推荐操作流程

### 立即操作（使用完整路径）

```powershell
# 1. 安装基础依赖
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install numpy soundfile flask

# 2. 安装 PyTorch（先检查 CUDA 版本）
# 运行 nvidia-smi 查看 CUDA 版本，然后选择对应的 PyTorch
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121

# 3. 安装其他依赖
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install torchaudio speechbrain

# 4. 验证
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -c "import torch; print('CUDA:', torch.cuda.is_available())"
```

### 后续修复（使用 Anaconda Prompt）

1. 打开 Anaconda Prompt
2. 运行 `conda update conda -y`
3. 如果还有问题，考虑重新安装 Anaconda

## 运行服务

安装完成后，运行服务时也使用完整路径：

```powershell
# 运行 Speaker Embedding 服务
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" core\engine\scripts\speaker_embedding_service.py --gpu
```

