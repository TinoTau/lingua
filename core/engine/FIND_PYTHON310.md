# 查找 Python 3.10 环境

## 快速查找命令

在 WSL 中运行查找脚本：

```bash
cd /mnt/d/Programs/github/lingua
bash core/engine/scripts/find_python310.sh
```

## 手动查找方法

### 方法 1: 直接测试 python3.10

```bash
python3.10 --version
```

如果可用，会显示版本号。

### 方法 2: 查找虚拟环境

```bash
# 在当前项目目录查找
cd /mnt/d/Programs/github/lingua
find . -maxdepth 2 -name "venv*" -type d

# 检查每个虚拟环境的 Python 版本
for venv in $(find . -maxdepth 2 -name "venv*" -type d); do
    if [ -f "$venv/bin/python" ]; then
        echo "$venv: $($venv/bin/python --version 2>&1)"
    fi
done
```

### 方法 3: 查找系统 Python 3.10

```bash
# 查找所有 Python 3.x
ls -la /usr/bin/python3*

# 或使用 whereis
whereis python3.10

# 或使用 find
find /usr/bin /usr/local/bin -name "python3.10" 2>/dev/null
```

### 方法 4: 查找 conda 环境

```bash
# 如果使用 conda
conda env list

# 或直接查找
ls -la ~/anaconda3/envs/ 2>/dev/null
ls -la ~/miniconda3/envs/ 2>/dev/null
```

## 如果找到了 Python 3.10

### 选项 1: 使用现有的 Python 3.10 虚拟环境

```bash
# 找到的虚拟环境路径（假设是 venv-py310）
cd /mnt/d/Programs/github/lingua
source venv-py310/bin/activate

# 然后安装依赖
pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir
```

### 选项 2: 创建新的 Python 3.10 虚拟环境

```bash
cd /mnt/d/Programs/github/lingua

# 创建新环境
python3.10 -m venv venv-wsl-py310

# 激活
source venv-wsl-py310/bin/activate

# 升级 pip
pip install --upgrade pip

# 安装依赖
pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir
```

### 选项 3: 修改现有虚拟环境（不推荐）

可以尝试修复现有虚拟环境，但更简单的方法是创建新环境。

## 如果没有 Python 3.10

### 安装 Python 3.10

```bash
sudo apt update
sudo apt install software-properties-common
sudo add-apt-repository ppa:deadsnakes/ppa
sudo apt update
sudo apt install python3.10 python3.10-venv python3.10-dev

# 然后创建虚拟环境
python3.10 -m venv venv-wsl-py310
```

## 修改启动脚本

如果创建了新的虚拟环境，需要修改启动脚本：

```bash
# 修改 start_yourtts_wsl.sh
# 将 source venv-wsl/bin/activate 改为：
source venv-wsl-py310/bin/activate
```

