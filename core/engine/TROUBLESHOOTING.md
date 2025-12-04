# 故障排除指南

## 常见错误和解决方案

### 1. Speaker Embedding 服务：torchaudio 兼容性问题

**错误**：
```
AttributeError: module 'torchaudio' has no attribute 'list_audio_backends'
```

**原因**：
- torchaudio 2.9+ 移除了 `list_audio_backends` 方法
- SpeechBrain 依赖这个方法

**解决方案**：

**方案 1：使用兼容性修复（推荐）**
- 服务脚本已自动应用修复
- 如果仍然失败，检查修复是否生效

**方案 2：降级 torchaudio**
```bash
pip install 'torchaudio<2.9'
```

**方案 3：手动应用修复**
在导入 SpeechBrain 之前运行：
```python
import torchaudio
if not hasattr(torchaudio, 'list_audio_backends'):
    def mock_list_audio_backends():
        return ['soundfile']
    torchaudio.list_audio_backends = mock_list_audio_backends
```

### 2. YourTTS 服务：缺少 TTS 模块

**错误**：
```
ModuleNotFoundError: No module named 'TTS'
```

**解决方案**：
```bash
pip install TTS
```

**注意**：TTS 模块较大（~500MB），安装可能需要一些时间。

### 3. 检查依赖

运行依赖检查脚本：
```bash
python core/engine/scripts/check_dependencies.py
```

或使用服务脚本的检查功能：
```bash
python core/engine/scripts/speaker_embedding_service.py --check-deps
python core/engine/scripts/yourtts_service.py --check-deps
```

### 4. 安装所有依赖

**一次性安装所有依赖**：
```bash
pip install flask numpy torch torchaudio soundfile speechbrain TTS
```

**分步安装**：

1. 基础依赖：
```bash
pip install flask numpy torch torchaudio soundfile
```

2. Speaker Embedding 依赖：
```bash
pip install speechbrain
```

3. YourTTS 依赖：
```bash
pip install TTS
```

### 5. GPU 不可用

**现象**：
```
⚠️  GPU requested but not available, using CPU
```

**检查 GPU**：
```python
import torch
print(f"CUDA available: {torch.cuda.is_available()}")
if torch.cuda.is_available():
    print(f"CUDA device: {torch.cuda.get_device_name(0)}")
```

**解决方案**：
- 如果没有 GPU，服务会自动使用 CPU（功能正常，但较慢）
- 如果有 GPU 但不可用，检查 CUDA 和 PyTorch 安装

### 6. 模型文件未找到

**错误**：
```
FileNotFoundError: Model not found at ...
```

**检查模型路径**：
- Speaker Embedding: `core/engine/models/speaker_embedding/cache`
- YourTTS: `core/engine/models/tts/your_tts`

**解决方案**：
- 确保模型文件已下载
- 检查路径是否正确
- 使用绝对路径

### 7. 端口被占用

**错误**：
```
OSError: [Errno 48] Address already in use
```

**解决方案**：

**Windows**：
```powershell
# 查找占用端口的进程
netstat -ano | findstr :5003
netstat -ano | findstr :5004

# 停止进程（替换 PID）
taskkill /PID <PID> /F
```

**Linux/Mac**：
```bash
# 查找占用端口的进程
lsof -i :5003
lsof -i :5004

# 停止进程
kill <PID>
```

**或使用其他端口**：
```bash
python core/engine/scripts/speaker_embedding_service.py --port 5005
python core/engine/scripts/yourtts_service.py --port 5006
```

### 8. 内存不足

**错误**：
```
RuntimeError: CUDA out of memory
```

**解决方案**：
- 使用 CPU 模式（不添加 `--gpu` 参数）
- 关闭其他占用 GPU 的程序
- 减少批处理大小

## 验证修复

### 1. 验证 torchaudio 修复

```python
import torchaudio
if not hasattr(torchaudio, 'list_audio_backends'):
    print("torchaudio 2.9+ detected, compatibility fix needed")
    # 应用修复
    def mock_list_audio_backends():
        return ['soundfile']
    torchaudio.list_audio_backends = mock_list_audio_backends
    print("✅ Fix applied")

# 测试 SpeechBrain 导入
try:
    from speechbrain.inference.speaker import EncoderClassifier
    print("✅ SpeechBrain import successful")
except Exception as e:
    print(f"❌ SpeechBrain import failed: {e}")
```

### 2. 验证 TTS 模块

```python
try:
    from TTS.api import TTS
    print("✅ TTS module available")
except ImportError:
    print("❌ TTS module not found")
    print("Install with: pip install TTS")
```

### 3. 验证服务启动

```bash
# 检查依赖
python core/engine/scripts/check_dependencies.py

# 尝试启动服务（会显示详细错误信息）
python core/engine/scripts/speaker_embedding_service.py --gpu
python core/engine/scripts/yourtts_service.py --gpu
```

## 快速修复脚本

创建 `fix_dependencies.ps1`（Windows）：

```powershell
# 安装所有依赖
pip install flask numpy torch torchaudio soundfile speechbrain TTS

# 检查依赖
python core/engine/scripts/check_dependencies.py
```

创建 `fix_dependencies.sh`（Linux/Mac）：

```bash
#!/bin/bash
# 安装所有依赖
pip install flask numpy torch torchaudio soundfile speechbrain TTS

# 检查依赖
python core/engine/scripts/check_dependencies.py
```

## 联系支持

如果问题仍然存在：
1. 运行依赖检查：`python core/engine/scripts/check_dependencies.py`
2. 查看详细错误信息
3. 检查 Python 版本（推荐 3.8+）
4. 检查系统要求

