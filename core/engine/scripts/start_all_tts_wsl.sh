#!/bin/bash
# 在 WSL 中启动所有 TTS 相关服务（使用 Python 3.10）

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
cd "$PROJECT_ROOT"

echo "============================================================"
echo "  启动所有 TTS 服务（WSL Python 3.10）"
echo "============================================================"
echo ""

# 激活 Python 3.10 虚拟环境
if [ -d "venv-wsl-py310" ]; then
    source venv-wsl-py310/bin/activate
    echo "✅ 已激活虚拟环境: venv-wsl-py310"
    
    # 验证 Python 版本
    PYTHON_VER=$(python --version 2>&1)
    if echo "$PYTHON_VER" | grep -q "3.10"; then
        echo "✅ Python 版本: $PYTHON_VER"
    else
        echo "⚠️  警告: Python 版本不是 3.10: $PYTHON_VER"
        echo "   建议检查虚拟环境是否正确设置"
    fi
else
    echo "❌ 错误: 虚拟环境 venv-wsl-py310 不存在"
    echo "   请先运行: bash core/engine/scripts/setup_python310_env.sh"
    exit 1
fi

# 检查 GPU
echo ""
echo "检查 GPU 可用性..."
USE_GPU=""
if command -v nvidia-smi &> /dev/null; then
    GPU_INFO=$(nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "✅ GPU 可用: $GPU_INFO"
        USE_GPU="--gpu"
    else
        echo "⚠️  GPU 检查失败，使用 CPU"
    fi
else
    echo "⚠️  nvidia-smi 不可用，使用 CPU"
fi

echo ""
echo "============================================================"
echo "  启动服务"
echo "============================================================"
echo ""

# 启动 Speaker Embedding 服务
echo "1. 启动 Speaker Embedding 服务..."
python core/engine/scripts/speaker_embedding_service.py $USE_GPU --port 5003 --host 0.0.0.0 > /tmp/speaker_embedding.log 2>&1 &
SPEAKER_EMBEDDING_PID=$!
echo "   PID: $SPEAKER_EMBEDDING_PID"
echo "   Log: /tmp/speaker_embedding.log"
echo "   URL: http://127.0.0.1:5003"
echo ""

# 等待服务启动
echo "   等待服务启动..."
sleep 5

# 启动 YourTTS 服务
echo "2. 启动 YourTTS 服务..."
python core/engine/scripts/yourtts_service.py $USE_GPU --port 5004 --host 0.0.0.0 > /tmp/yourtts.log 2>&1 &
YOURTTS_PID=$!
echo "   PID: $YOURTTS_PID"
echo "   Log: /tmp/yourtts.log"
echo "   URL: http://127.0.0.1:5004"
echo ""

# 保存 PIDs
echo $SPEAKER_EMBEDDING_PID > /tmp/speaker_embedding.pid
echo $YOURTTS_PID > /tmp/yourtts.pid

echo "============================================================"
echo "  ✅ 所有服务已启动"
echo "============================================================"
echo ""
echo "服务状态:"
echo "  - Speaker Embedding: PID $SPEAKER_EMBEDDING_PID (http://127.0.0.1:5003)"
echo "  - YourTTS: PID $YOURTTS_PID (http://127.0.0.1:5004)"
echo ""
echo "查看日志:"
echo "  tail -f /tmp/speaker_embedding.log"
echo "  tail -f /tmp/yourtts.log"
echo ""
echo "停止服务:"
echo "  kill $SPEAKER_EMBEDDING_PID $YOURTTS_PID"
echo "  或: kill \$(cat /tmp/speaker_embedding.pid) \$(cat /tmp/yourtts.pid)"
echo ""

