# SileroVad 边界检测原理

## 概述

SileroVad 使用深度学习模型（ONNX）来检测语音活动，并通过累积静音帧来识别自然停顿边界。

## 核心原理

### 1. 语音活动检测（VAD）

对每一帧音频（512 样本，32ms @ 16kHz），SileroVad 模型会输出一个**语音概率**（speech probability），范围在 0.0-1.0 之间：

```
speech_prob = model(input_audio_frame, hidden_state, sample_rate)
```

- **speech_prob > threshold**：当前帧包含语音
- **speech_prob < threshold**：当前帧是静音

### 2. 静音帧累积

系统会累积**连续静音帧**的数量：

```rust
if is_silence {
    silence_count += 1;  // 静音帧计数递增
} else {
    silence_count = 0;   // 检测到语音，重置计数
}
```

### 3. 静音持续时间计算

根据累积的静音帧数计算静音持续时间：

```rust
silence_duration_ms = (silence_count * frame_size * 1000) / sample_rate
```

例如：
- `frame_size = 512` samples
- `sample_rate = 16000` Hz
- 1 帧 = 512 / 16000 * 1000 = 32ms
- 19 帧 = 19 * 32 = 608ms

### 4. 边界判定

当满足以下条件时，判定为**自然停顿边界**：

```rust
is_boundary = is_silence && silence_duration_ms >= min_silence_duration_ms
```

例如，如果 `min_silence_duration_ms = 600ms`：
- 需要至少 19 帧连续静音（19 * 32ms = 608ms）
- 才会判定为自然停顿边界

## 状态管理

### 隐藏状态（Hidden State）

SileroVad 模型使用**隐藏状态**来维护上下文信息：

- **形状**：`[2, 1, 128]`
- **作用**：存储模型的内部状态，用于跨帧的上下文理解
- **更新**：每次推理后，模型会输出新的隐藏状态，用于下一次推理

这确保了模型能够：
- 理解音频的时序特征
- 更准确地识别语音和静音的转换
- 减少误判（例如，短暂的静音不会立即被判定为边界）

### 静音帧计数重置

当检测到边界时，静音帧计数会被重置：

```rust
if is_boundary {
    silence_count = 0;  // 重置计数，准备检测下一个边界
}
```

## 配置参数

### `silence_threshold`（静音阈值）

- **默认值**：0.5
- **作用**：判断当前帧是否为静音的阈值
- **调整建议**：
  - 如果误判静音为语音，**降低**阈值（例如 0.3）
  - 如果误判语音为静音，**提高**阈值（例如 0.7）

### `min_silence_duration_ms`（最小静音时长）

- **默认值**：600ms
- **作用**：判定为自然停顿所需的最短静音时长
- **调整建议**：
  - 如果边界检测太频繁，**增加**时长（例如 800ms）
  - 如果边界检测太慢，**减少**时长（例如 400ms）

### `frame_size`（帧大小）

- **默认值**：512 samples（32ms @ 16kHz）
- **作用**：每次处理的音频样本数
- **注意**：这是 SileroVad 模型的要求，不建议修改

## 工作流程示例

假设配置：
- `silence_threshold = 0.5`
- `min_silence_duration_ms = 600ms`
- `frame_size = 512` (32ms)

### 场景：说话 -> 停顿 -> 说话

```
时间轴：0ms    32ms   64ms   96ms   ...   320ms  352ms  384ms  ...  608ms  640ms
帧：    [语音] [语音] [语音] [语音] ...  [语音] [静音] [静音] ... [静音] [语音]
概率：  0.8    0.7    0.6    0.5    ...   0.6    0.3    0.2    ...  0.1    0.7
计数：  0      0      0      0      ...   0      1      2      ...  19     0
边界：  -      -      -      -      ...   -      -      -      ...  ✅     -
```

**详细过程**：

1. **0-320ms**：语音帧
   - `speech_prob > 0.5` → `is_silence = false`
   - `silence_count = 0`
   - `is_boundary = false`

2. **352ms**：第一个静音帧
   - `speech_prob = 0.3 < 0.5` → `is_silence = true`
   - `silence_count = 1`
   - `silence_duration_ms = 1 * 32 = 32ms < 600ms`
   - `is_boundary = false`

3. **384-576ms**：继续静音
   - `silence_count` 递增：2, 3, ..., 18
   - `silence_duration_ms` 递增：64ms, 96ms, ..., 576ms
   - `is_boundary = false`（未达到 600ms 阈值）

4. **608ms**：达到阈值
   - `silence_count = 19`
   - `silence_duration_ms = 19 * 32 = 608ms >= 600ms`
   - `is_boundary = true` ✅ **检测到自然停顿边界**
   - `silence_count = 0`（重置）

5. **640ms**：新的语音帧
   - `speech_prob = 0.7 > 0.5` → `is_silence = false`
   - `silence_count = 0`
   - `is_boundary = false`

## 常见问题

### Q1: 为什么边界检测太频繁？

**可能原因**：
1. `min_silence_duration_ms` 设置太小
2. `silence_threshold` 设置太高，导致误判语音为静音
3. 音频质量差，导致模型输出不稳定

**解决方案**：
- 增加 `min_silence_duration_ms`（例如 800ms）
- 降低 `silence_threshold`（例如 0.3）

### Q2: 为什么边界检测太慢？

**可能原因**：
1. `min_silence_duration_ms` 设置太大
2. `silence_threshold` 设置太低，导致误判静音为语音

**解决方案**：
- 减少 `min_silence_duration_ms`（例如 400ms）
- 提高 `silence_threshold`（例如 0.7）

### Q3: 为什么每次只处理很少的帧？

**可能原因**：
1. 在非连续模式下，每次检测到边界后，ASR 缓冲区被清空
2. 边界检测太频繁，导致缓冲区在累积足够帧之前就被清空

**解决方案**：
- 调整 `min_silence_duration_ms`，减少边界检测频率
- 检查 ASR 缓冲区的累积逻辑

### Q4: 隐藏状态的作用是什么？

**作用**：
- 维护模型的上下文信息
- 帮助模型理解音频的时序特征
- 提高语音/静音判别的准确性

**注意**：
- 隐藏状态在每次推理后更新
- 如果重置 VAD（调用 `reset()`），隐藏状态会被清空

## 调试建议

### 启用调试日志

代码中已经添加了调试日志，会输出：
- 边界检测时的静音持续时间
- 静音帧计数
- 语音概率

### 监控指标

关注以下指标：
1. **静音持续时间**：是否达到阈值
2. **静音帧计数**：是否正确累积
3. **语音概率**：是否在合理范围内（0.0-1.0）

### 调整策略

1. **先调整 `min_silence_duration_ms`**：
   - 如果边界太频繁，增加时长
   - 如果边界太慢，减少时长

2. **再调整 `silence_threshold`**：
   - 如果误判，调整阈值

3. **观察日志**：
   - 查看静音持续时间和帧计数
   - 确认是否符合预期

## 参考

- [Silero VAD 官方文档](https://github.com/snakers4/silero-vad)
- [ONNX Runtime 文档](https://onnxruntime.ai/docs/)

