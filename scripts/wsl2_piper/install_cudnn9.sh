#!/bin/bash
# 安装 cuDNN 9 脚本
# 注意：需要先从 NVIDIA 官网下载 cuDNN 9.x for CUDA 12.x

set -e

echo "=== cuDNN 9 安装脚本 ==="
echo ""
echo "前提条件："
echo "1. 已安装 CUDA 12.4"
echo "2. 已从 NVIDIA 官网下载 cuDNN 9.x for CUDA 12.4"
echo "   下载地址: https://developer.nvidia.com/cudnn"
echo "   需要注册 NVIDIA 开发者账号"
echo ""
echo "⚠️  重要：版本匹配要求"
echo "   - CUDA 12.4 应使用 cuDNN 9.1.1 for CUDA 12.4"
echo "   - 不要使用 cuDNN 9.12 for CUDA 12.9（版本不匹配可能导致兼容性问题）"
echo ""
echo "📌 Ubuntu 版本说明："
echo "   - 虽然 NVIDIA 官网可能只列出 Ubuntu 20.04/22.04 支持"
echo "   - 但使用 tar.xz 压缩包手动安装不依赖特定的 Ubuntu 版本"
echo "   - 只要 CUDA 版本匹配，可以在 Ubuntu 24.04 等版本上正常工作"
echo ""
echo "请将下载的 cuDNN 压缩包放在当前目录，文件名格式："
echo "  cudnn-linux-x86_64-9.1.1.*_cuda12.4-archive.tar.xz"
echo ""

# 检查是否有 cuDNN 压缩包
CUDNN_ARCHIVE=$(ls cudnn-linux-x86_64-9.*_cuda12.*-archive.tar.xz 2>/dev/null | head -1)

if [ -z "$CUDNN_ARCHIVE" ]; then
    echo "❌ 错误: 未找到 cuDNN 压缩包"
    echo ""
    echo "请执行以下步骤："
    echo "1. 访问 https://developer.nvidia.com/cudnn"
    echo "2. 注册/登录 NVIDIA 开发者账号"
    echo "3. 下载 cuDNN 9.1.1 for CUDA 12.4 (Linux x86_64)"
    echo "   注意：必须选择 CUDA 12.4 版本，不要选择 12.9 版本"
    echo "4. 将下载的文件放到当前目录"
    echo "5. 重新运行此脚本"
    exit 1
fi

echo "找到 cuDNN 压缩包: $CUDNN_ARCHIVE"
echo ""

# 检查版本匹配
if echo "$CUDNN_ARCHIVE" | grep -q "cuda12.9"; then
    echo "⚠️  警告: 检测到 cuDNN for CUDA 12.9，但您的 CUDA 版本是 12.4"
    echo "   这可能导致兼容性问题。"
    echo ""
    echo "   建议："
    echo "   1. 下载 cuDNN 9.1.1 for CUDA 12.4（推荐）"
    echo "   2. 或者继续安装并测试兼容性（不推荐）"
    echo ""
    read -p "是否继续安装？(y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "安装已取消。请下载匹配 CUDA 12.4 的 cuDNN 版本。"
        exit 1
    fi
    echo ""
fi

# 检查 CUDA 安装
CUDA_PATH="/usr/local/cuda-12.4"
if [ ! -d "$CUDA_PATH" ]; then
    echo "❌ 错误: 未找到 CUDA 12.4 安装目录: $CUDA_PATH"
    echo "请先安装 CUDA 12.4"
    exit 1
fi

echo "CUDA 路径: $CUDA_PATH"
echo ""

# 解压
echo "解压 cuDNN..."
TEMP_DIR=$(mktemp -d)
tar -xf "$CUDNN_ARCHIVE" -C "$TEMP_DIR"
CUDNN_DIR=$(find "$TEMP_DIR" -maxdepth 1 -type d -name "cudnn-*" | head -1)

if [ -z "$CUDNN_DIR" ]; then
    echo "❌ 错误: 无法找到解压后的 cuDNN 目录"
    rm -rf "$TEMP_DIR"
    exit 1
fi

echo "解压目录: $CUDNN_DIR"
echo ""

# 复制文件
echo "安装 cuDNN 库文件..."
sudo cp -P "$CUDNN_DIR"/include/cudnn*.h "$CUDA_PATH"/include 2>/dev/null || true
sudo cp -P "$CUDNN_DIR"/lib/libcudnn* "$CUDA_PATH"/lib64 2>/dev/null || true

# 设置权限
sudo chmod a+r "$CUDA_PATH"/include/cudnn*.h 2>/dev/null || true
sudo chmod a+r "$CUDA_PATH"/lib64/libcudnn* 2>/dev/null || true

# 更新动态链接器缓存
echo "更新动态链接器缓存..."
sudo ldconfig

# 清理临时文件
rm -rf "$TEMP_DIR"

echo ""
echo "✓ cuDNN 安装完成！"
echo ""
echo "验证安装："
echo "  ldconfig -p | grep cudnn"
echo ""
echo "测试 ONNX Runtime："
echo "  python -c \"import onnxruntime as ort; print(ort.get_available_providers())\""
echo ""
echo "如果看到 'CUDAExecutionProvider'，说明安装成功！"

