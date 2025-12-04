# 安装和使用 Python 3.10

## 当前情况

你的系统中 `python3.10` 命令指向的是 Python 3.12.3，不是真正的 Python 3.10。

## 安装真正的 Python 3.10

在 WSL 中运行：

```bash
# 1. 更新包列表
sudo apt update

# 2. 安装必要的工具
sudo apt install software-properties-common

# 3. 添加 deadsnakes PPA（提供多个 Python 版本）
sudo add-apt-repository ppa:deadsnakes/ppa

# 4. 更新包列表
sudo apt update

# 5. 安装 Python 3.10
sudo apt install python3.10 python3.10-venv python3.10-dev

# 6. 验证安装
python3.10 --version
# 应该显示: Python 3.10.x
```

## 创建 Python 3.10 虚拟环境

```bash
cd /mnt/d/Programs/github/lingua

# 创建新的虚拟环境
python3.10 -m venv venv-wsl-py310

# 激活新环境
source venv-wsl-py310/bin/activate

# 验证 Python 版本
python --version
# 应该显示: Python 3.10.x

# 升级 pip
pip install --upgrade pip

# 安装依赖
pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir
```

## 修改启动脚本

创建或修改启动脚本使用新的虚拟环境：

### 方法 1: 修改现有脚本

编辑 `core/engine/scripts/start_yourtts_wsl.sh`：

```bash
# 将这一行：
source venv-wsl/bin/activate

# 改为：
source venv-wsl-py310/bin/activate
```

### 方法 2: 创建新的启动脚本

创建 `core/engine/scripts/start_yourtts_wsl_py310.sh`：

```bash
#!/bin/bash
# 在 WSL 中启动 YourTTS 服务（使用 Python 3.10）

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
cd "$PROJECT_ROOT"

# 激活 Python 3.10 虚拟环境
if [ -d "venv-wsl-py310" ]; then
    source venv-wsl-py310/bin/activate
    echo "✅ 已激活虚拟环境: venv-wsl-py310"
else
    echo "❌ 错误: 虚拟环境 venv-wsl-py310 不存在"
    echo "   请先运行: python3.10 -m venv venv-wsl-py310"
    exit 1
fi

# 其余代码与 start_yourtts_wsl.sh 相同
# ...
```

## 或者：继续使用 Python 3.12（代码已有 fallback）

如果你不想安装 Python 3.10，也可以继续使用 Python 3.12：

1. 使用 numpy 1.26.4（已安装）
2. 代码已配置自动 fallback 到 scipy
3. 即使 librosa 失败，服务仍可正常运行

```bash
# 在当前 venv-wsl 环境中
source venv-wsl/bin/activate
pip install 'numpy==1.26.4' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir --force-reinstall
```

## 建议

- **如果希望最佳兼容性**：安装 Python 3.10 并创建新虚拟环境
- **如果希望快速解决**：继续使用 Python 3.12，依赖代码的 fallback 机制

