#!/bin/bash
# 查找 Python 3.10 环境

echo "============================================================"
echo "  查找 Python 3.10 环境"
echo "============================================================"
echo ""

echo "1. 检查系统中可用的 Python 3.10："
echo ""

# 检查常见的 Python 3.10 路径
PYTHON310_PATHS=(
    "/usr/bin/python3.10"
    "/usr/local/bin/python3.10"
    "$HOME/.local/bin/python3.10"
    "/usr/bin/python3.1"
)

FOUND=false

for path in "${PYTHON310_PATHS[@]}"; do
    if [ -f "$path" ]; then
        echo "✅ 找到: $path"
        $path --version 2>&1
        FOUND=true
    fi
done

# 使用 which/whereis 查找
echo ""
echo "2. 使用系统命令查找："
echo ""

if command -v whereis &> /dev/null; then
    WHEREIS_RESULT=$(whereis python3.10 2>/dev/null)
    if [ -n "$WHEREIS_RESULT" ] && [ "$WHEREIS_RESULT" != "python3.10:" ]; then
        echo "whereis python3.10:"
        echo "$WHEREIS_RESULT"
        FOUND=true
    fi
fi

# 查找所有 python3.x
echo ""
echo "3. 查找所有 Python 3.x 可执行文件："
echo ""

if command -v find &> /dev/null; then
    echo "在 /usr/bin 中查找："
    find /usr/bin -name "python3.*" -type f -executable 2>/dev/null | head -10
    echo ""
    echo "在 /usr/local/bin 中查找："
    find /usr/local/bin -name "python3.*" -type f -executable 2>/dev/null | head -10
fi

# 查找虚拟环境
echo ""
echo "4. 查找可能的 Python 3.10 虚拟环境："
echo ""

# 在当前项目目录查找
if [ -d "/mnt/d/Programs/github/lingua" ]; then
    cd /mnt/d/Programs/github/lingua
    echo "在当前项目目录查找："
    find . -maxdepth 2 -name "venv*" -type d 2>/dev/null | while read venv_dir; do
        if [ -f "$venv_dir/bin/python" ]; then
            PYTHON_VERSION=$($venv_dir/bin/python --version 2>&1)
            echo "  $venv_dir: $PYTHON_VERSION"
        fi
    done
fi

# 在用户目录查找
if [ -d "$HOME" ]; then
    echo ""
    echo "在用户目录查找虚拟环境："
    find "$HOME" -maxdepth 3 -name "venv*" -o -name ".venv*" -type d 2>/dev/null | head -10 | while read venv_dir; do
        if [ -f "$venv_dir/bin/python" ]; then
            PYTHON_VERSION=$($venv_dir/bin/python --version 2>&1)
            echo "  $venv_dir: $PYTHON_VERSION"
        fi
    done
fi

# 查找 conda 环境
echo ""
echo "5. 查找 conda 环境（如果已安装）："
echo ""

if [ -d "$HOME/anaconda3" ] || [ -d "$HOME/miniconda3" ]; then
    CONDA_ROOT="$HOME/anaconda3"
    [ -d "$HOME/miniconda3" ] && CONDA_ROOT="$HOME/miniconda3"
    
    if [ -d "$CONDA_ROOT/envs" ]; then
        echo "Conda 环境："
        ls -la "$CONDA_ROOT/envs" 2>/dev/null | grep python | head -10
    fi
fi

# 使用 pyenv（如果安装）
echo ""
echo "6. 检查 pyenv（如果已安装）："
echo ""

if command -v pyenv &> /dev/null; then
    echo "pyenv versions:"
    pyenv versions 2>/dev/null | grep "3.10" || echo "  未找到 Python 3.10"
fi

echo ""
echo "============================================================"
echo "  查找完成"
echo "============================================================"
echo ""

# 尝试直接调用 python3.10
echo "7. 直接测试 python3.10 命令："
echo ""

if command -v python3.10 &> /dev/null; then
    PYTHON310_PATH=$(which python3.10)
    PYTHON310_VERSION=$(python3.10 --version 2>&1)
    echo "✅ python3.10 可用："
    echo "   路径: $PYTHON310_PATH"
    echo "   版本: $PYTHON310_VERSION"
    echo ""
    echo "可以使用以下命令创建 Python 3.10 虚拟环境："
    echo "  python3.10 -m venv venv-wsl-py310"
else
    echo "❌ python3.10 命令不可用"
    echo ""
    echo "需要安装 Python 3.10："
    echo "  sudo apt update"
    echo "  sudo apt install python3.10 python3.10-venv python3.10-dev"
fi

