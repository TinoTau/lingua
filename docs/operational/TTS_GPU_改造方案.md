# TTS GPU 改造方案

**最后更新**: 2025-01-XX

本文档说明如何将 TTS 服务改造为使用 GPU 加速。

---

## 📊 当前状态

### 当前实现
- **TTS 引擎**: Piper TTS
- **模型格式**: ONNX
- **运行环境**: WSL2 (Linux)
- **执行方式**: 通过命令行工具 `piper` 调用 ONNX Runtime
- **GPU 支持**: ❌ 未启用（使用 CPU）

### 性能现状
- **CPU 模式**: 约 200-500ms（取决于文本长度）
- **预期 GPU 模式**: 约 50-150ms
- **预期提升**: 约 3-4 倍

---

## 🎯 改造方案

### 方案 1：使用 ONNX Runtime CUDA 执行提供程序（推荐）⭐

**优势**：
- ✅ 无需修改模型（继续使用现有 ONNX 模型）
- ✅ 改动最小（只需安装 ONNX Runtime GPU 版本）
- ✅ 兼容性好（Piper 原生支持）
- ✅ 性能提升明显

**步骤**：

#### 1. 在 WSL2 中安装 CUDA Toolkit

```bash
# 在 WSL2 中安装 CUDA Toolkit 12.4
wget https://developer.download.nvidia.com/compute/cuda/repos/wsl-ubuntu/x86_64/cuda-keyring_1.1-1_all.deb
sudo dpkg -i cuda-keyring_1.1-1_all.deb
sudo apt-get update
sudo apt-get -y install cuda-toolkit-12-4

# 验证安装
nvcc --version
```

#### 2. 安装 ONNX Runtime GPU 版本

```bash
# 进入 Piper 虚拟环境
cd ~/piper_env
source .venv/bin/activate

# 卸载 CPU 版本的 onnxruntime（如果已安装）
pip uninstall onnxruntime -y

# 安装 GPU 版本的 onnxruntime
pip install onnxruntime-gpu

# 验证安装
python -c "import onnxruntime as ort; print('Available providers:', ort.get_available_providers())"
```

**预期输出**：
```
Available providers: ['CUDAExecutionProvider', 'CPUExecutionProvider']
```

#### 3. 配置 Piper 使用 GPU

Piper 默认会自动检测并使用可用的执行提供程序。如果 ONNX Runtime GPU 版本已安装，Piper 会自动使用 CUDA。

**验证方法**：

```bash
# 测试 Piper 是否使用 GPU
piper --model ~/piper_models/zh/zh_CN-huayan-medium/zh_CN-huayan-medium.onnx \
      --input_file test.txt \
      --output_file test.wav \
      --verbose
```

查看输出中是否有 CUDA 相关信息。

#### 4. 修改 HTTP 服务脚本（可选：添加 GPU 检测）

修改 `scripts/wsl2_piper/piper_http_server.py`，添加 GPU 检测和日志：

```python
import onnxruntime as ort

# 在启动时检查可用的执行提供程序
available_providers = ort.get_available_providers()
if 'CUDAExecutionProvider' in available_providers:
    print("[INFO] ✓ ONNX Runtime GPU support available (CUDA)")
    print(f"[INFO] Available providers: {available_providers}")
else:
    print("[WARN] ⚠ ONNX Runtime GPU support not available, using CPU")
    print(f"[INFO] Available providers: {available_providers}")
```

#### 5. 验证 GPU 使用

```bash
# 在 WSL2 中监控 GPU 使用
watch -n 1 nvidia-smi

# 在另一个终端发送 TTS 请求
curl -X POST http://127.0.0.1:5005/tts \
  -H "Content-Type: application/json" \
  -d '{"text": "测试GPU加速", "voice": "zh_CN-huayan-medium"}'
```

如果看到 GPU 使用率上升，说明 GPU 加速已启用。

---

### 方案 2：使用 PyTorch 版本的 TTS 模型

**优势**：
- ✅ 更好的 GPU 支持
- ✅ 可以使用更先进的模型（VITS、FastSpeech2 等）
- ✅ 更灵活的模型定制

**劣势**：
- ❌ 需要重新训练或转换模型
- ❌ 改动较大
- ❌ 需要更多开发工作

**如果选择此方案**，可以考虑：

1. **使用 Coqui TTS**（支持 GPU）：
   ```bash
   pip install TTS
   # 使用 GPU 版本的 PyTorch
   ```

2. **使用 ESPnet TTS**（支持 GPU）：
   ```bash
   pip install espnet
   ```

