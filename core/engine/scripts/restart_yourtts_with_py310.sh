#!/bin/bash
# 停止旧服务并使用 Python 3.10 环境重新启动

cd /mnt/d/Programs/github/lingua

echo "============================================================"
echo "  重启 YourTTS 服务（使用 Python 3.10 环境）"
echo "============================================================"
echo ""

# 1. 停止旧的 YourTTS 服务
echo "1. 停止旧的 YourTTS 服务..."
YOURTTS_PIDS=$(ps aux | grep "yourtts_service.py" | grep -v grep | awk '{print $2}')

if [ -z "$YOURTTS_PIDS" ]; then
    echo "   ✅ 没有运行中的 YourTTS 服务"
else
    echo "   发现运行中的服务进程："
    for pid in $YOURTTS_PIDS; do
        echo "   停止 PID: $pid"
        kill $pid 2>/dev/null
    done
    
    # 等待进程停止
    echo "   等待进程停止..."
    sleep 2
    
    # 检查是否还有进程
    REMAINING=$(ps aux | grep "yourtts_service.py" | grep -v grep | awk '{print $2}')
    if [ -n "$REMAINING" ]; then
        echo "   ⚠️  强制停止剩余进程..."
        for pid in $REMAINING; do
            kill -9 $pid 2>/dev/null
        done
        sleep 1
    fi
    echo "   ✅ 所有旧服务已停止"
fi

echo ""
echo "2. 检查 Python 3.10 环境..."
if [ ! -d "venv-wsl-py310" ]; then
    echo "   ❌ venv-wsl-py310 环境不存在"
    echo "   请先运行: bash core/engine/scripts/setup_python310_env.sh"
    exit 1
fi

# 激活新环境
source venv-wsl-py310/bin/activate

# 验证环境
PYTHON_VER=$(python --version 2>&1)
if ! echo "$PYTHON_VER" | grep -q "3.10"; then
    echo "   ❌ Python 版本不是 3.10: $PYTHON_VER"
    exit 1
fi
echo "   ✅ Python 版本: $PYTHON_VER"

# 验证依赖
echo ""
echo "3. 验证依赖版本..."
python -c "
import numpy, numba, librosa
print(f'   numpy: {numpy.__version__}')
print(f'   numba: {numba.__version__}')
print(f'   librosa: {librosa.__version__}')
" || {
    echo "   ❌ 依赖检查失败"
    exit 1
}

# 测试 librosa
echo ""
echo "4. 测试 librosa..."
python -c "
import numpy as np
import librosa
test_audio = np.random.randn(1000).astype(np.float64)
try:
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print('   ✅ librosa.effects.time_stretch 测试通过')
except Exception as e:
    print(f'   ❌ librosa 测试失败: {e}')
    exit(1)
" || {
    echo "   ❌ librosa 测试失败，环境可能有问题"
    exit 1
}

echo ""
echo "============================================================"
echo "  启动 YourTTS 服务（Python 3.10）"
echo "============================================================"
echo ""

# 检查 GPU
USE_GPU=""
if command -v nvidia-smi &> /dev/null; then
    if nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null | head -1 > /dev/null 2>&1; then
        GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null | head -1)
        echo "✅ GPU 可用: $GPU_NAME"
        USE_GPU="--gpu"
    else
        echo "⚠️  GPU 检查失败，使用 CPU"
    fi
else
    echo "⚠️  nvidia-smi 不可用，使用 CPU"
fi

echo ""
echo "启动服务..."
echo "  Port: 5004"
echo "  Host: 0.0.0.0"
echo "  GPU: $([ -n "$USE_GPU" ] && echo "Yes" || echo "No")"
echo ""
echo "⚠️  注意：服务将在前台运行"
echo "   按 Ctrl+C 停止服务"
echo ""

# 启动服务
python core/engine/scripts/yourtts_service.py $USE_GPU --port 5004 --host 0.0.0.0

