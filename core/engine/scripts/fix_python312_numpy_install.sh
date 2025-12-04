#!/bin/bash
# 修复 Python 3.12 环境下安装 numpy 的问题

cd /mnt/d/Programs/github/lingua
source venv-wsl/bin/activate

echo "============================================================"
echo "  修复 Python 3.12 环境下 numpy 安装问题"
echo "============================================================"
echo ""

# 检查 Python 版本
PYTHON_VERSION=$(python --version 2>&1 | awk '{print $2}' | cut -d. -f1,2)
echo "检测到 Python 版本: $PYTHON_VERSION"

if [[ "$PYTHON_VERSION" == "3.12" ]]; then
    echo ""
    echo "⚠️  检测到 Python 3.12，numpy 1.24.3 不支持从源码编译"
    echo "   方案 1: 使用预编译的 wheel 包（如果有）"
    echo "   方案 2: 升级到支持 Python 3.12 的 numpy 版本"
    echo "   方案 3: 使用 Python 3.10 虚拟环境"
    echo ""
    
    echo "尝试方案 2: 升级到支持 Python 3.12 的版本组合..."
    echo ""
    
    # 清除缓存
    echo "1. 清除缓存..."
    rm -rf ~/.cache/numba ~/.cache/pip
    echo "✅ 缓存已清除"
    echo ""
    
    # 卸载现有版本
    echo "2. 卸载现有版本..."
    pip uninstall -y numpy numba librosa llvmlite 2>/dev/null || true
    echo "✅ 已卸载"
    echo ""
    
    # 安装支持 Python 3.12 的版本
    # numpy 1.26.x 支持 Python 3.12，但我们需要找到与 numba 0.59.1 兼容的版本
    echo "3. 安装兼容 Python 3.12 的版本..."
    echo "   - numpy: 1.26.4 (支持 Python 3.12)"
    echo "   - numba: 0.59.1"
    echo "   - librosa: 0.10.1"
    echo ""
    
    pip install 'numpy==1.26.4' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir --force-reinstall
    
    if [ $? -eq 0 ]; then
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
    print()
    print('============================================================')
    print('  ✅ 安装成功！')
    print('============================================================')
except Exception as e:
    print(f'⚠️  librosa 测试失败: {e}')
    print('   但已安装，代码会自动 fallback 到 scipy')
    print()
    print('============================================================')
    print('  ⚠️  安装完成，但 librosa 可能仍有兼容性问题')
    print('  代码已配置自动 fallback 到 scipy，服务仍可使用')
    print('============================================================')
"
    else
        echo ""
        echo "❌ 安装失败"
        echo ""
        echo "建议：使用 Python 3.10 虚拟环境"
        echo "1. 创建新的 Python 3.10 虚拟环境："
        echo "   python3.10 -m venv venv-wsl-py310"
        echo "2. 激活新环境："
        echo "   source venv-wsl-py310/bin/activate"
        echo "3. 然后重新运行安装脚本"
        exit 1
    fi
else
    echo "Python 版本不是 3.12，继续使用原来的安装脚本..."
    echo ""
    # 如果是 Python 3.10，使用原来的命令
    pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir --force-reinstall
fi