3. **使用 VITS 模型**（已在代码中，但未启用）：
   - 代码中已有 `VitsTtsEngine` 实现
   - 需要配置模型路径和 GPU 支持

---

## 🔧 推荐实施方案

### 阶段 1：快速启用 GPU（方案 1）

**目标**：最小改动，快速启用 GPU 加速

**步骤**：
1. ✅ 在 WSL2 中安装 CUDA Toolkit
2. ✅ 安装 `onnxruntime-gpu`
3. ✅ 验证 Piper 自动使用 GPU
4. ✅ 测试性能提升

**预计时间**：1-2 小时

### 阶段 2：优化和监控（可选）

**目标**：添加 GPU 监控和性能优化

**步骤**：
1. 添加 GPU 使用率监控
2. 添加性能日志
3. 优化批处理（如果需要）

---

## 📝 详细实施步骤

### 步骤 1：检查 WSL2 CUDA 支持

```bash
# 在 WSL2 中检查 NVIDIA 驱动
nvidia-smi

# 如果命令不存在，需要安装 NVIDIA 驱动（在 Windows 主机上）
# 确保 Windows 上已安装 NVIDIA 驱动（版本 >= 470.76）
```

### 步骤 2：安装 CUDA Toolkit（WSL2）

```bash
# 添加 NVIDIA CUDA 仓库
wget https://developer.download.nvidia.com/compute/cuda/repos/wsl-ubuntu/x86_64/cuda-keyring_1.1-1_all.deb
sudo dpkg -i cuda-keyring_1.1-1_all.deb
sudo apt-get update

# 安装 CUDA Toolkit 12.4
sudo apt-get -y install cuda-toolkit-12-4

# 设置环境变量（添加到 ~/.bashrc）
echo 'export PATH=/usr/local/cuda-12.4/bin:$PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc

# 验证安装
nvcc --version
```

### 步骤 3：安装 ONNX Runtime GPU

```bash
# 进入 Piper 虚拟环境
cd ~/piper_env
source .venv/bin/activate

# 检查当前 onnxruntime 版本
pip show onnxruntime

# 卸载 CPU 版本
pip uninstall onnxruntime -y

# 安装 GPU 版本（确保 CUDA 版本匹配）
# CUDA 12.4 可以使用 onnxruntime-gpu（通常支持 CUDA 11.x 和 12.x）
pip install onnxruntime-gpu

# 验证安装
python -c "import onnxruntime as ort; print('Providers:', ort.get_available_providers())"
```

**注意**：如果遇到版本兼容性问题，可以尝试：

```bash
# 安装特定版本的 onnxruntime-gpu
pip install onnxruntime-gpu==1.16.0
```

### 步骤 4：验证 GPU 使用

```bash
# 方法 1：使用 Python 脚本测试
python << EOF
import onnxruntime as ort
import numpy as np

# 检查可用提供程序
providers = ort.get_available_providers()
print("Available providers:", providers)

if 'CUDAExecutionProvider' in providers:
    print("✓ GPU support is available!")
    
    # 创建简单的测试会话
    # 注意：这需要实际的 ONNX 模型文件
    # 这里只是检查提供程序是否可用
else:
    print("✗ GPU support is not available")
EOF

# 方法 2：使用 Piper 命令行测试
echo "测试文本" > test.txt
piper --model ~/piper_models/zh/zh_CN-huayan-medium/zh_CN-huayan-medium.onnx \
      --input_file test.txt \
      --output_file test.wav \
      --verbose 2>&1 | grep -i cuda
```

### 步骤 5：修改 HTTP 服务添加 GPU 检测

修改 `scripts/wsl2_piper/piper_http_server.py`：

```python
# 在文件开头添加
try:
    import onnxruntime as ort
    ORT_AVAILABLE = True
except ImportError:
    ORT_AVAILABLE = False

# 在 main() 函数中添加 GPU 检测
def main():
    # ... 现有代码 ...
    
    # 检查 GPU 支持
    if ORT_AVAILABLE:
        providers = ort.get_available_providers()
        if 'CUDAExecutionProvider' in providers:
            print(f"[INFO] ✓ GPU support enabled (CUDA)")
            print(f"[INFO] Available providers: {providers}")
        else:
            print(f"[WARN] ⚠ GPU support not available, using CPU")
            print(f"[INFO] Available providers: {providers}")
    else:
        print("[WARN] ⚠ onnxruntime not available, cannot check GPU support")
    
    # ... 继续现有代码 ...
```

### 步骤 6：重启服务并测试

```bash
# 停止现有服务
pkill -f piper_http_server

# 重新启动服务
cd ~/piper_env
source .venv/bin/activate
python /path/to/piper_http_server.py --host 0.0.0.0 --port 5005

# 查看启动日志，应该看到 GPU 支持信息
```

