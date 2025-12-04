#!/bin/bash
# 启动 Speaker Embedding 和 YourTTS 服务（Linux/WSL）

echo "============================================================"
echo "  Starting TTS Services in WSL"
echo "============================================================"
echo ""

# 获取脚本目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
cd "$PROJECT_ROOT"

# 激活 Python 3.10 虚拟环境（用于 YourTTS）
if [ -d "venv-wsl-py310" ]; then
    source venv-wsl-py310/bin/activate
    echo "✅ 已激活虚拟环境: venv-wsl-py310 (Python 3.10)"
    PYTHON_VER=$(python --version 2>&1)
    if echo "$PYTHON_VER" | grep -q "3.10"; then
        echo "✅ Python 版本: $PYTHON_VER"
    else
        echo "⚠️  警告: Python 版本不是 3.10: $PYTHON_VER"
    fi
elif [ -d "venv-wsl" ]; then
    source venv-wsl/bin/activate
    echo "⚠️  使用旧环境: venv-wsl (建议使用 venv-wsl-py310)"
    echo "   请运行: bash core/engine/scripts/setup_python310_env.sh"
else
    echo "❌ 错误: Python 虚拟环境不存在"
    echo "   请先运行: bash core/engine/scripts/setup_python310_env.sh"
    exit 1
fi

echo ""
echo "检查 CUDA 是否可用（可选）..."
python -c "import torch; print('CUDA available:', torch.cuda.is_available())" 2>&1 || echo "Warning: Could not check CUDA availability"

# 检查 GPU
if command -v nvidia-smi &> /dev/null; then
    echo ""
    echo "GPU 信息:"
    nvidia-smi --query-gpu=name --format=csv,noheader
    USE_GPU="--gpu"
else
    echo ""
    echo "GPU not available, using CPU"
    USE_GPU=""
fi

# 启动 Speaker Embedding 服务（GPU 模式）
echo ""
echo "============================================================"
echo "  Starting Speaker Embedding Service"
echo "============================================================"
echo "Starting Speaker Embedding service..."
python core/engine/scripts/speaker_embedding_service.py $USE_GPU --port 5003 --host 0.0.0.0 &
SPEAKER_EMBEDDING_PID=$!
echo "✅ Speaker Embedding service started (PID: $SPEAKER_EMBEDDING_PID)"

# 等待服务启动
echo "Waiting for Speaker Embedding service to start..."
sleep 5

# 启动 YourTTS 服务（GPU 模式）
echo ""
echo "============================================================"
echo "  Starting YourTTS Service"
echo "============================================================"
echo "Starting YourTTS service..."
python core/engine/scripts/yourtts_service.py $USE_GPU --port 5004 --host 0.0.0.0 &
YOURTTS_PID=$!
echo "✅ YourTTS service started (PID: $YOURTTS_PID)"

echo ""
echo "✅ Services started!"
echo "   Speaker Embedding: PID $SPEAKER_EMBEDDING_PID (http://127.0.0.1:5003)"
echo "   YourTTS: PID $YOURTTS_PID (http://127.0.0.1:5004)"
echo ""
echo "To stop services, run:"
echo "   kill $SPEAKER_EMBEDDING_PID $YOURTTS_PID"
echo ""
echo "Or save PIDs to file:"
echo "   echo $SPEAKER_EMBEDDING_PID > /tmp/speaker_embedding.pid"
echo "   echo $YOURTTS_PID > /tmp/yourtts.pid"

# 保存 PIDs
echo $SPEAKER_EMBEDDING_PID > /tmp/speaker_embedding.pid
echo $YOURTTS_PID > /tmp/yourtts.pid

