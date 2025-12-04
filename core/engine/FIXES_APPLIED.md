# 已应用的修复

## 问题 1: torchaudio 兼容性错误

### 错误信息
```
AttributeError: module 'torchaudio' has no attribute 'list_audio_backends'
```

### 修复方案

**已实现**：
1. ✅ 在 `speaker_embedding_service.py` 中添加了兼容性修复
2. ✅ 在导入 SpeechBrain 之前修补 `torchaudio.list_audio_backends`
3. ✅ 在导入 SpeechBrain 之前修补 `speechbrain.utils.torch_audio_backend` 模块

**修复代码位置**：
- `core/engine/scripts/speaker_embedding_service.py` (lines 32-100)

**如何工作**：
1. 检测 torchaudio 2.9+（没有 `list_audio_backends` 方法）
2. 创建模拟的 `list_audio_backends` 函数
3. 在 SpeechBrain 导入前修补 backend 检查模块

### 如果修复不工作

**手动降级 torchaudio**：
```bash
pip install 'torchaudio<2.9'
```

## 问题 2: TTS 模块缺失

### 错误信息
```
ModuleNotFoundError: No module named 'TTS'
```

### 修复方案

**已实现**：
1. ✅ 在 `yourtts_service.py` 中添加了自动安装检查
2. ✅ 如果 TTS 未安装，会提示安装

**修复代码位置**：
- `core/engine/scripts/yourtts_service.py` (lines 59-75)

**如何工作**：
1. 检查 TTS 模块是否安装
2. 如果未安装，尝试自动安装
3. 如果安装失败，显示错误信息和安装指令

### 手动安装

```bash
pip install TTS
```

## 依赖检查工具

**已创建**：
- `core/engine/scripts/check_dependencies.py` - 检查所有依赖

**使用方法**：
```bash
python core/engine/scripts/check_dependencies.py
```

## 验证修复

### 1. 检查依赖
```bash
python core/engine/scripts/check_dependencies.py
```

### 2. 测试服务启动
```bash
# Speaker Embedding 服务
python core/engine/scripts/speaker_embedding_service.py --gpu

# YourTTS 服务
python core/engine/scripts/yourtts_service.py --gpu
```

## 如果问题仍然存在

1. **检查 Python 版本**：推荐 3.8+
2. **检查依赖版本**：运行 `check_dependencies.py`
3. **手动安装依赖**：`pip install flask numpy torch 'torchaudio<2.9' soundfile speechbrain TTS`
4. **查看详细错误**：服务启动时会显示详细错误信息

