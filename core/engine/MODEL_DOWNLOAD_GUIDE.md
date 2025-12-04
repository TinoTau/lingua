# 模型下载指南

## 快速开始

### 1. Speaker Embedding 模型（音色提取）

**推荐模型**：ECAPA-TDNN

```bash
# 创建模型目录
mkdir -p core/engine/models/speaker_embedding

# 方式1：使用下载脚本（需要手动下载模型文件）
python scripts/download_speaker_embedding_model.py

# 方式2：手动下载
# 从 HuggingFace 下载：https://huggingface.co/speechbrain/spkrec-ecapa-voxceleb
# 转换为 ONNX 格式后保存到：core/engine/models/speaker_embedding/ecapa_tdnn.onnx
```

**模型要求**：
- 格式：ONNX
- 输入：16kHz 单声道音频（f32）
- 输出：512 维特征向量
- 大小：~50MB

### 2. Zero-shot TTS 模型（音色分配）

**推荐模型**：YourTTS

```bash
# 安装和设置
python scripts/download_yourtts_service.py

# 启动服务
python -m TTS.server.server --model_name tts_models/multilingual/multi-dataset/your_tts --port 5002
```

**或使用 Docker**：
```bash
docker run -p 5002:5002 coqui/tts:latest
```

## 模型列表

| 模型类型 | 模型名称 | 大小 | 用途 | 下载方式 |
|---------|---------|------|------|---------|
| Speaker Embedding | ECAPA-TDNN | ~50MB | 提取音色特征 | HuggingFace / SpeechBrain |
| Zero-shot TTS | YourTTS | ~500MB | 根据参考音频生成语音 | Coqui TTS |
| Zero-shot TTS | VALL-E X | ~1GB | 高质量语音克隆 | GitHub |
| Zero-shot TTS | StyleSpeech | ~500MB | 风格化语音合成 | GitHub |

## 详细说明

### ECAPA-TDNN（Speaker Embedding）

**下载链接**：
- HuggingFace: https://huggingface.co/speechbrain/spkrec-ecapa-voxceleb
- SpeechBrain: https://speechbrain.github.io/

**转换 ONNX**：
```python
# 见 scripts/download_speaker_embedding_model.py
```

### YourTTS（Zero-shot TTS）

**安装**：
```bash
pip install TTS
```

**使用**：
```python
from TTS.api import TTS

tts = TTS("tts_models/multilingual/multi-dataset/your_tts")
tts.tts_to_file(
    text="Hello",
    speaker_wav="reference.wav",  # 参考音频
    file_path="output.wav"
)
```

## 当前项目模型位置

```
core/engine/models/
  ├── asr/whisper-base/          ✅ 已有
  ├── emotion/xlm-r/             ✅ 已有
  ├── nmt/m2m100-en-zh/          ✅ 已有
  ├── tts/vits-zh-aishell3/      ✅ 已有
  ├── vad/silero/                ✅ 已有
  └── speaker_embedding/         ⭐ 需要添加
      └── ecapa_tdnn.onnx        ⭐ 需要下载
```

## 下一步

1. **下载 ECAPA-TDNN 模型** → 实现音色提取
2. **设置 YourTTS 服务** → 实现音色分配
3. **集成到代码** → 修改 `embedding_based.rs` 和 TTS 逻辑

