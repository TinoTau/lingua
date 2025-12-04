#!/bin/bash
# 在 WSL 中启动 YourTTS 服务（使用 Python 3.10）

# 获取脚本目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"

# 切换到项目目录
cd "$PROJECT_ROOT"

# 激活虚拟环境（Python 3.10）
if [ -d "venv-wsl-py310" ]; then
    source venv-wsl-py310/bin/activate
    echo "✅ 已激活虚拟环境: venv-wsl-py310"
    
    # 验证 Python 版本
    PYTHON_VER=$(python --version 2>&1)
    if ! echo "$PYTHON_VER" | grep -q "3.10"; then
        echo "⚠️  警告: 虚拟环境 Python 版本不是 3.10: $PYTHON_VER"
    else
        echo "✅ Python 版本: $PYTHON_VER"
    fi
else
    echo "❌ 错误: 虚拟环境 venv-wsl-py310 不存在"
    echo "   请先运行: bash core/engine/scripts/setup_python310_env.sh"
    exit 1
fi

echo "============================================================"
echo "  Starting YourTTS Service in WSL (Python 3.10)"
echo "============================================================"
echo "Project root: $PROJECT_ROOT"
echo ""

# 检查 GPU 是否可用
echo "Checking GPU availability..."
if command -v nvidia-smi &> /dev/null; then
    nvidia-smi --query-gpu=name --format=csv,noheader
    echo ""
    USE_GPU="--gpu"
else
    echo "GPU not available, using CPU"
    USE_GPU=""
fi

# 启动 YourTTS 服务
# 注意：host 设置为 0.0.0.0 以允许从 Windows 访问
echo "Starting YourTTS service..."
echo "  Port: 5004"
echo "  Host: 0.0.0.0 (accessible from Windows)"
echo "  GPU: $([ -n "$USE_GPU" ] && echo "Yes" || echo "No")"
echo ""

python3 core/engine/scripts/yourtts_service.py $USE_GPU --port 5004 --host 0.0.0.0

