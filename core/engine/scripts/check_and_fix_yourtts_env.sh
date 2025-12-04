#!/bin/bash
# 检查并修复 YourTTS 服务运行环境

cd /mnt/d/Programs/github/lingua

echo "============================================================"
echo "  检查 YourTTS 服务运行环境"
echo "============================================================"
echo ""

# 检查是否有 YourTTS 服务在运行
echo "1. 检查运行中的 YourTTS 服务..."
YOURTTS_PIDS=$(ps aux | grep "yourtts_service.py" | grep -v grep | awk '{print $2}')

if [ -z "$YOURTTS_PIDS" ]; then
    echo "   ✅ 没有运行中的 YourTTS 服务"
    echo ""
else
    echo "   ⚠️  发现运行中的 YourTTS 服务进程："
    for pid in $YOURTTS_PIDS; do
        echo "      PID: $pid"
        ps -p $pid -o args= | head -1 | sed 's/^/          /'
        
        # 检查进程使用的 Python 路径
        PYTHON_PATH=$(readlink -f /proc/$pid/exe 2>/dev/null || ps -p $pid -o comm= 2>/dev/null)
        if echo "$PYTHON_PATH" | grep -q "venv-wsl-py310"; then
            echo "      ✅ 使用正确的环境: venv-wsl-py310"
        elif echo "$PYTHON_PATH" | grep -q "venv-wsl"; then
            echo "      ❌ 使用错误的环境: venv-wsl (需要停止并重启)"
            echo "      停止命令: kill $pid"
        else
            echo "      ⚠️  Python 路径: $PYTHON_PATH"
        fi
        echo ""
    done
fi

echo "2. 检查环境中的依赖版本..."
echo ""

# 检查 venv-wsl (旧环境)
if [ -d "venv-wsl" ]; then
    echo "   旧环境 (venv-wsl):"
    source venv-wsl/bin/activate 2>/dev/null
    if [ $? -eq 0 ]; then
        PYTHON_VER=$(python --version 2>&1)
        NUMPY_VER=$(python -c "import numpy; print(numpy.__version__)" 2>/dev/null || echo "未安装")
        NUMBA_VER=$(python -c "import numba; print(numba.__version__)" 2>/dev/null || echo "未安装")
        LIBROSA_VER=$(python -c "import librosa; print(librosa.__version__)" 2>/dev/null || echo "未安装")
        echo "      Python: $PYTHON_VER"
        echo "      numpy: $NUMPY_VER"
        echo "      numba: $NUMBA_VER"
        echo "      librosa: $LIBROSA_VER"
        
        # 测试 librosa
        echo -n "      librosa 测试: "
        python -c "import numpy as np; import librosa; test_audio = np.random.randn(1000).astype(np.float64); librosa.effects.time_stretch(test_audio, rate=1.0); print('✅ 通过')" 2>&1 | grep -q "通过" && echo "✅ 通过" || echo "❌ 失败"
        deactivate 2>/dev/null
    fi
    echo ""
fi

# 检查 venv-wsl-py310 (新环境)
if [ -d "venv-wsl-py310" ]; then
    echo "   新环境 (venv-wsl-py310):"
    source venv-wsl-py310/bin/activate 2>/dev/null
    if [ $? -eq 0 ]; then
        PYTHON_VER=$(python --version 2>&1)
        NUMPY_VER=$(python -c "import numpy; print(numpy.__version__)" 2>/dev/null || echo "未安装")
        NUMBA_VER=$(python -c "import numba; print(numba.__version__)" 2>/dev/null || echo "未安装")
        LIBROSA_VER=$(python -c "import librosa; print(librosa.__version__)" 2>/dev/null || echo "未安装")
        echo "      Python: $PYTHON_VER"
        echo "      numpy: $NUMPY_VER"
        echo "      numba: $NUMBA_VER"
        echo "      librosa: $LIBROSA_VER"
        
        # 测试 librosa
        echo -n "      librosa 测试: "
        python -c "import numpy as np; import librosa; test_audio = np.random.randn(1000).astype(np.float64); librosa.effects.time_stretch(test_audio, rate=1.0); print('✅ 通过')" 2>&1 | grep -q "通过" && echo "✅ 通过" || echo "❌ 失败"
        deactivate 2>/dev/null
    fi
    echo ""
fi

echo "============================================================"
echo "  结论和建议"
echo "============================================================"
echo ""

if [ -n "$YOURTTS_PIDS" ]; then
    echo "⚠️  发现运行中的服务，建议："
    echo ""
    echo "1. 停止旧服务："
    for pid in $YOURTTS_PIDS; do
        echo "   kill $pid"
    done
    echo ""
    echo "2. 使用新环境重启服务："
    echo "   bash core/engine/scripts/start_yourtts_wsl.sh"
    echo ""
else
    echo "✅ 可以启动服务："
    echo "   bash core/engine/scripts/start_yourtts_wsl.sh"
    echo ""
fi

