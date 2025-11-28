# cuDNN 9.1.1 快速安装指南

## 前提条件
- 已安装 CUDA 12.4
- 已下载 cuDNN 9.1.1 的 .deb 包

## 安装步骤

### 方法 1：使用安装脚本（推荐）

如果 .deb 文件在 Windows 路径 `D:\installer\cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb`：

```bash
cd /mnt/d/Programs/github/lingua/scripts/wsl2_piper
bash install_cudnn9_deb.sh /mnt/d/installer/cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb
```

或者将文件复制到脚本目录：

```bash
# 复制文件到脚本目录
cp /mnt/d/installer/cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb /mnt/d/Programs/github/lingua/scripts/wsl2_piper/

# 运行安装脚本
cd /mnt/d/Programs/github/lingua/scripts/wsl2_piper
bash install_cudnn9_deb.sh
```

### 方法 2：手动安装

```bash
# 1. 安装 .deb 包（设置本地仓库）
sudo dpkg -i /mnt/d/installer/cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb

# 2. 如果出现依赖错误，修复依赖
sudo apt-get install -f -y

# 3. 更新 apt 仓库
sudo apt-get update

# 4. 安装 cuDNN 库（尝试多个可能的包名）
sudo apt-get install -y libcudnn9 || sudo apt-get install -y libcudnn9-cuda-12

# 5. 安装开发文件（可选，但推荐）
sudo apt-get install -y libcudnn9-dev-cuda-12

# 6. 更新库缓存
sudo ldconfig
```

## 验证安装

```bash
# 检查库文件
ldconfig -p | grep cudnn

# 测试 ONNX Runtime
cd ~/piper_env
source .venv/bin/activate
export LD_LIBRARY_PATH=/usr/local/cuda-12.4/targets/x86_64-linux/lib:/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH
python -c "import onnxruntime as ort; providers = ort.get_available_providers(); print('可用执行提供程序:', providers); print('CUDA 可用:', 'CUDAExecutionProvider' in providers)"
```

如果看到 `CUDA 可用: True`，说明安装成功！

## 运行完整测试

```bash
python /mnt/d/Programs/github/lingua/scripts/wsl2_piper/test_piper_gpu.py
```

应该看到：
```
实际使用的执行提供程序: ['CUDAExecutionProvider', 'CPUExecutionProvider']
✓ 确认使用 GPU 加速
```

