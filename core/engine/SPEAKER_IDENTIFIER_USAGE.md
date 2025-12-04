# Speaker Identifier 使用指南

## 概述

Speaker Identifier 支持两种模式，可以通过配置切换：

1. **VAD 基于边界模式**（免费用户）：通过时间间隔判断说话者切换
2. **Embedding 基于模式**（付费用户）：使用 Speaker Embedding 模型准确识别说话者

## 配置方式

### 1. VAD 基于边界模式（免费用户）

```rust
use core_engine::*;

let engine = CoreEngineBuilder::new()
    // ... 其他配置 ...
    .with_speaker_identification(
        SpeakerIdentifierMode::VadBased {
            min_switch_interval_ms: 1000,  // 1秒内切换认为是插话（新说话者）
            max_same_speaker_interval_ms: 5000,  // 5秒以上认为是新说话者
        }
    )?
    .with_speaker_voice_mapping(vec![
        "zh_CN-huayan-medium".to_string(),
        "zh_CN-xiaoyan-medium".to_string(),
        "en_US-lessac-medium".to_string(),
    ])
    .with_continuous_mode(true, 5000, 200)  // 启用连续模式
    .build()?;
```

### 2. Embedding 基于模式（付费用户）

```rust
use core_engine::*;

let engine = CoreEngineBuilder::new()
    // ... 其他配置 ...
    .with_speaker_identification(
        SpeakerIdentifierMode::EmbeddingBased {
            model_path: "models/speaker_embedding.onnx".to_string(),
            similarity_threshold: 0.7,  // 相似度阈值
        }
    )?
    .with_speaker_voice_mapping(vec![
        "zh_CN-huayan-medium".to_string(),
        "zh_CN-xiaoyan-medium".to_string(),
    ])
    .with_continuous_mode(true, 5000, 200)
    .build()?;
```

## 工作流程

1. **VAD 检测边界** → 检测到语音边界
2. **Speaker Identifier 识别说话者** → 判断是新说话者还是同一人
3. **ASR 识别** → 生成文本，包含 `speaker_id`
4. **NMT 翻译** → 传递 `speaker_id` 到翻译结果
5. **TTS 合成** → 根据 `speaker_id` 选择对应的 voice

## 使用场景

### 场景 1：轮流说话

```
用户A: "Hello" (0-2000ms)
[VAD 边界]
用户B: "Hi there" (2000-4000ms)
[VAD 边界]
用户A: "How are you?" (4000-6000ms)
```

**识别结果**：
- 0ms: speaker_1 (用户A)
- 2000ms: speaker_2 (用户B，间隔 2000ms > 1000ms，新说话者)
- 4000ms: speaker_1 (用户A，间隔 2000ms，同一说话者继续)

### 场景 2：插话

```
用户A: "I think we should..." (0-3000ms)
[VAD 边界]
用户B: "Wait!" (3000-3500ms)  // 插话
[VAD 边界]
用户A: "Let me finish" (3500-5000ms)
```

**识别结果**：
- 0ms: speaker_1 (用户A)
- 3000ms: speaker_2 (用户B，间隔 3000ms，新说话者)
- 3500ms: speaker_3 (用户A，间隔 500ms < 1000ms，插话场景，新说话者)

## 配置参数说明

### VAD 基于边界模式

- `min_switch_interval_ms`: 说话者切换的最小时间间隔（毫秒）
  - 如果两个边界之间的间隔小于此值，认为是新说话者（插话）
  - 推荐值：1000ms（1秒）

- `max_same_speaker_interval_ms`: 同一说话者的最大间隔（毫秒）
  - 如果两个边界之间的间隔大于此值，认为是新说话者
  - 推荐值：5000ms（5秒）

### Embedding 基于模式

- `model_path`: Speaker Embedding 模型文件路径
  - 需要 ONNX 格式的模型文件
  - 推荐模型：ECAPA-TDNN

- `similarity_threshold`: 相似度阈值（0.0-1.0）
  - 如果两个 embedding 的相似度 >= 此值，认为是同一说话者
  - 推荐值：0.7

## 注意事项

1. **必须启用连续模式**：Speaker Identifier 只在连续模式下工作
2. **必须配置 Speaker Voice Mapper**：用于为每个说话者分配不同的 TTS 音色
3. **VAD 模式适合简单场景**：对于复杂的多人对话，建议使用 Embedding 模式

## 测试命令

```bash
# 运行单元测试
cargo test --lib speaker_identifier -- --nocapture

# 运行集成测试
cargo test --test speaker_identifier_test -- --nocapture
```

