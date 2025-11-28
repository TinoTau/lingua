# PyTorch CUDA 安装指南

**最后更新**: 2025-11-28

本文档提供在 Lingua NMT 服务中安装 CUDA 版本 PyTorch 的详细步骤。

---

## 📋 前置条件

### 1. 确认系统信息

根据您的系统信息：
- **GPU**: NVIDIA GeForce RTX 4060 Laptop GPU
- **CUDA 版本**: 12.7
- **驱动版本**: 566.26

### 2. 确认 CUDA 可用性

```powershell
# 检查 NVIDIA GPU
nvidia-smi
```

如果显示 GPU 信息，说明驱动已正确安装。

---

## 🔧 安装步骤

### 方法 1: 在 NMT 服务虚拟环境中安装（推荐）

#### 步骤 1: 激活虚拟环境

```powershell
# 进入 NMT 服务目录
cd services\nmt_m2m100

# 激活虚拟环境
.\venv\Scripts\Activate.ps1
```

如果虚拟环境不存在，先创建：

```powershell
cd services\nmt_m2m100
python -m venv venv
.\venv\Scripts\Activate.ps1
pip install -r requirements.txt
```

#### 步骤 2: 检查当前 PyTorch 版本

```powershell
python -c "import torch; print('PyTorch version:', torch.__version__); print('CUDA available:', torch.cuda.is_available())"
```

#### 步骤 3: 卸载 CPU 版本（如果已安装）

```powershell
# 卸载 CPU 版本的 PyTorch
pip uninstall torch torchvision torchaudio -y
```

#### 步骤 4: 安装 CUDA 版本的 PyTorch

**重要**：如果您的 Python 版本是 3.13 或更高，PyTorch 可能还没有提供预编译的 CUDA 版本。请先检查您的 Python 版本：

```powershell
python --version
```

**如果 Python 版本是 3.13+**，有两个选择：

**选择 1：使用 pip 默认源安装（推荐）**

```powershell
# 直接使用 pip 安装，会自动选择兼容的版本
pip install torch torchvision torchaudio
```

然后检查是否支持 CUDA：

```powershell
python -c "import torch; print('CUDA available:', torch.cuda.is_available())"
```

**选择 2：降级到 Python 3.11 或 3.12（如果选择 1 不支持 CUDA）**

重新创建虚拟环境：

```powershell
# 删除旧虚拟环境
Remove-Item -Recurse -Force venv

# 使用 Python 3.11 或 3.12 创建新虚拟环境
# 如果有多个 Python 版本，使用 python3.11 或 python3.12
python3.11 -m venv venv
# 或
python3.12 -m venv venv

# 激活虚拟环境
.\venv\Scripts\Activate.ps1

# 安装依赖
pip install -r requirements.txt

# 卸载 CPU 版本
pip uninstall torch torchvision torchaudio -y

# 安装 CUDA 版本
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
```

**如果 Python 版本是 3.11 或 3.12**，使用以下命令：

根据您的 CUDA 12.7 版本，安装支持 CUDA 12.1 的 PyTorch（向后兼容）：

```powershell
# 安装 CUDA 12.1 版本的 PyTorch（推荐，兼容 CUDA 12.7）
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
```

**或者**，如果您想使用 CUDA 11.8 版本（更稳定，但性能略低）：

```powershell
# 安装 CUDA 11.8 版本的 PyTorch
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu118
```

**或者**，如果您想使用最新的 CUDA 12.4 版本：

```powershell
# 安装 CUDA 12.4 版本的 PyTorch
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu124
```

#### 步骤 5: 验证安装

```powershell
python -c "import torch; print('PyTorch version:', torch.__version__); print('CUDA available:', torch.cuda.is_available()); print('CUDA version:', torch.version.cuda if torch.cuda.is_available() else 'N/A'); print('GPU count:', torch.cuda.device_count()); print('GPU name:', torch.cuda.get_device_name(0) if torch.cuda.is_available() else 'N/A')"
```

**预期输出**：
```
PyTorch version: 2.x.x+cu121
CUDA available: True
CUDA version: 12.1
GPU count: 1
GPU name: NVIDIA GeForce RTX 4060 Laptop GPU
```

#### 步骤 6: 测试 GPU 计算

```powershell
python -c "import torch; x = torch.randn(3, 3).cuda(); print('GPU tensor:', x); print('GPU test passed!')"
```

如果看到 `GPU tensor: tensor([...], device='cuda:0')`，说明 GPU 正常工作。

---

### 方法 2: 使用 pip 直接安装（不推荐，可能影响其他项目）

如果您想在全局 Python 环境中安装：

```powershell
# 卸载 CPU 版本
pip uninstall torch torchvision torchaudio -y

# 安装 CUDA 版本
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
```

