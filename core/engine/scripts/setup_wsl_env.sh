#!/bin/bash
# WSL 环境配置脚本
# 用于配置 Ubuntu 22.04 并安装所有 TTS 服务依赖

echo "============================================================"
echo "  WSL 环境配置脚本"
echo "============================================================"
echo ""

# 检查是否在 WSL 中
if [ -z "$WSL_DISTRO_NAME" ] && [ -z "$WSLENV" ]; then
    echo "⚠️  警告: 未检测到 WSL 环境"
    echo "   建议在 WSL 中运行此脚本"
    echo ""
fi

# 检查 Ubuntu 版本
if [ -f /etc/os-release ]; then
    . /etc/os-release
    echo "检测到系统: $NAME $VERSION"
    if [[ "$VERSION_ID" != "22.04" ]]; then
        echo "⚠️  警告: 当前 Ubuntu 版本不是 22.04"
        echo "   推荐使用 Ubuntu 22.04 以获得最佳兼容性"
        echo ""
    fi
fi

echo ""
echo "步骤 1: 更新系统包..." -ForegroundColor Yellow
sudo apt update
sudo apt upgrade -y

echo ""
echo "步骤 2: 安装 Python 3.10..." -ForegroundColor Yellow

# 检查 Python 3.10 是否已安装
if command -v python3.10 &> /dev/null; then
    echo "✅ Python 3.10 已安装"
    python3.10 --version
else
    echo "安装 Python 3.10..."
    sudo apt install -y python3.10 python3.10-venv python3.10-dev python3-pip
    
    # 设置 python3 指向 3.10
    sudo update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.10 1
fi

echo ""
echo "步骤 3: 创建虚拟环境..." -ForegroundColor Yellow

# 获取脚本目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
VENV_PATH="$PROJECT_ROOT/venv-wsl"

cd "$PROJECT_ROOT"

if [ -d "$VENV_PATH" ]; then
    echo "⚠️  虚拟环境已存在: $VENV_PATH"
    read -p "是否删除并重新创建? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "删除现有虚拟环境..."
        rm -rf "$VENV_PATH"
    else
        echo "使用现有虚拟环境"
    fi
fi

if [ ! -d "$VENV_PATH" ]; then
    echo "创建虚拟环境..."
    python3.10 -m venv "$VENV_PATH"
    if [ $? -ne 0 ]; then
        echo "❌ 虚拟环境创建失败"
        exit 1
    fi
    echo "✅ 虚拟环境创建成功"
fi

echo ""
echo "步骤 4: 激活虚拟环境并升级 pip..." -ForegroundColor Yellow
source "$VENV_PATH/bin/activate"
pip install --upgrade pip

echo ""
echo "步骤 5: 安装 PyTorch (GPU, CUDA 12.1)..." -ForegroundColor Yellow
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121

echo ""
echo "步骤 6: 安装其他依赖..." -ForegroundColor Yellow

dependencies=(
    "numpy"
    "soundfile"
    "flask"
    "onnx"
    "onnxruntime"
    "TTS"
    "fastapi"
    "uvicorn"
    "pydantic"
)

for dep in "${dependencies[@]}"; do
    echo "  安装 $dep..."
    pip install "$dep"
done

echo ""
echo "步骤 7: 验证安装..." -ForegroundColor Yellow

python3 -c "
import torch
print('PyTorch:', torch.__version__)
print('CUDA available:', torch.cuda.is_available())
if torch.cuda.is_available():
    print('CUDA version:', torch.version.cuda)
    print('GPU:', torch.cuda.get_device_name(0))
else:
    print('⚠️  CUDA 不可用（将使用 CPU）')
"

python3 -c "from TTS.api import TTS; print('✅ TTS library OK')" 2>/dev/null || echo "⚠️  TTS 库验证失败"

echo ""
echo "============================================================"
echo "✅ WSL 环境配置完成！"
echo "============================================================"
echo ""
echo "使用以下命令激活环境："
echo "  source $VENV_PATH/bin/activate"
echo ""
echo "启动服务："
echo "  python3 core/engine/scripts/yourtts_service.py --gpu --host 0.0.0.0"
echo ""

