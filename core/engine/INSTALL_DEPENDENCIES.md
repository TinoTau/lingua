# 依赖安装指南

## 快速安装

### 一键安装所有依赖

```bash
pip install flask numpy torch 'torchaudio<2.9' soundfile speechbrain TTS
```

### 分步安装

#### 1. 基础依赖
```bash
pip install flask numpy torch torchaudio soundfile
```

#### 2. Speaker Embedding 依赖
```bash
pip install speechbrain
```

#### 3. YourTTS 依赖
```bash
pip install TTS
```

## 解决 torchaudio 兼容性问题

### 问题
torchaudio 2.9+ 移除了 `list_audio_backends` 方法，导致 SpeechBrain 无法导入。

### 解决方案

**方案 1：降级 torchaudio（推荐）**
```bash
pip install 'torchaudio<2.9'
```

**方案 2：使用兼容性修复（已集成）**
- 服务脚本已自动应用修复
- 如果仍然失败，检查修复是否生效

## 验证安装

### 检查依赖
```bash
python core/engine/scripts/check_dependencies.py
```

### 测试导入
```python
# 测试基础依赖
import flask
import numpy
import torch
import torchaudio
import soundfile
print("✅ Basic dependencies OK")

# 测试 Speaker Embedding
from speechbrain.inference.speaker import EncoderClassifier
print("✅ SpeechBrain OK")

# 测试 YourTTS
from TTS.api import TTS
print("✅ TTS OK")
```

## 常见问题

### 1. torchaudio 版本冲突

**问题**：已安装 torchaudio 2.9+，但需要 <2.9

**解决**：
```bash
pip uninstall torchaudio
pip install 'torchaudio<2.9'
```

### 2. TTS 安装失败

**问题**：TTS 模块安装失败或很慢

**解决**：
- 检查网络连接
- 使用国内镜像：`pip install TTS -i https://pypi.tuna.tsinghua.edu.cn/simple`
- 分步安装依赖

### 3. CUDA 相关问题

**问题**：PyTorch 不支持 CUDA

**解决**：
- 安装 CUDA 版本的 PyTorch：`pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu118`
- 或使用 CPU 版本（功能正常，但较慢）

## 安装后验证

### 1. 检查依赖
```bash
python core/engine/scripts/check_dependencies.py
```

### 2. 测试服务启动
```bash
# Speaker Embedding 服务
python core/engine/scripts/speaker_embedding_service.py --check-deps

# YourTTS 服务
python core/engine/scripts/yourtts_service.py --check-deps
```

### 3. 启动服务
```bash
# 如果依赖检查通过，启动服务
python core/engine/scripts/speaker_embedding_service.py --gpu
python core/engine/scripts/yourtts_service.py --gpu
```

