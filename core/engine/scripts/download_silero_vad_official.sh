#!/bin/bash
# 下载官方 Silero VAD 模型（WSL/Linux 版本）
# 使用方法：在 WSL 或 Linux 中运行此脚本

set -e

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORE_ENGINE_DIR="$(dirname "$SCRIPT_DIR")"
MODEL_DIR="$CORE_ENGINE_DIR/models/vad/silero"

# 创建模型目录（如果不存在）
mkdir -p "$MODEL_DIR"

# 模型文件路径
MODEL_PATH="$MODEL_DIR/silero_vad_official.onnx"
BACKUP_PATH="$MODEL_DIR/silero_vad.onnx.backup"

# 备份现有模型（如果存在）
if [ -f "$MODEL_DIR/silero_vad.onnx" ]; then
    echo "备份现有模型..."
    cp "$MODEL_DIR/silero_vad.onnx" "$BACKUP_PATH"
    echo "已备份到: $BACKUP_PATH"
fi

# 官方模型下载地址（使用 Hugging Face）
# 主地址（可能需要认证）：
# - https://huggingface.co/snakers4/silero-vad/resolve/main/models/silero_vad.onnx
# 备用地址（已验证可用）：
# - https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx
MODEL_URL="https://huggingface.co/Derur/silero-models/resolve/main/vad/silero_vad.onnx"

echo "============================================================"
echo "  下载官方 Silero VAD 模型"
echo "============================================================"
echo ""
echo "下载地址: $MODEL_URL"
echo "保存路径: $MODEL_PATH"
echo ""

# 检查 wget 是否可用
if command -v wget &> /dev/null; then
    echo "使用 wget 下载..."
    wget "$MODEL_URL" -O "$MODEL_PATH"
elif command -v curl &> /dev/null; then
    echo "使用 curl 下载..."
    curl -L -o "$MODEL_PATH" "$MODEL_URL"
else
    echo "错误: 未找到 wget 或 curl 命令" >&2
    exit 1
fi

# 验证文件
if [ -f "$MODEL_PATH" ]; then
    FILE_SIZE=$(du -h "$MODEL_PATH" | cut -f1)
    echo ""
    echo "✓ 下载成功！"
    echo "  文件大小: $FILE_SIZE"
    echo "  文件路径: $MODEL_PATH"
    echo ""
    echo "提示: 请更新配置文件中的模型路径为: models/vad/silero/silero_vad_official.onnx"
else
    echo "✗ 下载失败：文件不存在" >&2
    exit 1
fi

echo "============================================================"

