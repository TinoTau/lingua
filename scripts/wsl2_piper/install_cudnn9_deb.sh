#!/bin/bash
# 安装 cuDNN 9 脚本（使用 .deb 包）
# 注意：此脚本用于安装 cuDNN 的 .deb 本地仓库包

set -e

# 从 .deb 包中提取文件的函数
extract_from_deb() {
    local deb_file="$1"
    local CUDA_PATH="/usr/local/cuda-12.4"
    
    echo "从 .deb 包中提取文件..."
    
    # 创建临时目录
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # 提取 .deb 包
    ar x "$deb_file" 2>/dev/null || {
        echo "❌ 无法提取 .deb 包，可能需要安装 ar 工具"
        echo "   sudo apt-get install -y binutils"
        rm -rf "$TEMP_DIR"
        return 1
    }
    
    # 提取 data.tar.xz 或 data.tar.gz
    if [ -f data.tar.xz ]; then
        tar -xf data.tar.xz
    elif [ -f data.tar.gz ]; then
        tar -xzf data.tar.gz
    else
        echo "❌ 无法找到数据文件"
        rm -rf "$TEMP_DIR"
        return 1
    fi
    
    # 查找并复制文件
    echo "查找 cuDNN 文件..."
    
    # 查找头文件
    find . -name "cudnn*.h" -type f 2>/dev/null | while read -r file; do
        sudo cp "$file" "$CUDA_PATH/include/"
        echo "  ✓ 已复制头文件: $(basename "$file")"
    done
    
    # 查找库文件
    find . -name "libcudnn.so*" -type f 2>/dev/null | while read -r file; do
        sudo cp "$file" "$CUDA_PATH/lib64/"
        echo "  ✓ 已复制库文件: $(basename "$file")"
    done
    
    # 设置权限
    sudo chmod a+r "$CUDA_PATH/include/cudnn"*.h 2>/dev/null || true
    sudo chmod a+r "$CUDA_PATH/lib64/libcudnn"* 2>/dev/null || true
    
    # 创建符号链接（如果需要）
    if [ -f "$CUDA_PATH/lib64/libcudnn.so.9.1.1" ] && [ ! -f "$CUDA_PATH/lib64/libcudnn.so.9" ]; then
        cd "$CUDA_PATH/lib64"
        sudo ln -s libcudnn.so.9.1.1 libcudnn.so.9
        cd - > /dev/null
        echo "  ✓ 创建符号链接: libcudnn.so.9"
    fi
    
    # 清理
    cd - > /dev/null
    rm -rf "$TEMP_DIR"
    
    echo "✓ 文件提取完成"
}

echo "=== cuDNN 9 安装脚本（.deb 包方式）==="
echo ""
echo "前提条件："
echo "1. 已安装 CUDA 12.4"
echo "2. 已下载 cuDNN 9.1.1 的 .deb 本地仓库包"
echo "   文件名格式: cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb"
echo ""
echo "⚠️  注意："
echo "   - 此包是为 Ubuntu 22.04 设计的，在 Ubuntu 24.04 上可能不兼容"
echo "   - 如果标准安装失败，脚本会自动尝试从 .deb 包中提取文件"
echo ""

# 检查是否有 .deb 文件（当前目录或 Windows 路径）
DEB_FILE=$(ls cudnn-local-repo-ubuntu*.deb 2>/dev/null | head -1)

# 如果当前目录没有，尝试从 Windows 路径查找
if [ -z "$DEB_FILE" ]; then
    if [ -f "/mnt/d/installer/cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb" ]; then
        DEB_FILE="/mnt/d/installer/cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb"
        echo "✓ 从 Windows 路径找到 .deb 文件: $DEB_FILE"
    fi
fi

# 如果提供了命令行参数，使用参数作为文件路径
if [ -n "$1" ]; then
    if [ -f "$1" ]; then
        DEB_FILE="$1"
        echo "✓ 使用指定的文件路径: $DEB_FILE"
    else
        echo "❌ 错误: 指定的文件不存在: $1"
        exit 1
    fi
fi

