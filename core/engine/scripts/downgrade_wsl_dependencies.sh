#!/bin/bash
# 在 WSL 环境中降级到更稳定的依赖版本

cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate

echo "============================================================"
echo "  降级到更稳定的依赖版本"
echo "============================================================"
echo ""

# 检查当前版本
echo "当前版本："
python -c "import numpy; import numba; import librosa; print(f'numpy: {numpy.__version__}'); print(f'numba: {numba.__version__}'); print(f'librosa: {librosa.__version__}')" 2>/dev/null || echo "某些库未安装"

echo ""
echo "开始降级到稳定版本组合..."
echo "  - numpy: 1.24.3"
echo "  - numba: 0.59.1"
echo "  - librosa: 0.10.1"
echo ""

# 清除 numba 缓存
echo "1. 清除 numba 编译缓存..."
rm -rf ~/.cache/numba
echo "✅ numba 缓存已清除"
echo ""

# 卸载现有版本
echo "2. 卸载现有版本..."
pip uninstall -y numpy numba librosa llvmlite 2>/dev/null || true
echo "✅ 已卸载"
echo ""

# 安装稳定版本
echo "3. 安装稳定版本组合..."
pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir --force-reinstall

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ 安装失败，请检查错误信息"
    exit 1
fi

echo ""
echo "4. 验证安装..."
python -c "
import numpy as np
import numba
import librosa

print(f'✅ numpy: {np.__version__}')
print(f'✅ numba: {numba.__version__}')
print(f'✅ librosa: {librosa.__version__}')
print()

# 测试 librosa.effects.time_stretch
print('测试 librosa.effects.time_stretch...')
test_audio = np.random.randn(1000).astype(np.float64)
try:
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print('✅ librosa.effects.time_stretch 测试通过')
    print(f'   输入长度: {len(test_audio)}, 输出长度: {len(stretched)}')
    
    # 测试不同的 rate
    print()
    print('测试不同的 rate 值...')
    for rate in [0.8, 1.2]:
        stretched = librosa.effects.time_stretch(test_audio, rate=rate)
        print(f'  ✅ rate={rate}: {len(test_audio)} -> {len(stretched)} samples')
    
    print()
    print('============================================================')
    print('  ✅ 降级成功！所有测试通过')
    print('============================================================')
except Exception as e:
    print(f'❌ 测试失败: {e}')
    import traceback
    traceback.print_exc()
    exit(1)
"

if [ $? -eq 0 ]; then
    echo ""
    echo "============================================================"
    echo "  ✅ 降级完成！"
    echo "============================================================"
    echo ""
    echo "下一步：重启 YourTTS 服务"
    echo ""
else
    echo ""
    echo "============================================================"
    echo "  ⚠️  降级完成，但测试失败"
    echo "============================================================"
    echo "请检查错误信息"
fi

