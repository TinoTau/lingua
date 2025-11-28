# 从 Windows 路径安装 cuDNN 到 WSL2

## 前提条件
- cuDNN 已安装在 Windows: `C:\Program Files\NVIDIA\CUDNN\v9.1`
- WSL2 中已安装 CUDA 12.4: `/usr/local/cuda-12.4`

## 操作步骤

### 1. 在 WSL2 中打开终端

```bash
wsl
```

### 2. 检查 Windows 路径下的 cuDNN 文件

```bash
# 查看 Windows 路径下的文件结构
ls -la "/mnt/c/Program Files/NVIDIA/CUDNN/v9.1"
```

通常 cuDNN 的目录结构是：
```
v9.1/
├── bin/
├── include/
│   └── cudnn*.h
└── lib/
    └── libcudnn*.so*
```

或者可能是：
```
v9.1/
├── include/
│   └── cudnn*.h
└── lib/
    └── x64/
        └── libcudnn*.so*
```

### 3. 找到库文件位置

```bash
# 查找 libcudnn.so.9 文件
find "/mnt/c/Program Files/NVIDIA/CUDNN/v9.1" -name "libcudnn.so*" 2>/dev/null

# 查找头文件
find "/mnt/c/Program Files/NVIDIA/CUDNN/v9.1" -name "cudnn*.h" 2>/dev/null
```

### 4. 复制文件到 WSL2 的 CUDA 目录

**重要：由于路径中有空格，需要使用不同的方法**

**方法 1：使用 find 命令（推荐）**

```bash
# 设置变量（避免重复输入长路径）
CUDNN_WIN="/mnt/c/Program Files/NVIDIA/CUDNN/v9.1"
CUDA_PATH="/usr/local/cuda-12.4"

# 复制头文件（使用 find 命令）
sudo find "$CUDNN_WIN" -name "cudnn*.h" -exec cp {} "$CUDA_PATH/include/" \;

# 复制库文件（使用 find 命令）
sudo find "$CUDNN_WIN" -name "libcudnn.so*" -exec cp {} "$CUDA_PATH/lib64/" \;

# 设置权限
sudo chmod a+r "$CUDA_PATH/include/cudnn"*.h
sudo chmod a+r "$CUDA_PATH/lib64/libcudnn"*
```

**方法 2：先进入目录再复制**

```bash
# 设置变量
CUDNN_WIN="/mnt/c/Program Files/NVIDIA/CUDNN/v9.1"
CUDA_PATH="/usr/local/cuda-12.4"

# 复制头文件
cd "$CUDNN_WIN/include" 2>/dev/null && sudo cp cudnn*.h "$CUDA_PATH/include/" && cd - || echo "include 目录不存在，尝试其他路径"

# 复制库文件（根据实际路径调整）
cd "$CUDNN_WIN/lib" 2>/dev/null && sudo cp libcudnn.so* "$CUDA_PATH/lib64/" && cd - || \
cd "$CUDNN_WIN/lib/x64" 2>/dev/null && sudo cp libcudnn.so* "$CUDA_PATH/lib64/" && cd - || \
echo "请检查库文件的实际位置"

# 设置权限
sudo chmod a+r "$CUDA_PATH/include/cudnn"*.h
sudo chmod a+r "$CUDA_PATH/lib64/libcudnn"*
```

**方法 3：手动列出文件后复制**

```bash
# 先查看文件
ls -la "/mnt/c/Program Files/NVIDIA/CUDNN/v9.1/include/"
ls -la "/mnt/c/Program Files/NVIDIA/CUDNN/v9.1/lib/"

# 然后根据实际文件名逐个复制
# 例如：
# sudo cp "/mnt/c/Program Files/NVIDIA/CUDNN/v9.1/include/cudnn.h" /usr/local/cuda-12.4/include/
# sudo cp "/mnt/c/Program Files/NVIDIA/CUDNN/v9.1/include/cudnn_version.h" /usr/local/cuda-12.4/include/
# sudo cp "/mnt/c/Program Files/NVIDIA/CUDNN/v9.1/lib/libcudnn.so.9.1.1" /usr/local/cuda-12.4/lib64/
```

**情况 B：如果文件在其他位置**

根据 `find` 命令的结果，调整路径后执行复制命令。

### 5. 更新动态链接器缓存

```bash
sudo ldconfig
```

### 6. 验证安装

```bash
# 检查库文件是否存在
ldconfig -p | grep cudnn

# 检查文件是否在正确位置
ls -la /usr/local/cuda-12.4/lib64/libcudnn.so*

# 检查头文件
ls -la /usr/local/cuda-12.4/include/cudnn*.h
```

### 7. 测试 ONNX Runtime

```bash
cd ~/piper_env
source .venv/bin/activate

# 设置库路径
export LD_LIBRARY_PATH=/usr/local/cuda-12.4/targets/x86_64-linux/lib:/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH

# 测试 ONNX Runtime
python -c "import onnxruntime as ort; providers = ort.get_available_providers(); print('可用执行提供程序:', providers); print('CUDA 可用:', 'CUDAExecutionProvider' in providers)"
```

如果看到 `CUDA 可用: True`，说明安装成功！

### 8. 运行完整测试

```bash
python /mnt/d/Programs/github/lingua/scripts/wsl2_piper/test_piper_gpu.py
```

应该看到：
```
实际使用的执行提供程序: ['CUDAExecutionProvider', 'CPUExecutionProvider']
✓ 确认使用 GPU 加速
```

## 常见问题

### 问题 1：找不到 libcudnn.so.9

如果 `ldconfig -p | grep cudnn` 没有输出，可能是：
1. 文件没有正确复制
2. 需要创建符号链接

```bash
# 检查是否有 libcudnn.so.9.x.x 文件
ls -la /usr/local/cuda-12.4/lib64/libcudnn.so*

# 如果有 libcudnn.so.9.1.1 但没有 libcudnn.so.9，创建符号链接
cd /usr/local/cuda-12.4/lib64
sudo ln -s libcudnn.so.9.1.1 libcudnn.so.9
sudo ldconfig
```

### 问题 2：权限错误

如果复制时出现权限错误，确保使用 `sudo`：

```bash
sudo cp ...
sudo chmod ...
```

### 问题 3：路径中有空格

Windows 路径中的空格需要用引号括起来：

```bash
"/mnt/c/Program Files/NVIDIA/CUDNN/v9.1"
```

## 永久设置库路径

安装成功后，将库路径添加到 `~/.bashrc`：

```bash
echo 'export LD_LIBRARY_PATH=/usr/local/cuda-12.4/targets/x86_64-linux/lib:/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc
```

