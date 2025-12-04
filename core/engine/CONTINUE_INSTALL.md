# 继续安装依赖（conda 命令不可用时）

## 当前状态

✅ PyTorch 已安装：2.5.1+cu121  
✅ CUDA 可用：True  
⚠️ conda 命令不可用（base 环境有问题）

## 解决方案：使用完整路径继续安装

由于 conda 命令有问题，但环境本身正常，可以直接使用完整路径安装剩余依赖。

### 步骤 1：安装基础依赖

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install numpy soundfile flask
```

### 步骤 2：安装 torchaudio（如果还没装）

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install torchaudio
```

### 步骤 3：安装 SpeechBrain

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -m pip install speechbrain
```

### 步骤 4：验证所有依赖

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -c "import numpy, soundfile, flask, torch, torchaudio, speechbrain; print('✅ 所有依赖安装成功')"
```

### 步骤 5：验证 PyTorch GPU

```powershell
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" -c "import torch; print('PyTorch:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('CUDA version:', torch.version.cuda if torch.cuda.is_available() else 'N/A'); print('GPU:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"
```

## 简化命令（创建别名）

为了简化后续操作，可以在 PowerShell 中创建函数：

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
pip install numpy soundfile flask
python --version
```

## 运行服务

安装完成后，运行服务：

```powershell
# 使用完整路径运行服务
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" core\engine\scripts\speaker_embedding_service.py --gpu
```

## 关于 conda activate

**不需要再执行 `conda activate lingua-py310`**，因为：
1. conda 命令本身有问题（base 环境损坏）
2. 环境已经配置好了，可以直接使用完整路径
3. 或者使用 Anaconda Prompt（通常更稳定）

## 后续修复 conda（可选）

如果需要修复 conda 命令，可以：
1. 使用 **Anaconda Prompt**（从开始菜单打开）
2. 在 Anaconda Prompt 中运行：`conda update conda -y`
3. 或者重新安装 Anaconda

但这不是必须的，因为环境本身已经可以正常使用了。