### 步骤 7：性能测试

```bash
# 测试 TTS 请求
time curl -X POST http://127.0.0.1:5005/tts \
  -H "Content-Type: application/json" \
  -d '{"text": "这是一个测试文本，用于验证GPU加速效果。", "voice": "zh_CN-huayan-medium"}' \
  -o test_output.wav

# 对比 CPU 和 GPU 模式的性能
```

---

## 🐛 故障排查

### 问题 1：`CUDAExecutionProvider` 不可用或加载失败

**错误信息 1**：
```
Failed to load library libonnxruntime_providers_cuda.so with error: 
libcublasLt.so.12: cannot open shared object file: No such file or directory
```

**错误信息 2**：
```
Failed to load library libonnxruntime_providers_cuda.so with error: 
libcudnn.so.9: cannot open shared object file: No such file or directory
```

**可能原因**：
1. CUDA 运行时库未安装（WSL2 中需要单独安装）
2. cuDNN 9.* 未安装（ONNX Runtime 要求 cuDNN 9.* 和 CUDA 12.*）
3. CUDA 库路径未正确设置
4. CUDA 版本不匹配

**解决方法**：

**方法 1：安装 CUDA 运行时库（推荐）**

```bash
# 在 WSL2 中安装 CUDA 运行时库
sudo apt-get update
sudo apt-get install -y cuda-toolkit-12-4

# 设置库路径
export LD_LIBRARY_PATH=/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH

# 永久设置（添加到 ~/.bashrc）
echo 'export LD_LIBRARY_PATH=/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc
```

**方法 2：使用 Windows 主机的 CUDA 库（如果已安装）**

```bash
# 查找 Windows 中的 CUDA 库
# CUDA 通常安装在 C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.4\bin

# 在 WSL2 中创建符号链接或设置路径
# 注意：WSL2 可以直接访问 Windows 文件系统，但库文件可能需要复制到 WSL2
```

**方法 3：安装 CUDA 运行时库和 cuDNN 9（推荐）**

**步骤 1：安装 CUDA 运行时库**

```bash
sudo apt-get update
sudo apt-get install -y cuda-runtime-12-4 cuda-libraries-12-4
```

**步骤 2：下载 cuDNN 9**

ONNX Runtime 要求 cuDNN 9.*，但 Ubuntu 仓库通常只有 8.x，需要从 NVIDIA 官网手动下载：

⚠️ **重要：版本匹配要求**
- **CUDA 12.4** 应使用 **cuDNN 9.1.1 for CUDA 12.4**
- 不要使用 cuDNN 9.12 for CUDA 12.9（版本不匹配可能导致兼容性问题、性能下降或运行时错误）

**下载方式 A：下载 Linux 版本**

下载步骤：
1. 访问 https://developer.nvidia.com/cudnn
2. 注册/登录 NVIDIA 开发者账号（免费）
3. 下载 **cuDNN 9.1.1 for CUDA 12.4** (Linux x86_64)
   - 通常提供 `.deb` 格式（Ubuntu/Debian）或 `.rpm` 格式（RedHat/CentOS）
   - 文件名格式：`cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb`（Ubuntu 22.04）
   - ⚠️ 注意：必须选择 **CUDA 12.4** 版本，不要选择 12.9 或其他版本
   - 📌 **Ubuntu 版本说明**：虽然包是为 Ubuntu 22.04 设计的，但可以在 Ubuntu 24.04 上安装。如果遇到依赖问题，安装脚本会自动尝试从 `.deb` 包中提取文件。

**下载方式 B：使用 Windows 安装的 cuDNN（不推荐）**

⚠️ **注意**：Windows 版本的 cuDNN 包含的是 `.dll` 文件，无法在 WSL2（Linux）中使用。WSL2 必须使用 Linux 版本的 cuDNN（`.so` 文件）。

如果您在 Windows 上安装了 cuDNN，只能复制头文件，但库文件无法使用。**强烈建议下载 Linux 版本的 cuDNN**。

如果确实需要从 Windows 路径复制（仅头文件），详细步骤请参考：`scripts/wsl2_piper/安装cuDNN_从Windows路径.md`

**步骤 3：安装 cuDNN 9**

**方法 A：使用 .deb 包安装（推荐）**

如果您下载的是 `.deb` 格式的本地仓库包（例如：`cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb`）：

⚠️ **注意**：
- `.deb` 包不能在 Windows 中安装，必须在 WSL2 的 Ubuntu 环境中安装
- 此包是为 Ubuntu 22.04 设计的，在 Ubuntu 24.04 上可能遇到依赖问题
- 如果标准安装失败，安装脚本会自动尝试从 `.deb` 包中提取文件

