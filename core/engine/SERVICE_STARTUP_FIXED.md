# 服务启动问题修复

## 已修复的问题

### 1. torchaudio 兼容性问题 ✅

**问题**：`AttributeError: module 'torchaudio' has no attribute 'list_audio_backends'`

**修复**：
- ✅ 在 `speaker_embedding_service.py` 中添加了自动兼容性修复
- ✅ 在导入 SpeechBrain 之前修补 `torchaudio.list_audio_backends`
- ✅ 在导入 SpeechBrain 之前修补 `speechbrain.utils.torch_audio_backend` 模块

**如果修复不工作，手动降级**：
```bash
pip install 'torchaudio<2.9'
```

### 2. TTS 模块缺失 ✅

**问题**：`ModuleNotFoundError: No module named 'TTS'`

**修复**：
- ✅ 在 `yourtts_service.py` 中添加了自动安装检查
- ✅ 如果 TTS 未安装，会提示安装

**手动安装**：
```bash
pip install TTS
```

## 安装依赖

### 一键安装
```bash
pip install flask numpy torch 'torchaudio<2.9' soundfile speechbrain TTS
```

### 检查依赖
```bash
python core/engine/scripts/check_dependencies.py
```

## 启动服务

### 方式 1：使用启动脚本

**Windows**：
```powershell
.\core\engine\scripts\start_services.ps1
```

**Linux/Mac**：
```bash
chmod +x core/engine/scripts/start_services.sh
./core/engine/scripts/start_services.sh
```

### 方式 2：手动启动

**终端 1 - Speaker Embedding 服务（GPU 模式）**：
```bash
python core/engine/scripts/speaker_embedding_service.py --gpu
```

**终端 2 - YourTTS 服务（GPU 模式）**：
```bash
python core/engine/scripts/yourtts_service.py --gpu
```

## 验证服务运行

### 健康检查
```bash
curl http://127.0.0.1:5003/health  # Speaker Embedding
curl http://127.0.0.1:5004/health  # YourTTS
```

### Python 验证
```python
import requests

# Speaker Embedding
r = requests.get("http://127.0.0.1:5003/health")
print("Speaker Embedding:", r.json())

# YourTTS
r = requests.get("http://127.0.0.1:5004/health")
print("YourTTS:", r.json())
```

## 故障排除

如果服务仍然无法启动：

1. **检查依赖**：
   ```bash
   python core/engine/scripts/check_dependencies.py
   ```

2. **安装缺失的依赖**：
   ```bash
   pip install flask numpy torch 'torchaudio<2.9' soundfile speechbrain TTS
   ```

3. **查看详细错误**：
   - 服务启动时会显示详细错误信息
   - 检查错误消息中的具体问题

4. **参考文档**：
   - `TROUBLESHOOTING.md` - 详细故障排除指南
   - `QUICK_FIX.md` - 快速修复指南
   - `INSTALL_DEPENDENCIES.md` - 依赖安装指南

