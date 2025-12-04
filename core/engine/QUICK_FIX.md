# 快速修复指南

## 问题 1: torchaudio 兼容性错误

**错误**：
```
AttributeError: module 'torchaudio' has no attribute 'list_audio_backends'
```

**快速修复**：
```bash
# 方案 1：降级 torchaudio（推荐）
pip install 'torchaudio<2.9'

# 方案 2：服务脚本已自动修复，如果仍然失败，检查修复是否生效
```

## 问题 2: TTS 模块缺失

**错误**：
```
ModuleNotFoundError: No module named 'TTS'
```

**快速修复**：
```bash
pip install TTS
```

## 一键安装所有依赖

```bash
pip install flask numpy torch 'torchaudio<2.9' soundfile speechbrain TTS
```

## 检查依赖

```bash
python core/engine/scripts/check_dependencies.py
```

## 启动服务（修复后）

```bash
# Speaker Embedding 服务
python core/engine/scripts/speaker_embedding_service.py --gpu

# YourTTS 服务
python core/engine/scripts/yourtts_service.py --gpu
```