```bash
# 将下载的 .deb 文件放到脚本目录
cd /mnt/d/Programs/github/lingua/scripts/wsl2_piper
# 将下载的 cudnn-local-repo-ubuntu*.deb 文件复制到这里

# 运行 .deb 安装脚本
bash install_cudnn9_deb.sh
```

脚本会自动：
1. 尝试通过 apt 安装（如果兼容）
2. 如果失败，自动从 .deb 包中提取文件并安装

或者手动安装：

```bash
# 安装 .deb 包（设置本地仓库）
sudo dpkg -i cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb

# 如果出现依赖错误，修复依赖
sudo apt-get install -f -y

# 更新 apt 仓库
sudo apt-get update

# 安装 cuDNN 库（尝试多个可能的包名）
sudo apt-get install -y libcudnn9 || sudo apt-get install -y libcudnn9-cuda-12

# 安装开发文件（可选）
sudo apt-get install -y libcudnn9-dev || sudo apt-get install -y libcudnn9-dev-cuda-12

# 更新库缓存
sudo ldconfig
```

**如果标准安装失败，可以从 .deb 包中提取文件**：

```bash
# 安装必要的工具
sudo apt-get install -y binutils

# 创建临时目录
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# 提取 .deb 包
ar x /path/to/cudnn-local-repo-ubuntu2204-9.1.1_1.0-1_amd64.deb

# 提取数据文件
tar -xf data.tar.xz  # 或 tar -xzf data.tar.gz

# 查找并复制文件到 CUDA 目录
find . -name "cudnn*.h" -exec sudo cp {} /usr/local/cuda-12.4/include/ \;
find . -name "libcudnn.so*" -exec sudo cp {} /usr/local/cuda-12.4/lib64/ \;

# 设置权限
sudo chmod a+r /usr/local/cuda-12.4/include/cudnn*.h
sudo chmod a+r /usr/local/cuda-12.4/lib64/libcudnn*

# 创建符号链接（如果需要）
cd /usr/local/cuda-12.4/lib64
sudo ln -s libcudnn.so.9.1.1 libcudnn.so.9 2>/dev/null || true

# 更新库缓存
sudo ldconfig

# 清理
cd -
rm -rf "$TEMP_DIR"
```

**方法 B：手动安装**

```bash
# 解压并安装
cd ~/Downloads  # 假设下载文件在这里
tar -xvf cudnn-linux-x86_64-9.1.1.*_cuda12.4-archive.tar.xz
cd cudnn-linux-x86_64-9.1.1.*_cuda12.4-archive

# 复制库文件到 CUDA 目录
sudo cp include/cudnn*.h /usr/local/cuda-12.4/include
sudo cp lib/libcudnn* /usr/local/cuda-12.4/lib64
sudo chmod a+r /usr/local/cuda-12.4/include/cudnn*.h
sudo chmod a+r /usr/local/cuda-12.4/lib64/libcudnn*

# 更新动态链接器缓存
sudo ldconfig
```

**步骤 4：设置库路径**

```bash
# 临时设置
export LD_LIBRARY_PATH=/usr/local/cuda-12.4/targets/x86_64-linux/lib:/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH

# 永久设置（添加到 ~/.bashrc）
echo 'export LD_LIBRARY_PATH=/usr/local/cuda-12.4/targets/x86_64-linux/lib:/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc
```

**方法 4：尝试从 Ubuntu 仓库安装（如果可用）**

```bash
# 检查是否有 cuDNN 9 包
apt-cache search cudnn9

# 如果有，尝试安装（但通常 Ubuntu 仓库只有 8.x）
# sudo apt-get install -y libcudnn9-cuda-12  # 如果存在
```

**验证修复**：

```bash
# 检查 CUDA 库文件是否存在
ldconfig -p | grep cublas
ldconfig -p | grep cudnn

# 检查 cuDNN 版本（如果已安装）
cat /usr/local/cuda-12.4/include/cudnn_version.h | grep CUDNN_MAJOR -A 2

# 测试 ONNX Runtime
python -c "import onnxruntime as ort; print(ort.get_available_providers())"
# 应该看到 'CUDAExecutionProvider' 在列表中

# 运行测试脚本验证 GPU 使用
cd ~/piper_env
source .venv/bin/activate
python /mnt/d/Programs/github/lingua/scripts/wsl2_piper/test_piper_gpu.py
# 应该看到 "实际使用的执行提供程序: ['CUDAExecutionProvider', 'CPUExecutionProvider']"
```

