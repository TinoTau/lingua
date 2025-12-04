# 修复 Conda 环境问题

## 问题描述

激活 conda 环境后，出现以下错误：
- `Unable to create process using 'D:\Program Files\Anaconda\python.exe'`
- pip 命令无法执行

## 原因分析

这通常是因为：
1. conda 环境创建不完整
2. Python 可执行文件路径有问题
3. 环境配置损坏

## 解决方案

### 方案 1：重新创建环境（推荐）

```powershell
# 1. 退出当前环境（如果已激活）
conda deactivate

# 2. 删除有问题的环境
conda env remove -n lingua-py310 -y

# 3. 清理 conda 缓存
conda clean --all -y

# 4. 重新创建环境
conda create -n lingua-py310 python=3.10 -y

# 5. 激活环境
conda activate lingua-py310

# 6. 验证 Python 可用
python --version

# 7. 验证 pip 可用
pip --version
```

### 方案 2：修复现有环境

```powershell
# 1. 退出当前环境
conda deactivate

# 2. 重新安装 Python
conda install -n lingua-py310 python=3.10 -y

# 3. 重新安装 pip
conda install -n lingua-py310 pip -y

# 4. 激活环境
conda activate lingua-py310

# 5. 验证
python --version
pip --version
```

### 方案 3：使用完整路径（临时方案）

如果环境已激活但命令不可用，可以尝试：

```powershell
# 使用完整路径运行 Python
& "D:\Program Files\Anaconda\envs\lingua-py310\python.exe" --version

# 使用完整路径运行 pip
& "D:\Program Files\Anaconda\envs\lingua-py310\Scripts\pip.exe" install numpy
```

## 验证环境

环境修复后，运行以下命令验证：

```powershell
# 激活环境
conda activate lingua-py310

# 检查 Python
python --version
# 应该显示：Python 3.10.x

# 检查 pip
pip --version
# 应该显示：pip x.x.x from ... lingua-py310 ...

# 检查 conda
conda info --envs
# 应该显示 * lingua-py310
```

## 如果问题仍然存在

### 检查 Anaconda 安装

```powershell
# 检查 conda 根目录
conda info --base

# 检查环境目录
conda env list
```

### 重新初始化 conda

```powershell
# 重新初始化 conda for PowerShell
conda init powershell

# 关闭并重新打开 PowerShell
```

### 使用 Anaconda Prompt

如果 PowerShell 有问题，可以尝试使用 **Anaconda Prompt**：
1. 打开 "Anaconda Prompt"（从开始菜单）
2. 运行相同的命令

