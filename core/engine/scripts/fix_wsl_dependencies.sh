#!/bin/bash
# 在 WSL 环境中修复 YourTTS 服务的依赖版本

# 获取脚本目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"

# 如果路径是 Windows 路径格式，转换为 WSL 路径
if [[ "$PROJECT_ROOT" == /mnt/* ]]; then
    # 已经是 WSL 路径
    :
elif [[ -d "/mnt/d/Programs/github/lingua" ]]; then
    PROJECT_ROOT="/mnt/d/Programs/github/lingua"
fi

# 切换到项目目录
cd "$PROJECT_ROOT" || {
    echo "❌ 错误: 无法切换到项目目录: $PROJECT_ROOT"
    exit 1
}

# 激活虚拟环境
if [ -d "venv-wsl" ]; then
    source venv-wsl/bin/activate
    echo "✅ 已激活虚拟环境: venv-wsl"
else
    echo "❌ 错误: 虚拟环境 venv-wsl 不存在"
    exit 1
fi

echo "============================================================"
echo "  修复 WSL 环境中的依赖库版本"
echo "============================================================"
echo ""

# 检查当前版本
echo "检查当前版本..."
python -c "import numpy; import numba; import librosa; print(f'numpy: {numpy.__version__}'); print(f'numba: {numba.__version__}'); print(f'librosa: {librosa.__version__}')" 2>/dev/null || echo "某些库未安装"

echo ""
echo "开始安装兼容版本..."

# 方案 1：先尝试 numpy 1.26.4
echo ""
echo "方案 1: 安装 numpy==1.26.4, numba==0.59.1, librosa==0.10.1"
pip install 'numpy==1.26.4' 'numba==0.59.1' 'librosa==0.10.1' --force-reinstall --no-cache-dir

if [ $? -eq 0 ]; then
    echo ""
    echo "验证安装..."
    python -c "
import numpy as np
import librosa
import numba
print(f'✅ numpy: {np.__version__}')
print(f'✅ numba: {numba.__version__}')
print(f'✅ librosa: {librosa.__version__}')

# 测试 librosa.effects.time_stretch
test_audio = np.random.randn(1000).astype(np.float64)
try:
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print('✅ librosa.effects.time_stretch 测试通过')
    exit(0)
except Exception as e:
    print(f'❌ librosa.effects.time_stretch 测试失败: {e}')
    exit(1)
"
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "============================================================"
        echo "  ✅ 修复成功！"
        echo "============================================================"
        exit 0
    else
        echo ""
        echo "方案 1 失败，尝试方案 2..."
        # 方案 2：使用 numpy 1.24.3
        echo ""
        echo "方案 2: 安装 numpy==1.24.3, numba==0.59.1, librosa==0.10.1"
        pip install 'numpy==1.24.3' 'numba==0.59.1' 'librosa==0.10.1' --force-reinstall --no-cache-dir
        
        if [ $? -eq 0 ]; then
            echo ""
            echo "验证安装..."
            python -c "
import numpy as np
import librosa
import numba
print(f'✅ numpy: {np.__version__}')
print(f'✅ numba: {numba.__version__}')
print(f'✅ librosa: {librosa.__version__}')

# 测试 librosa.effects.time_stretch
test_audio = np.random.randn(1000).astype(np.float64)
try:
    stretched = librosa.effects.time_stretch(test_audio, rate=1.0)
    print('✅ librosa.effects.time_stretch 测试通过')
    exit(0)
except Exception as e:
    print(f'❌ librosa.effects.time_stretch 测试失败: {e}')
    exit(1)
"
            
            if [ $? -eq 0 ]; then
                echo ""
                echo "============================================================"
                echo "  ✅ 修复成功（方案 2）！"
                echo "============================================================"
                exit 0
            fi
        fi
    fi
fi

echo ""
echo "============================================================"
echo "  ❌ 修复失败，请手动检查"
echo "============================================================"
exit 1

