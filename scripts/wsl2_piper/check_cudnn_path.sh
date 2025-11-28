#!/bin/bash
# 检查 Windows 路径下的 cuDNN 文件结构

WINDOWS_CUDNN_PATH="/mnt/c/Program Files/NVIDIA/CUDNN/v9.1"

echo "检查路径: $WINDOWS_CUDNN_PATH"
echo ""

if [ ! -d "$WINDOWS_CUDNN_PATH" ]; then
    echo "❌ 路径不存在: $WINDOWS_CUDNN_PATH"
    echo ""
    echo "请检查 cuDNN 的实际安装路径。"
    exit 1
fi

echo "✓ 路径存在"
echo ""
echo "目录结构："
ls -la "$WINDOWS_CUDNN_PATH" 2>/dev/null | head -20
echo ""

echo "查找头文件："
find "$WINDOWS_CUDNN_PATH" -name "cudnn*.h" 2>/dev/null
echo ""

echo "查找库文件："
find "$WINDOWS_CUDNN_PATH" -name "libcudnn.so*" 2>/dev/null
echo ""

echo "查找所有 .so 文件："
find "$WINDOWS_CUDNN_PATH" -name "*.so*" 2>/dev/null | head -10
echo ""

echo "查找所有 .dll 文件（Windows 库）："
find "$WINDOWS_CUDNN_PATH" -name "*.dll" 2>/dev/null | head -10

