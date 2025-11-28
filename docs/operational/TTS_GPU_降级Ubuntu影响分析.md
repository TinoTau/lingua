# TTS GPU - 降级 Ubuntu 影响分析

**最后更新**: 2025-11-28

---

## 降级 Ubuntu 的影响

### ✅ 不会受影响的部分

1. **模型文件**：`~/piper_models/` 中的 ONNX 模型文件不会受影响
2. **TTS 功能本身**：Piper TTS 的功能不会改变
3. **Windows 主机**：不会影响 Windows 系统

### ❌ 可能受影响的部分

1. **Python 环境**：可能需要重新安装 Python 和虚拟环境
2. **已安装的包**：需要重新安装所有 Python 依赖
3. **系统包**：可能需要重新安装系统级依赖
4. **配置**：可能需要重新配置环境变量和路径

### ⚠️ 需要重新做的操作

1. 重新创建虚拟环境：`python -m venv ~/piper_env/.venv`
2. 重新安装 Python 包：`pip install piper-tts onnxruntime-gpu fastapi uvicorn`
3. 重新配置环境变量
4. 重新测试服务

---

## 更简单的替代方案

### 方案 1：只安装 CUDA 运行时（推荐）⭐

不降级 Ubuntu，只安装必要的 CUDA 运行时库：

```bash
# 安装 CUDA 运行时（不包含编译器）
sudo apt-get install -y cuda-runtime-12-4

# 这会安装所有必要的 CUDA 运行时库，包括：
# - libcudart
# - libcublas
# - libcurand
# - libcusolver
# - libcusparse
# - 等等

# 更新库缓存
sudo ldconfig

# 设置库路径
export LD_LIBRARY_PATH=/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH
```

### 方案 2：使用 CPU 模式（最简单）⭐

**TTS 的 CPU 性能通常已经足够好**：
- CPU 模式：200-500ms（对于大多数应用已经很快）
- GPU 模式：50-150ms（提升明显，但不是必需的）

**如果当前 CPU 性能已经满足需求，可以暂时不启用 GPU**。

### 方案 3：使用 conda 管理 CUDA 环境

使用 conda 可以更好地隔离 CUDA 库，避免系统级依赖问题：

```bash
# 安装 miniconda（如果还没有）
wget https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh
bash Miniconda3-latest-Linux-x86_64.sh

# 创建新的 conda 环境，自动安装 CUDA 库
conda create -n piper-gpu python=3.12
conda activate piper-gpu
conda install -c conda-forge cudatoolkit=12.4

# 安装其他依赖
pip install piper-tts onnxruntime-gpu fastapi uvicorn
```

---

## 推荐方案

### 优先级 1：尝试安装 cuda-runtime-12-4（最简单）

```bash
sudo apt-get install -y cuda-runtime-12-4
sudo ldconfig
export LD_LIBRARY_PATH=/usr/local/cuda-12.4/lib64:$LD_LIBRARY_PATH
```

### 优先级 2：如果方案 1 不行，接受 CPU 模式

TTS 的 CPU 性能通常已经足够，GPU 加速是可选的优化。

### 优先级 3：如果必须使用 GPU，考虑 conda

使用 conda 可以更好地管理 CUDA 环境，避免系统级依赖问题。

---

## 结论

**不建议降级 Ubuntu**，因为：
1. 需要重新配置整个环境
2. 可能引入其他兼容性问题
3. TTS 的 CPU 性能通常已经足够

**建议**：
1. 先尝试安装 `cuda-runtime-12-4`（只包含运行时库）
2. 如果不行，暂时使用 CPU 模式
3. 如果必须使用 GPU，考虑使用 conda 环境

---

**最后更新**: 2025-11-28