**注意**：不推荐在全局环境安装，建议使用虚拟环境。

---

## 🔍 验证 NMT 服务是否使用 GPU

### 方法 1: 修改 NMT 服务代码添加日志

编辑 `services/nmt_m2m100/nmt_service.py`，在 `load_model()` 函数中添加：

```python
@app.on_event("startup")
async def load_model():
    """启动时加载模型"""
    global tokenizer, model
    try:
        print(f"[NMT Service] Loading model: {MODEL_NAME}")
        print(f"[NMT Service] Device: {DEVICE}")
        
        # 添加 GPU 检查
        if torch.cuda.is_available():
            print(f"[NMT Service] CUDA available: {torch.cuda.is_available()}")
            print(f"[NMT Service] CUDA version: {torch.version.cuda}")
            print(f"[NMT Service] GPU count: {torch.cuda.device_count()}")
            print(f"[NMT Service] GPU name: {torch.cuda.get_device_name(0)}")
        else:
            print(f"[NMT Service] WARNING: CUDA not available, using CPU")
        
        # ... 其余代码
```

### 方法 2: 启动服务时观察日志

启动 NMT 服务后，查看日志输出：

```powershell
cd services\nmt_m2m100
.\venv\Scripts\Activate.ps1
uvicorn nmt_service:app --host 127.0.0.1 --port 5008
```

如果看到 `[NMT Service] Device: cuda` 或 `CUDA available: True`，说明正在使用 GPU。

### 方法 3: 使用 nvidia-smi 监控

在另一个终端运行：

```powershell
# 实时监控 GPU 使用情况
nvidia-smi -l 1
```

然后发送一个翻译请求，如果看到 GPU 使用率上升，说明正在使用 GPU。

---

## 🐛 故障排查

### 问题 1: `CUDA available: False`

**可能原因**：
1. PyTorch 版本不匹配
2. CUDA 驱动版本不兼容
3. 虚拟环境中安装的是 CPU 版本

**解决方法**：
```powershell
# 1. 确认已卸载 CPU 版本
pip uninstall torch torchvision torchaudio -y

# 2. 重新安装 CUDA 版本
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121

# 3. 验证
python -c "import torch; print(torch.cuda.is_available())"
```

### 问题 2: `RuntimeError: CUDA out of memory`

**可能原因**：
- GPU 显存不足（您的 RTX 4060 有 8GB，通常足够）

**解决方法**：
1. 关闭其他使用 GPU 的程序
2. 减少 batch size（如果 NMT 服务支持）
3. 使用更小的模型

### 问题 3: `ImportError: DLL load failed`

**可能原因**：
- CUDA 运行时库缺失

**解决方法**：
1. 确保已安装 CUDA Toolkit（不是驱动）
2. 下载地址：https://developer.nvidia.com/cuda-downloads
3. 安装后重启终端

### 问题 4: 安装速度慢

**解决方法**：
```powershell
# 使用国内镜像源（如果可用）
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121 -i https://pypi.tuna.tsinghua.edu.cn/simple
```

---

## 📊 性能对比

### CPU vs GPU 性能（预期）

| 组件 | CPU 耗时 | GPU 耗时 | 提升倍数 |
|------|---------|---------|---------|
| NMT 翻译 | 3-4 秒 | 0.5-1 秒 | 3-4x |
| 模型加载 | 5-10 秒 | 2-3 秒 | 2-3x |

**注意**：实际性能取决于文本长度、模型大小等因素。

---

## 🔄 更新 requirements.txt（可选）

如果您想将 CUDA 版本的 PyTorch 固定到 `requirements.txt`：

```txt
# 在 services/nmt_m2m100/requirements.txt 中添加
--extra-index-url https://download.pytorch.org/whl/cu121
torch>=2.0.0
torchvision>=0.15.0
torchaudio>=2.0.0
```

这样其他人安装依赖时也会自动安装 CUDA 版本。

---

## 📚 相关文档

- [GPU 启用指南](../GPU_启用指南.md)
- [编译和启动命令参考](./编译和启动命令参考.md)
- [PyTorch 官方文档](https://pytorch.org/get-started/locally/)

---

## ✅ 快速检查清单

安装完成后，请确认：

- [ ] `torch.cuda.is_available()` 返回 `True`
- [ ] `torch.cuda.device_count()` 返回 `1`
- [ ] `torch.cuda.get_device_name(0)` 显示您的 GPU 名称
- [ ] NMT 服务启动时显示 `Device: cuda`
- [ ] 发送翻译请求时，`nvidia-smi` 显示 GPU 使用率上升

---

**最后更新**: 2025-11-28

