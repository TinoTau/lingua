#!/bin/bash
# 在 Python 3.10 环境中安装 TTS 和 YourTTS 服务的所有依赖

set -e  # 遇到错误立即退出

cd /mnt/d/Programs/github/lingua

echo "============================================================"
echo "  安装 TTS 和 YourTTS 服务依赖（Python 3.10）"
echo "============================================================"
echo ""

# 激活虚拟环境
if [ -d "venv-wsl-py310" ]; then
    source venv-wsl-py310/bin/activate
    echo "✅ 已激活虚拟环境: venv-wsl-py310"
    
    # 验证 Python 版本
    PYTHON_VER=$(python --version 2>&1)
    if echo "$PYTHON_VER" | grep -q "3.10"; then
        echo "✅ Python 版本: $PYTHON_VER"
    else
        echo "❌ 错误: Python 版本不是 3.10: $PYTHON_VER"
        echo "   请确保使用正确的虚拟环境"
        exit 1
    fi
else
    echo "❌ 错误: 虚拟环境 venv-wsl-py310 不存在"
    echo "   请先运行: bash core/engine/scripts/setup_python310_env.sh"
    exit 1
fi

echo ""
echo "============================================================"
echo "  1. 已安装的核心依赖（已配置版本）"
echo "============================================================"
echo "  这些依赖已在 setup_python310_env.sh 中安装："
echo "  - numpy==1.24.3"
echo "  - numba==0.59.1"
echo "  - librosa==0.10.1"
echo "  - llvmlite==0.42.0"
echo "  - scipy (自动安装的依赖)"
echo ""

echo "============================================================"
echo "  2. 安装 PyTorch 和相关依赖"
echo "============================================================"
echo ""

# 检查 CUDA 是否可用
USE_CUDA=false
if command -v nvidia-smi &> /dev/null; then
    if nvidia-smi &> /dev/null; then
        CUDA_VERSION=$(nvidia-smi | grep "CUDA Version" | awk '{print $9}' | cut -d. -f1,2 || echo "")
        if [ -n "$CUDA_VERSION" ]; then
            echo "✅ 检测到 CUDA: $CUDA_VERSION"
            USE_CUDA=true
        fi
    fi
fi

if [ "$USE_CUDA" = true ]; then
    echo "安装 PyTorch (CUDA 支持)..."
    echo "  使用 PyTorch 官方索引安装 CUDA 版本"
    pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121 --no-cache-dir
else
    echo "安装 PyTorch (CPU 版本)..."
    pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu --no-cache-dir
fi

echo ""
echo "============================================================"
echo "  3. 安装基础依赖"
echo "============================================================"
echo ""

pip install --upgrade pip --no-cache-dir

# 基础依赖
echo "安装 Flask 和相关依赖..."
pip install flask flask-cors --no-cache-dir

echo ""
echo "安装音频处理库..."
pip install soundfile --no-cache-dir

echo ""
echo "============================================================"
echo "  4. 安装 TTS 库 (Coqui TTS)"
echo "============================================================"
echo ""

echo "安装 TTS 库（这可能需要一些时间）..."
pip install TTS --no-cache-dir

echo ""
echo "============================================================"
echo "  5. 安装 Speaker Embedding 依赖"
echo "============================================================"
echo ""

echo "安装 SpeechBrain..."
pip install speechbrain --no-cache-dir

echo ""
echo "============================================================"
echo "  6. 安装其他可选依赖"
echo "============================================================"
echo ""

# 其他可能有用的依赖
pip install requests pillow --no-cache-dir

echo ""
echo "============================================================"
echo "  7. 验证安装"
echo "============================================================"
echo ""

python -c "
import sys

print('检查核心依赖...')
dependencies = {
    'numpy': 'numpy',
    'numba': 'numba',
    'librosa': 'librosa',
    'scipy': 'scipy',
    'flask': 'flask',
    'torch': 'torch',
    'soundfile': 'soundfile',
    'TTS': 'TTS',
    'speechbrain': 'speechbrain',
}

missing = []
installed = []

for pkg, import_name in dependencies.items():
    try:
        mod = __import__(import_name)
        version = getattr(mod, '__version__', 'unknown')
        print(f'  ✅ {pkg}: {version}')
        installed.append(pkg)
    except ImportError:
        print(f'  ❌ {pkg}: 未安装')
        missing.append(pkg)

print()
print('检查 PyTorch 配置...')
try:
    import torch
    print(f'  ✅ PyTorch: {torch.__version__}')
    if torch.cuda.is_available():
        print(f'  ✅ CUDA 可用: {torch.cuda.get_device_name(0)}')
        print(f'  ✅ CUDA 版本: {torch.version.cuda}')
    else:
        print('  ⚠️  CUDA 不可用（使用 CPU）')
except Exception as e:
    print(f'  ❌ PyTorch 检查失败: {e}')

print()
print('检查 torchaudio...')
try:
    import torchaudio
    print(f'  ✅ torchaudio: {torchaudio.__version__}')
except ImportError:
    print('  ❌ torchaudio: 未安装')
    missing.append('torchaudio')

print()
if missing:
    print(f'❌ 缺少依赖: {missing}')
    sys.exit(1)
else:
    print('✅ 所有依赖已安装！')
    sys.exit(0)
"

if [ $? -eq 0 ]; then
    echo ""
    echo "============================================================"
    echo "  ✅ 依赖安装完成！"
    echo "============================================================"
    echo ""
    echo "下一步："
    echo "1. 启动 YourTTS 服务："
    echo "   bash core/engine/scripts/start_yourtts_wsl.sh"
    echo ""
    echo "2. 启动 Speaker Embedding 服务："
    echo "   source venv-wsl-py310/bin/activate"
    echo "   python core/engine/scripts/speaker_embedding_service.py --gpu --port 5003 --host 0.0.0.0"
    echo ""
    echo "3. 或启动所有服务："
    echo "   bash core/engine/scripts/start_all_tts_wsl.sh"
    echo ""
else
    echo ""
    echo "============================================================"
    echo "  ⚠️  安装完成，但部分依赖可能有问题"
    echo "============================================================"
    echo "请检查上面的错误信息"
    exit 1
fi

