#!/bin/bash
# 从 Windows 路径复制 cuDNN 到 WSL2 CUDA 目录

set -e

CUDNN_WIN="/mnt/c/Program Files/NVIDIA/CUDNN/v9.1"
CUDA_PATH="/usr/local/cuda-12.4"

echo "=== 从 Windows 路径复制 cuDNN 到 WSL2 ==="
echo ""
echo "源路径: $CUDNN_WIN"
echo "目标路径: $CUDA_PATH"
echo ""

# 检查源路径是否存在
if [ ! -d "$CUDNN_WIN" ]; then
    echo "❌ 错误: Windows cuDNN 路径不存在: $CUDNN_WIN"
    echo "请检查 cuDNN 的实际安装路径"
    exit 1
fi

# 检查目标路径是否存在
if [ ! -d "$CUDA_PATH" ]; then
    echo "❌ 错误: CUDA 路径不存在: $CUDA_PATH"
    echo "请先安装 CUDA 12.4"
    exit 1
fi

echo "✓ 路径检查通过"
echo ""

# 查找并复制头文件
echo "查找头文件..."
HEADER_FILES=$(find "$CUDNN_WIN" -name "cudnn*.h" 2>/dev/null)

if [ -z "$HEADER_FILES" ]; then
    echo "⚠️  警告: 未找到 cudnn*.h 头文件"
    echo "请检查 cuDNN 安装是否完整"
else
    echo "找到以下头文件："
    echo "$HEADER_FILES" | while read -r file; do
        echo "  - $file"
    done
    echo ""
    echo "复制头文件..."
    echo "$HEADER_FILES" | while read -r file; do
        sudo cp "$file" "$CUDA_PATH/include/"
        echo "  ✓ 已复制: $(basename "$file")"
    done
    sudo chmod a+r "$CUDA_PATH/include/cudnn"*.h
    echo "✓ 头文件复制完成"
    echo ""
fi

# 查找并复制库文件
echo "查找库文件..."
LIB_FILES=$(find "$CUDNN_WIN" -name "libcudnn.so*" 2>/dev/null)

# 如果没找到 .so 文件，检查是否有 .dll 文件（Windows 版本）
if [ -z "$LIB_FILES" ]; then
    DLL_FILES=$(find "$CUDNN_WIN" -name "cudnn*.dll" 2>/dev/null | head -1)
    if [ -n "$DLL_FILES" ]; then
        echo "⚠️  警告: 找到的是 Windows 版本的 cuDNN（.dll 文件）"
        echo "WSL2 需要 Linux 版本的 cuDNN（.so 文件）"
        echo ""
        echo "解决方案："
        echo "1. 从 NVIDIA 官网下载 Linux 版本的 cuDNN 9.1.1 for CUDA 12.4"
        echo "   https://developer.nvidia.com/cudnn"
        echo "2. 下载文件格式: cudnn-linux-x86_64-9.1.1.*_cuda12.4-archive.tar.xz"
        echo "3. 解压后使用 install_cudnn9.sh 脚本安装"
        echo ""
        echo "或者，如果您有 Linux 版本的 cuDNN 压缩包，可以："
        echo "  cd /mnt/d/Programs/github/lingua/scripts/wsl2_piper"
        echo "  bash install_cudnn9.sh"
    else
        echo "⚠️  警告: 未找到 libcudnn.so* 库文件"
        echo "请检查 cuDNN 安装是否完整"
    fi
else
    echo "找到以下库文件："
    echo "$LIB_FILES" | while read -r file; do
        echo "  - $file"
    done
    echo ""
    echo "复制库文件..."
    echo "$LIB_FILES" | while read -r file; do
        sudo cp "$file" "$CUDA_PATH/lib64/"
        echo "  ✓ 已复制: $(basename "$file")"
    done
    sudo chmod a+r "$CUDA_PATH/lib64/libcudnn"*
    echo "✓ 库文件复制完成"
    echo ""
fi

# 检查是否需要创建符号链接
echo "检查符号链接..."
if [ -f "$CUDA_PATH/lib64/libcudnn.so.9.1.1" ] && [ ! -f "$CUDA_PATH/lib64/libcudnn.so.9" ]; then
    echo "创建符号链接: libcudnn.so.9 -> libcudnn.so.9.1.1"
    cd "$CUDA_PATH/lib64"
    sudo ln -s libcudnn.so.9.1.1 libcudnn.so.9
    cd - > /dev/null
    echo "✓ 符号链接创建完成"
    echo ""
fi

# 更新动态链接器缓存
echo "更新动态链接器缓存..."
sudo ldconfig
echo "✓ 缓存更新完成"
echo ""

# 验证安装
echo "验证安装..."
echo ""
echo "头文件："
ls -la "$CUDA_PATH/include/cudnn"*.h 2>/dev/null || echo "  未找到头文件"
echo ""
echo "库文件："
ls -la "$CUDA_PATH/lib64/libcudnn"*.so* 2>/dev/null || echo "  未找到库文件"
echo ""
echo "系统库缓存："
ldconfig -p | grep cudnn || echo "  未在系统缓存中找到 cuDNN"
echo ""

echo "=== 复制完成 ==="
echo ""
echo "下一步："
echo "1. 设置库路径: export LD_LIBRARY_PATH=$CUDA_PATH/targets/x86_64-linux/lib:$CUDA_PATH/lib64:\$LD_LIBRARY_PATH"
echo "2. 测试 ONNX Runtime: python -c \"import onnxruntime as ort; print(ort.get_available_providers())\""
echo "3. 运行测试脚本: python /mnt/d/Programs/github/lingua/scripts/wsl2_piper/test_piper_gpu.py"

