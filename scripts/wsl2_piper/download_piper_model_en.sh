#!/bin/bash
# WSL2 内 Piper 英文模型下载脚本
# 用途：从官方源下载英文 Piper 模型

set -e

echo "=== Piper 英文模型下载 ==="
echo ""

# 模型配置
MODEL_NAME="en_US-lessac-medium"
MODEL_DIR="$HOME/piper_models/en"
BASE_URL="https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium"

# 创建模型目录
mkdir -p "$MODEL_DIR"
cd "$MODEL_DIR"

echo "[INFO] 模型目录: $MODEL_DIR"
echo "[INFO] 模型名称: $MODEL_NAME"
echo ""

# 下载模型文件
echo "[1/2] 下载模型文件..."
if [ ! -f "${MODEL_NAME}.onnx" ]; then
    echo "[INFO] 下载 ${MODEL_NAME}.onnx..."
    wget -q --show-progress "${BASE_URL}/${MODEL_NAME}.onnx" -O "${MODEL_NAME}.onnx"
    echo "[OK] 模型文件下载完成"
else
    echo "[INFO] 模型文件已存在，跳过下载"
fi

# 下载配置文件
echo ""
echo "[2/2] 下载配置文件..."
if [ ! -f "${MODEL_NAME}.onnx.json" ]; then
    echo "[INFO] 下载 ${MODEL_NAME}.onnx.json..."
    wget -q --show-progress "${BASE_URL}/${MODEL_NAME}.onnx.json" -O "${MODEL_NAME}.onnx.json"
    echo "[OK] 配置文件下载完成"
else
    echo "[INFO] 配置文件已存在，跳过下载"
fi

# 验证文件
echo ""
echo "[验证] 检查文件..."
if [ -f "${MODEL_NAME}.onnx" ] && [ -f "${MODEL_NAME}.onnx.json" ]; then
    ONNX_SIZE=$(du -h "${MODEL_NAME}.onnx" | cut -f1)
    JSON_SIZE=$(du -h "${MODEL_NAME}.onnx.json" | cut -f1)
    echo "[OK] 模型文件: ${MODEL_NAME}.onnx ($ONNX_SIZE)"
    echo "[OK] 配置文件: ${MODEL_NAME}.onnx.json ($JSON_SIZE)"
else
    echo "[ERROR] 文件下载不完整" >&2
    exit 1
fi

echo ""
echo "=== 模型下载完成 ==="
echo ""
echo "模型位置: $MODEL_DIR"
echo ""
echo "下一步："
echo "  1. 重启 Piper HTTP 服务（如果正在运行）"
echo "  2. 验证模型可用性: curl http://127.0.0.1:5005/voices"
echo "  3. 测试英文语音合成"
echo ""

