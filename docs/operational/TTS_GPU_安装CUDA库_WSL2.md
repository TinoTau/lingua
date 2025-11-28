# TTS GPU - WSL2 中安装 CUDA 库指南

**最后更新**: 2025-11-28

本文档说明如何在 WSL2 中安装 CUDA 运行时库，以支持 ONNX Runtime GPU 加速。

---

## 问题描述

错误信息：
```
Failed to load library libonnxruntime_providers_cuda.so with error: 
libcublasLt.so.12: cannot open shared object file: No such file or directory
```

这说明 ONNX Runtime 找到了 CUDA 提供程序，但缺少 CUDA 运行时库。

---

## 解决方案

### 方法 1：只安装 CUDA 运行时库（推荐，体积小）

```bash
# 1. 更新包列表
sudo apt-get update

# 2. 安装 CUDA 运行时库（不包含编译器）
sudo apt-get install -y \
    libcublas-12-4 \
    libcublas-dev-12-4 \
    libcurand-12-4 \
    libcurand-dev-12-4 \
    libcusolver-12-4 \
    libcusolver-dev-12-4 \
    libcusparse-12-4 \
    libcusparse-dev-12-4 \
    libcudnn9-cuda-12

# 3. 更新库缓存
sudo ldconfig

# 4. 设置库路径
export LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH

# 5. 永久设置（添加到 ~/.bashrc）
echo 'export LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH' >> ~/.bashrc
```

### 方法 2：安装 CUDA Toolkit（如果方法 1 不行）

如果遇到依赖问题，可以尝试跳过有问题的包：

```bash
# 1. 添加 CUDA 仓库（如果还没添加）
wget https://developer.download.nvidia.com/compute/cuda/repos/wsl-ubuntu/x86_64/cuda-keyring_1.1-1_all.deb
sudo dpkg -i cuda-keyring_1.1-1_all.deb
sudo apt-get update

# 2. 尝试安装 CUDA Toolkit，但跳过有问题的包
sudo apt-get install -y cuda-toolkit-12-4 --no-install-recommends

# 或者只安装核心包
sudo apt-get install -y \
    cuda-cudart-12-4 \
    cuda-cublas-12-4 \
    cuda-cublas-dev-12-4 \
    cuda-cudnn-12-4 \
    libcudnn9-cuda-12

# 3. 设置环境变量
export PATH=/usr/local/cuda-12.4/bin:$PATH
export LD_LIBRARY_PATH=/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH

# 4. 永久设置
echo 'export PATH=/usr/local/cuda-12.4/bin:$PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
```

### 方法 3：从 Windows 主机复制 CUDA 库（临时方案）

如果上述方法都不行，可以尝试从 Windows 主机复制 CUDA 库：

```bash
# Windows 中的 CUDA 库通常在：
# C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4\bin

# 在 WSL2 中，可以通过 /mnt/c 访问 Windows C 盘
# 但库文件可能需要复制到 WSL2 文件系统

# 创建目录
mkdir -p ~/cuda_libs

# 复制库文件（需要手动操作，或使用脚本）
# 注意：这种方法不推荐，因为可能存在兼容性问题
```

---

## 验证安装

### 1. 检查库文件是否存在

```bash
# 检查 cublas 库
ldconfig -p | grep cublas

# 应该看到类似输出：
# libcublas.so.12 (libc6,x86-64) => /usr/lib/x86_64-linux-gnu/libcublas.so.12
```

### 2. 检查库路径

```bash
# 检查 LD_LIBRARY_PATH
echo $LD_LIBRARY_PATH

# 检查库文件是否可访问
ldd /usr/lib/x86_64-linux-gnu/libcublas.so.12 2>&1 | head -5
```

### 3. 测试 ONNX Runtime

```bash
cd ~/piper_env
source .venv/bin/activate
python -c "import onnxruntime as ort; print('Providers:', ort.get_available_providers())"
```

### 4. 运行测试脚本

```bash
python /mnt/d/Programs/github/lingua/scripts/wsl2_piper/test_piper_gpu.py
```

---

## 故障排查

### 问题 1：依赖冲突

如果遇到 `libtinfo5` 等依赖问题：

```bash
# 尝试修复依赖
sudo apt-get --fix-broken install

# 或者安装缺失的依赖
sudo apt-get install -y libtinfo5

# 如果还是不行，尝试使用 Ubuntu 24.04 的包
sudo apt-get install -y libtinfo6
```

### 问题 2：库文件找不到

```bash
# 查找库文件位置
find /usr -name "libcublasLt.so.12" 2>/dev/null
find /usr/local -name "libcublasLt.so.12" 2>/dev/null

# 如果找到，添加到 LD_LIBRARY_PATH
export LD_LIBRARY_PATH=/找到的路径:$LD_LIBRARY_PATH
```

### 问题 3：版本不匹配

确保 CUDA 库版本与 ONNX Runtime 要求的版本匹配：
- ONNX Runtime 1.23.2 需要 CUDA 12.x
- 确保安装的是 CUDA 12.4 的库

---

## 快速检查清单

- [ ] 已添加 CUDA 仓库
- [ ] 已安装 CUDA 运行时库
- [ ] 已运行 `sudo ldconfig`
- [ ] 已设置 `LD_LIBRARY_PATH`
- [ ] 已添加到 `~/.bashrc`（永久设置）
- [ ] 已验证库文件存在（`ldconfig -p | grep cublas`）
- [ ] 已验证 ONNX Runtime 可以找到 CUDA 提供程序
- [ ] 已测试 Piper TTS GPU 模式

---

**最后更新**: 2025-11-28