### 问题 2：cuDNN 版本不匹配

**错误信息**：
```
安装时提示 cuDNN 适配 CUDA 12.9，但系统 CUDA 版本是 12.4
```

**问题说明**：
- cuDNN 版本必须与 CUDA 版本匹配
- CUDA 12.4 应使用 cuDNN 9.1.1 for CUDA 12.4
- 使用不匹配的版本可能导致：
  - 运行时错误或崩溃
  - 性能下降
  - 功能异常

**解决方法**：

1. **推荐：下载匹配的版本**
   - 访问 https://developer.nvidia.com/cudnn
   - 下载 **cuDNN 9.1.1 for CUDA 12.4**（不是 12.9）
   - 重新安装

2. **如果已安装不匹配版本，可以尝试测试兼容性**：
   ```bash
   # 安装后测试
   python -c "import onnxruntime as ort; print(ort.get_available_providers())"
   
   # 运行测试脚本
   python /mnt/d/Programs/github/lingua/scripts/wsl2_piper/test_piper_gpu.py
   
   # 如果出现错误或性能异常，建议卸载并安装匹配版本
   ```

3. **卸载已安装的 cuDNN**（如果需要）：
   ```bash
   sudo rm -f /usr/local/cuda-12.4/include/cudnn*.h
   sudo rm -f /usr/local/cuda-12.4/lib64/libcudnn*
   sudo ldconfig
   ```

### 问题 3：Piper 仍然使用 CPU

**可能原因**：
1. ONNX Runtime 未检测到 GPU
2. cuDNN 版本不匹配导致 CUDA 提供程序加载失败
3. 模型文件路径问题

**解决方法**：
```bash
# 检查 ONNX Runtime 提供程序
python -c "import onnxruntime as ort; print(ort.get_available_providers())"

# 如果只有 CPU，检查 CUDA 库路径
ldconfig -p | grep cuda

# 设置 CUDA 库路径
export LD_LIBRARY_PATH=/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH
```

### 问题 4：性能提升不明显

**可能原因**：
1. 文本太短，GPU 优势不明显
2. 模型太小，CPU 已经足够快
3. 数据传输开销

**解决方法**：
- 测试更长的文本（> 100 字符）
- 使用批处理（如果支持）
- 检查 GPU 使用率（`nvidia-smi`）

---

## 📊 性能对比

### 预期性能提升

| 文本长度 | CPU 模式 | GPU 模式 | 提升 |
|---------|---------|---------|------|
| 短文本（< 50 字符） | 200-300ms | 50-100ms | 2-3x |
| 中等文本（50-200 字符） | 300-500ms | 100-150ms | 3-4x |
| 长文本（> 200 字符） | 500-800ms | 150-250ms | 3-4x |

### 验证方法

```bash
# 创建测试脚本
cat > test_tts_perf.sh << 'EOF'
#!/bin/bash
TEXT="这是一个性能测试文本，用于验证TTS服务的GPU加速效果。我们将测试不同长度的文本，以评估性能提升。"
for i in {1..10}; do
    time curl -s -X POST http://127.0.0.1:5005/tts \
      -H "Content-Type: application/json" \
      -d "{\"text\": \"$TEXT\", \"voice\": \"zh_CN-huayan-medium\"}" \
      -o /dev/null
done
EOF

chmod +x test_tts_perf.sh
./test_tts_perf.sh
```

---

## 📚 相关文档

- [ONNX Runtime GPU 安装指南](https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html)
- [Piper TTS 官方文档](https://github.com/rhasspy/piper)
- [CUDA Toolkit 安装指南](./CUDA_Toolkit_安装指南.md)
- [GPU 改造进度总结](./GPU改造进度总结.md)

---

## ✅ 检查清单

### 准备阶段
- [ ] WSL2 中已安装 NVIDIA 驱动
- [ ] WSL2 中已安装 CUDA Toolkit 12.4
- [ ] 验证 `nvidia-smi` 可用
- [ ] 验证 `nvcc --version` 可用

### 安装阶段
- [ ] 已卸载 CPU 版本的 `onnxruntime`
- [ ] 已安装 GPU 版本的 `onnxruntime-gpu`
- [ ] 验证 `CUDAExecutionProvider` 可用

### 配置阶段
- [ ] 已修改 HTTP 服务脚本添加 GPU 检测
- [ ] 已重启 TTS 服务
- [ ] 启动日志显示 GPU 支持已启用

### 验证阶段
- [ ] 发送 TTS 请求时 GPU 使用率上升
- [ ] 性能测试显示明显的性能提升
- [ ] 服务稳定运行无错误

---

**最后更新**: 2025-01-XX

