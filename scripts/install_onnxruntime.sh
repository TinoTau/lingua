#!/bin/bash
# 安装 onnxruntime 的 Bash 脚本
# 
# 使用方法:
#   bash scripts/install_onnxruntime.sh
#   或者指定 Python 解释器:
#   bash scripts/install_onnxruntime.sh python3.10

PYTHON_EXE="${1:-python}"
PACKAGE="onnxruntime"

echo "=== 安装 $PACKAGE ==="
echo "Python 解释器: $PYTHON_EXE"
echo ""

# 检查 Python 是否可用
echo "检查 Python 版本..."
if ! $PYTHON_EXE --version; then
    echo "❌ 错误: 无法运行 $PYTHON_EXE"
    echo "请确保 Python 已安装并在 PATH 中"
    exit 1
fi
echo ""

# 检查 pip 是否可用
echo "检查 pip 是否可用..."
if ! $PYTHON_EXE -m pip --version; then
    echo "❌ 错误: pip 不可用"
    echo "请先安装 pip"
    exit 1
fi
echo ""

# 升级 pip
echo "升级 pip..."
$PYTHON_EXE -m pip install --upgrade pip --quiet
echo ""

# 安装 onnxruntime
echo "安装 $PACKAGE..."
echo "这可能需要几分钟时间，请耐心等待..."
echo ""

$PYTHON_EXE -m pip install $PACKAGE

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ $PACKAGE 安装成功！"
    echo ""
    
    # 验证安装
    echo "验证安装..."
    if $PYTHON_EXE -c "import onnxruntime; print(f'onnxruntime version: {onnxruntime.__version__}')"; then
        echo "✅ 安装验证成功"
    else
        echo "⚠️  警告: 无法导入 onnxruntime"
    fi
else
    echo ""
    echo "❌ 安装失败"
    echo "请检查错误信息并重试"
    exit 1
fi