if [ -z "$DEB_FILE" ] || [ ! -f "$DEB_FILE" ]; then
    echo "❌ 错误: 未找到 cuDNN .deb 文件"
    echo ""
    echo "请执行以下步骤之一："
    echo "1. 将下载的 .deb 文件放到当前目录"
    echo "2. 使用命令行参数指定文件路径："
    echo "   bash install_cudnn9_deb.sh /mnt/d/installer/cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb"
    echo "3. 或者将文件复制到当前目录"
    exit 1
fi

echo "找到 .deb 文件: $DEB_FILE"
echo ""

# 检查 Ubuntu 版本
UBUNTU_VERSION=$(lsb_release -rs 2>/dev/null || echo "unknown")
echo "当前 Ubuntu 版本: $UBUNTU_VERSION"
echo ""

if [[ "$UBUNTU_VERSION" == "24.04" ]]; then
    echo "⚠️  警告: 当前是 Ubuntu 24.04，但 .deb 包是为 Ubuntu 22.04 设计的"
    echo "   可能会遇到依赖问题。如果安装失败，请使用 tar.xz 格式的压缩包"
    echo ""
    read -p "是否继续安装？(y/N): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "安装已取消。"
        exit 1
    fi
    echo ""
fi

# 安装 .deb 包（这会设置本地 apt 仓库）
echo "安装 .deb 包（设置本地仓库）..."
sudo dpkg -i "$DEB_FILE" || {
    echo "⚠️  dpkg 安装可能失败，尝试修复依赖..."
    sudo apt-get update
    sudo apt-get install -f -y
}

echo ""

# 安装 GPG 密钥（如果需要）
echo "检查并安装 GPG 密钥..."
if [ -f "/var/cudnn-local-repo-ubuntu2204-9.1.1/cudnn-local-AD7F4AC5-keyring.gpg" ]; then
    echo "安装 GPG 密钥..."
    sudo cp /var/cudnn-local-repo-ubuntu2204-9.1.1/cudnn-local-AD7F4AC5-keyring.gpg /usr/share/keyrings/
    echo "✓ GPG 密钥已安装"
else
    echo "⚠️  警告: 未找到 GPG 密钥文件，但继续尝试..."
fi

echo ""

# 更新 apt 仓库
echo "更新 apt 仓库..."
sudo apt-get update

echo ""

# 安装 cuDNN
echo "安装 cuDNN 库..."
echo "尝试安装 libcudnn9 (cuDNN 9.x)..."

# 尝试多个可能的包名
if sudo apt-get install -y libcudnn9 2>/dev/null; then
    echo "✓ libcudnn9 安装成功"
elif sudo apt-get install -y libcudnn9-cuda-12 2>/dev/null; then
    echo "✓ libcudnn9-cuda-12 安装成功"
else
    echo "⚠️  标准包安装失败，尝试查找可用的 cuDNN 包..."
    echo ""
    echo "可用的 cuDNN 包："
    apt-cache search cudnn9 | head -10
    echo ""
    echo "请手动选择并安装合适的包，例如："
    echo "  sudo apt-get install -y libcudnn9-cuda-12"
    echo "  或"
    echo "  sudo apt-get install -y libcudnn9"
    echo ""
    read -p "是否尝试从 .deb 包中提取文件？(Y/n): " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        echo "从 .deb 包中提取文件..."
        extract_from_deb "$DEB_FILE"
    else
        exit 1
    fi
fi

# 安装开发文件（可选）
echo ""
read -p "是否安装 cuDNN 开发文件（头文件）？(Y/n): " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    echo "安装 cuDNN 开发文件..."
    if sudo apt-get install -y libcudnn9-dev-cuda-12 2>/dev/null; then
        echo "✓ libcudnn9-dev-cuda-12 安装成功"
    elif sudo apt-get install -y libcudnn9-dev 2>/dev/null; then
        echo "✓ libcudnn9-dev 安装成功"
    else
        echo "⚠️  开发文件安装失败，但运行时库已安装"
        echo "   可以手动尝试: sudo apt-get install -y libcudnn9-dev-cuda-12"
    fi
fi

# 更新动态链接器缓存
echo ""
echo "更新动态链接器缓存..."
sudo ldconfig

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

