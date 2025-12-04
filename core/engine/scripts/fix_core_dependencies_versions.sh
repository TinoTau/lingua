#!/bin/bash
# 修复核心依赖版本（确保兼容性）

cd /mnt/d/Programs/github/lingua

echo "============================================================"
echo "  修复核心依赖版本"
echo "============================================================"
echo ""

# 激活虚拟环境
if [ -d "venv-wsl-py310" ]; then
    source venv-wsl-py310/bin/activate
    echo "✅ 已激活虚拟环境: venv-wsl-py310"
else
    echo "❌ 错误: 虚拟环境不存在"
    exit 1
fi

echo ""
echo "当前版本："
python -c "import numpy, numba, librosa, scipy; print(f'numpy: {numpy.__version__}'); print(f'numba: {numba.__version__}'); print(f'librosa: {librosa.__version__}'); print(f'scipy: {scipy.__version__}')"

echo ""
echo "============================================================"
echo "  检查版本兼容性"
echo "============================================================"
echo ""

# 检查是否需要修复
NEED_FIX=false

# 检查 numpy
NUMPY_VER=$(python -c "import numpy; print(numpy.__version__)" 2>/dev/null)
if [[ "$NUMPY_VER" != "1.24.3" ]]; then
    echo "⚠️  numpy 版本: $NUMPY_VER (期望: 1.24.3)"
    NEED_FIX=true
else
    echo "✅ numpy 版本正确: $NUMPY_VER"
fi

# 检查 librosa
LIBROSA_VER=$(python -c "import librosa; print(librosa.__version__)" 2>/dev/null)
if [[ "$LIBROSA_VER" != "0.10.1" ]]; then
    echo "⚠️  librosa 版本: $LIBROSA_VER (期望: 0.10.1)"
    NEED_FIX=true
else
    echo "✅ librosa 版本正确: $LIBROSA_VER"
fi

# 检查 numba
NUMBA_VER=$(python -c "import numba; print(numba.__version__)" 2>/dev/null)
if [[ "$NUMBA_VER" != "0.59.1" ]]; then
    echo "⚠️  numba 版本: $NUMBA_VER (期望: 0.59.1)"
    NEED_FIX=true
else
    echo "✅ numba 版本正确: $NUMBA_VER"
fi

if [ "$NEED_FIX" = false ]; then
    echo ""
    echo "✅ 所有核心依赖版本正确，无需修复"
    echo ""
    echo "测试 librosa 功能..."
    python -c "
import numpy as np
import librosa
test_audio = np.random.randn(1000).astype(np.float64)
try:
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print('✅ librosa.effects.time_stretch 测试通过')
except Exception as e:
    print(f'❌ 测试失败: {e}')
    exit(1)
"
    if [ $? -eq 0 ]; then
        echo ""
        echo "✅ 所有测试通过，环境配置正确！"
        exit 0
    fi
fi

echo ""
echo "============================================================"
echo "  修复版本"
echo "============================================================"
echo ""

# 清除 numba 缓存
echo "1. 清除 numba 缓存..."
rm -rf ~/.cache/numba
echo "✅ 缓存已清除"

echo ""
echo "2. 安装正确的版本..."
echo "   注意：可能会降级一些依赖，但这是为了确保兼容性"

# 强制重新安装核心依赖
pip install --force-reinstall --no-cache-dir \
    'numpy==1.24.3' \
    'numba==0.59.1' \
    'librosa==0.10.1' \
    'llvmlite==0.42.0'

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ 安装失败"
    exit 1
fi

echo ""
echo "============================================================"
echo "  验证修复"
echo "============================================================"
echo ""

python -c "
import numpy as np
import numba
import librosa
import scipy

print('修复后的版本：')
print(f'  numpy: {np.__version__}')
print(f'  numba: {numba.__version__}')
print(f'  librosa: {librosa.__version__}')
print(f'  scipy: {scipy.__version__}')
print()

# 测试 librosa
print('测试 librosa.effects.time_stretch...')
test_audio = np.random.randn(1000).astype(np.float64)
try:
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print('✅ librosa.effects.time_stretch 测试通过')
    
    # 测试不同 rate
    for rate in [0.8, 1.2]:
        stretched = librosa.effects.time_stretch(test_audio, rate=rate)
        print(f'  ✅ rate={rate} 测试通过')
    
    print()
    print('✅ 所有测试通过！')
except Exception as e:
    print(f'❌ 测试失败: {e}')
    import traceback
    traceback.print_exc()
    exit(1)
"

if [ $? -eq 0 ]; then
    echo ""
    echo "============================================================"
    echo "  ✅ 版本修复完成！"
    echo "============================================================"
    echo ""
    echo "现在可以使用以下命令启动服务："
    echo "  bash core/engine/scripts/start_yourtts_wsl.sh"
    echo ""
else
    echo ""
    echo "============================================================"
    echo "  ⚠️  版本修复完成，但测试失败"
    echo "============================================================"
    echo "请检查错误信息"
    exit 1
fi

