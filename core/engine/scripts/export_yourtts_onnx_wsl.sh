#!/bin/bash
# 在 WSL 中导出 YourTTS 模型为 ONNX 格式

# 获取脚本目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"

# 切换到项目目录
cd "$PROJECT_ROOT"

echo "============================================================"
echo "  YourTTS ONNX 导出工具（WSL 环境）"
echo "============================================================"
echo "项目根目录: $PROJECT_ROOT"
echo ""

# 检查是否在 WSL 环境中
if [ -z "$WSL_DISTRO_NAME" ] && [ -z "$WSLENV" ]; then
    echo "⚠️  警告: 未检测到 WSL 环境"
    echo "   建议在 WSL 中运行此脚本"
    echo ""
fi

# 检查 Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3 未找到"
    exit 1
fi

echo "Python 版本: $(python3 --version)"
echo "Python 路径: $(which python3)"
echo ""

# 检查依赖
echo "📌 检查依赖..."
python3 -c "import TTS" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "⚠️  TTS 库未安装，尝试安装..."
    python3 -m pip install TTS
fi

python3 -c "import torch" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "⚠️  torch 未安装，尝试安装..."
    python3 -m pip install torch
fi

python3 -c "import onnx" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "⚠️  onnx 未安装，尝试安装..."
    python3 -m pip install onnx
fi

python3 -c "import onnxruntime" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "⚠️  onnxruntime 未安装，尝试安装..."
    python3 -m pip install onnxruntime
fi

echo "✅ 依赖检查完成"
echo ""

# 运行导出脚本
echo "🚀 开始导出 YourTTS 模型为 ONNX..."
echo ""

python3 core/engine/scripts/export_yourtts_to_onnx.py "$@"

EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo ""
    echo "============================================================"
    echo "✅ 导出完成！"
    echo "============================================================"
else
    echo ""
    echo "============================================================"
    echo "❌ 导出失败"
    echo "============================================================"
fi

exit $EXIT_CODE

