#!/bin/bash
# 检查是否有真正的 Python 3.10（不是符号链接）

echo "============================================================"
echo "  检查真正的 Python 3.10"
echo "============================================================"
echo ""

echo "1. 检查 python3.10 的实际位置和类型："
if command -v python3.10 &> /dev/null; then
    PYTHON310_PATH=$(which python3.10)
    echo "   路径: $PYTHON310_PATH"
    
    # 检查是否是符号链接
    if [ -L "$PYTHON310_PATH" ]; then
        echo "   ⚠️  这是符号链接，指向："
        ls -l "$PYTHON310_PATH"
        REAL_PATH=$(readlink -f "$PYTHON310_PATH")
        echo "   实际文件: $REAL_PATH"
        "$REAL_PATH" --version 2>&1
    else
        echo "   ✅ 这是实际文件（非符号链接）"
        "$PYTHON310_PATH" --version 2>&1
    fi
else
    echo "   ❌ python3.10 命令不可用"
fi

echo ""
echo "2. 检查系统 Python 版本："
echo ""

# 检查 /usr/bin/python3.x
for py in python3.8 python3.9 python3.10 python3.11 python3.12; do
    if [ -f "/usr/bin/$py" ]; then
        echo "   /usr/bin/$py:"
        if [ -L "/usr/bin/$py" ]; then
            echo "      符号链接 -> $(readlink /usr/bin/$py)"
        else
            /usr/bin/$py --version 2>&1 | sed 's/^/      /'
        fi
    fi
done

echo ""
echo "3. 尝试查找所有 Python 可执行文件："
echo ""

find /usr/bin -name "python3.*" -type f -executable 2>/dev/null | while read py; do
    VERSION=$($py --version 2>&1)
    if echo "$VERSION" | grep -q "3.10"; then
        echo "   ✅ 找到 Python 3.10: $py"
    fi
done

echo ""
echo "4. 检查是否需要安装 Python 3.10："
echo ""

# 尝试检查 apt 包
if command -v apt-cache &> /dev/null; then
    echo "   检查可用的 Python 3.10 包："
    apt-cache search python3.10 | grep "^python3.10" | head -5
fi

echo ""
echo "============================================================"
echo "  结论"
echo "============================================================"
echo ""

if command -v python3.10 &> /dev/null; then
    ACTUAL_VERSION=$(python3.10 --version 2>&1)
    if echo "$ACTUAL_VERSION" | grep -q "3.10"; then
        echo "✅ 系统有真正的 Python 3.10"
        echo "   可以使用: python3.10 -m venv venv-wsl-py310"
    else
        echo "❌ python3.10 指向的是 Python 3.12"
        echo ""
        echo "需要安装真正的 Python 3.10："
        echo "  sudo apt update"
        echo "  sudo apt install software-properties-common"
        echo "  sudo add-apt-repository ppa:deadsnakes/ppa"
        echo "  sudo apt update"
        echo "  sudo apt install python3.10 python3.10-venv python3.10-dev"
    fi
else
    echo "❌ 系统没有 Python 3.10"
    echo ""
    echo "需要安装："
    echo "  sudo apt update"
    echo "  sudo apt install software-properties-common"
    echo "  sudo add-apt-repository ppa:deadsnakes/ppa"
    echo "  sudo apt update"
    echo "  sudo apt install python3.10 python3.10-venv python3.10-dev"
fi

