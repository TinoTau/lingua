#!/bin/bash
# 安装 Python 3.10 并创建新的虚拟环境

set -e  # 遇到错误立即退出

echo "============================================================"
echo "  安装 Python 3.10 并创建新虚拟环境"
echo "============================================================"
echo ""

cd /mnt/d/Programs/github/lingua

# 1. 检查是否已有 Python 3.10
echo "1. 检查 Python 3.10 是否已安装..."
PYTHON310_CMD=""

# 先检查完整路径
if [ -f "/usr/bin/python3.10" ]; then
    PYTHON_VERSION=$(/usr/bin/python3.10 --version 2>&1)
    if echo "$PYTHON_VERSION" | grep -q "3.10"; then
        echo "   ✅ Python 3.10 已安装: $PYTHON_VERSION"
        PYTHON310_CMD="/usr/bin/python3.10"
        INSTALL_PYTHON=false
    else
        echo "   ⚠️  /usr/bin/python3.10 版本不对: $PYTHON_VERSION"
        INSTALL_PYTHON=true
    fi
elif command -v python3.10 &> /dev/null; then
    PYTHON_VERSION=$(python3.10 --version 2>&1)
    if echo "$PYTHON_VERSION" | grep -q "3.10"; then
        echo "   ✅ Python 3.10 已安装: $PYTHON_VERSION"
        PYTHON310_CMD=$(which python3.10)
        INSTALL_PYTHON=false
    else
        echo "   ⚠️  python3.10 命令存在但版本不对: $PYTHON_VERSION"
        echo "   需要安装真正的 Python 3.10"
        INSTALL_PYTHON=true
    fi
else
    echo "   ❌ Python 3.10 未安装"
    INSTALL_PYTHON=true
fi

# 2. 安装 Python 3.10（如果需要）
if [ "$INSTALL_PYTHON" = true ]; then
    echo ""
    echo "2. 安装 Python 3.10..."
    echo "   这需要 sudo 权限，可能需要输入密码"
    echo ""
    
    sudo apt update
    sudo apt install -y software-properties-common
    sudo add-apt-repository -y ppa:deadsnakes/ppa
    sudo apt update
    sudo apt install -y python3.10 python3.10-venv python3.10-dev
    
    echo ""
    echo "   验证安装..."
    # 使用完整路径验证
    if [ -f "/usr/bin/python3.10" ]; then
        PYTHON310_VER=$(/usr/bin/python3.10 --version 2>&1)
        echo "   完整路径 /usr/bin/python3.10: $PYTHON310_VER"
        if echo "$PYTHON310_VER" | grep -q "3.10"; then
            echo "   ✅ Python 3.10 安装成功: $PYTHON310_VER"
            PYTHON310_CMD="/usr/bin/python3.10"
        else
            echo "   ⚠️  完整路径版本也不对，继续尝试..."
            PYTHON310_CMD="/usr/bin/python3.10"
        fi
    else
        echo "   ❌ /usr/bin/python3.10 不存在"
        exit 1
    fi
fi

# 3. 创建新的虚拟环境
echo ""
echo "3. 创建 Python 3.10 虚拟环境..."
if [ -d "venv-wsl-py310" ]; then
    echo "   ⚠️  venv-wsl-py310 已存在"
    read -p "   是否删除并重新创建? (y/n): " answer
    if [ "$answer" = "y" ]; then
        rm -rf venv-wsl-py310
        echo "   ✅ 已删除旧环境"
    else
        echo "   ⚠️  跳过创建，使用现有环境"
    fi
fi

if [ ! -d "venv-wsl-py310" ]; then
    # 使用完整路径创建虚拟环境
    if [ -z "$PYTHON310_CMD" ]; then
        # 如果没有设置，尝试查找
        if [ -f "/usr/bin/python3.10" ]; then
            PYTHON310_CMD="/usr/bin/python3.10"
        elif command -v python3.10 &> /dev/null; then
            PYTHON310_CMD=$(which python3.10)
        else
            echo "   ❌ 找不到 Python 3.10"
            exit 1
        fi
    fi
    
    echo "   使用: $PYTHON310_CMD"
    $PYTHON310_CMD --version
    $PYTHON310_CMD -m venv venv-wsl-py310
    echo "   ✅ 虚拟环境创建成功: venv-wsl-py310"
fi

# 4. 激活环境并安装依赖
echo ""
echo "4. 激活虚拟环境并安装依赖..."
source venv-wsl-py310/bin/activate

# 验证 Python 版本
PYTHON_VER=$(python --version 2>&1)
if ! echo "$PYTHON_VER" | grep -q "3.10"; then
    echo "   ❌ 虚拟环境 Python 版本错误: $PYTHON_VER"
    exit 1
fi
echo "   ✅ Python 版本: $PYTHON_VER"

# 升级 pip
echo ""
echo "5. 升级 pip..."
pip install --upgrade pip --no-cache-dir

# 安装依赖
echo ""
echo "6. 安装依赖库..."
echo "   - numpy==1.24.3"
echo "   - numba==0.59.1"
echo "   - librosa==0.10.1"
echo "   - llvmlite==0.42.0"
echo ""

pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' 'llvmlite==0.42.0' --no-cache-dir

if [ $? -ne 0 ]; then
    echo ""
    echo "   ❌ 依赖安装失败"
    exit 1
fi

# 7. 验证安装
echo ""
echo "7. 验证安装..."
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
    print(f'✅ librosa.effects.time_stretch 测试通过')
    print(f'   输入长度: {len(test_audio)}, 输出长度: {len(stretched)}')
    
    # 测试不同的 rate
    print()
    print('测试不同的 rate 值...')
    for rate in [0.8, 1.2]:
        stretched = librosa.effects.time_stretch(test_audio, rate=rate)
        print(f'  ✅ rate={rate}: {len(test_audio)} -> {len(stretched)} samples')
    
    print()
    print('============================================================')
    print('  ✅ 所有测试通过！')
    print('============================================================')
    exit(0)
except Exception as e:
    print(f'❌ 测试失败: {e}')
    import traceback
    traceback.print_exc()
    exit(1)
"

if [ $? -eq 0 ]; then
    echo ""
    echo "============================================================"
    echo "  ✅ 环境设置完成！"
    echo "============================================================"
    echo ""
    echo "下一步："
    echo "1. 修改启动脚本使用新环境："
    echo "   编辑 core/engine/scripts/start_yourtts_wsl.sh"
    echo "   将: source venv-wsl/bin/activate"
    echo "   改为: source venv-wsl-py310/bin/activate"
    echo ""
    echo "2. 或者使用新启动脚本："
    echo "   bash core/engine/scripts/start_yourtts_wsl_py310.sh"
    echo ""
else
    echo ""
    echo "============================================================"
    echo "  ❌ 验证失败，请检查错误信息"
    echo "============================================================"
    exit 1
fi

