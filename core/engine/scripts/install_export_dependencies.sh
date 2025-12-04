#!/bin/bash
# 在 WSL 中安装 ONNX 导出所需的依赖

echo "============================================================"
echo "  安装 YourTTS ONNX 导出依赖"
echo "============================================================"
echo ""

# 检查 Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 未找到"
    exit 1
fi

echo "Python 版本: $(python3 --version)"
echo "Python 路径: $(which python3)"
echo ""

# 更新 pip
echo "📦 更新 pip..."
python3 -m pip install --upgrade pip
echo ""

# 安装依赖
echo "📦 安装依赖包..."
echo ""

dependencies=("torch" "onnx" "onnxruntime" "TTS")

for dep in "${dependencies[@]}"; do
    echo "检查 $dep..."
    python3 -c "import $dep" 2>/dev/null
    if [ $? -eq 0 ]; then
        echo "  ✅ $dep 已安装"
    else
        echo "  ⚠️  $dep 未安装，正在安装..."
        python3 -m pip install "$dep"
        if [ $? -eq 0 ]; then
            echo "  ✅ $dep 安装成功"
        else
            echo "  ❌ $dep 安装失败"
            exit 1
        fi
    fi
    echo ""
done

echo "============================================================"
echo "✅ 所有依赖安装完成！"
echo "============================================================"
echo ""
echo "现在可以运行导出脚本："
echo "  python3 core/engine/scripts/export_yourtts_to_onnx.py"
echo ""

