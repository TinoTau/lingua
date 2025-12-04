#!/bin/bash
# 修复 numba 编译缓存问题

cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate

echo "============================================================"
echo "  修复 numba 编译缓存问题"
echo "============================================================"
echo ""

# 检查当前版本
echo "检查当前版本..."
python -c "import numpy; import numba; import librosa; print(f'numpy: {numpy.__version__}'); print(f'numba: {numba.__version__}'); print(f'librosa: {librosa.__version__}')"

echo ""
echo "1. 清除 numba 编译缓存..."
rm -rf ~/.cache/numba
echo "✅ numba 缓存已清除"

echo ""
echo "2. 重新安装 numba 和 llvmlite..."
pip uninstall -y numba llvmlite
pip install 'numba==0.59.1' 'llvmlite==0.42.0' --no-cache-dir --force-reinstall

echo ""
echo "3. 如果仍然失败，尝试降级到更稳定的版本组合..."
echo "   方案 2: numpy 1.24.3 + numba 0.59.1 + librosa 0.10.1"
read -p "是否尝试方案 2? (y/n): " answer
if [ "$answer" = "y" ]; then
    pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' --no-cache-dir --force-reinstall
fi

echo ""
echo "4. 验证安装..."
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
    echo "============================================================"
    echo "  ✅ 修复成功！请重启 YourTTS 服务"
    echo "============================================================"
else
    echo ""
    echo "============================================================"
    echo "  ⚠️  修复失败，可能需要手动调试"
    echo "============================================================"
fi

