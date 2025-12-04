# SileroVad 架构说明

## 架构定位

**SileroVad 是集成在 CoreEngine 中的功能模块，不是单独的服务。**

## 模块关系

```
CoreEngine
├── VAD 模块 (Voice Activity Detection)
│   ├── SimpleVad (时间基于的简单 VAD)
│   └── SileroVad (基于 ONNX 的自然停顿检测) ← 这里
├── ASR 模块 (Automatic Speech Recognition)
│   └── WhisperAsrStreaming
├── NMT 模块 (Neural Machine Translation)
├── TTS 模块 (Text-to-Speech)
└── 其他模块...
```

## 工作流程

```
音频输入
  ↓
VAD 检测 (SileroVad)
  ├── 检测语音活动
  ├── 检测自然停顿 (BoundaryType::NaturalPause)
  └── 返回边界信息
  ↓
触发 ASR 推理
  ├── 当检测到边界时
  └── 使用累积的音频帧进行识别
  ↓
后续处理 (NMT, TTS...)
```

## 配置方式

在 `lingua_core_config.toml` 中配置：

```toml
[vad]
type = "silero"  # 或 "simple"
model_path = "models/vad/silero/silero_vad.onnx"
silence_threshold = 0.5
min_silence_duration_ms = 600
```

## 与 ASR 的关系

- **VAD 和 ASR 是独立的模块**
- VAD 负责：检测语音边界，决定何时触发 ASR
- ASR 负责：识别语音内容
- VAD 的输出（边界信息）用于控制 ASR 的推理时机

## 优势

1. **集成在 CoreEngine 中**：无需额外的服务，减少网络开销
2. **自然停顿检测**：比简单的时间基于 VAD 更准确
3. **可配置**：可以通过配置文件切换 SimpleVad 或 SileroVad
4. **低延迟**：直接在 Rust 中运行，无需 HTTP 调用

## 与 Speaker Embedding 服务的对比

| 特性 | SileroVad | Speaker Embedding |
|------|-----------|-------------------|
| 架构 | 集成模块 | 独立 HTTP 服务 |
| 功能 | 语音边界检测 | 说话者识别 |
| 模型 | ONNX (Rust) | PyTorch (Python) |
| 延迟 | 低（本地） | 中等（HTTP） |

