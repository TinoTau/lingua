# Whisper 实时 ASR 历史缓冲区方案设计说明

> 交付对象：后端 / 引擎开发人员  
> 适用场景：实时语音流中，使用 Whisper ASR + VAD + 说话人识别（Speaker Embedding）  
> 目标：在不显著增加延迟的前提下，让 ASR 和 Speaker 模型都能看到**足够长且连续**的语音片段，避免句子被严重截断。

---

## 1. 背景与问题概述

现有实时管线（简化）：

1. 录音设备持续产生音频帧（16 kHz 单声道）。
2. 每帧音频写入 `audio_buffer`。
3. Silero VAD 对 `audio_buffer` 中的帧做有声 / 静音判断。
4. 当检测到“静音超过阈值”或“分片时长超限”时：
   - 触发一次“段落结束”事件；
   - 将当前 `audio_buffer` 作为一个 segment 送入 Whisper ASR；
   - （可选）同一段也用于说话人识别（Speaker Embedding）；
   - 随后 **清空 `audio_buffer`**。

问题：

- 人类说话中存在自然停顿（例如 500–800 ms 思考时间），容易被 VAD 误判为“句子结束”；
- 每次分段后就清空 `audio_buffer`，导致：
  - ASR 只能看到 600–1500 ms 的短片段，句子被截成多段；
  - Speaker 模型经常只拿到不足 1 秒的语音，难以稳定提取说话人特征，只能频繁回退默认音色。

**根因归纳：**  
> 「缓冲区被过早、过于频繁地切分和清空，无法为下游模块提供足够长、上下文连续的音频。」

为解决此问题，引入 **历史缓冲区（history buffer）** 方案。

---

## 2. 历史缓冲区方案：总体思路与工作流程

### 2.1 核心思路

在 `WhisperAsrStreaming` 中维护两类缓冲区：

- `audio_buffer`：当前实时片段，用于 ASR 的短延迟转写；
- `history_buffer`：历史窗口，仅用于为 Speaker / ASR 提供更长上下文。

每次 VAD 触发段落结束时：

1. 先将当前 `audio_buffer` 的内容追加到 `history_buffer`；
2. 对 `history_buffer` 做裁剪，只保留最近 N 秒（推荐 2–3 秒）；
3. 再清空 `audio_buffer`，用于下一段输入；
4. 当需要做说话人识别（或较完整的句子 ASR）时：
   - 使用 `history_buffer + audio_buffer` 的合并结果作为输入；
   - 若合并后时长仍不足阈值（如 1000 ms），跳过本次识别。

### 2.2 时序示意（文字版）

1. 【录音阶段】
   - 音频帧不断进入 `audio_buffer`。
2. 【VAD 检测到语音 → 静音过渡】
   - 满足“连续静音超过 X ms”或“累计时长超过 Y ms”时，认为当前 segment 结束。
3. 【分段处理】
   - Whisper 使用 `audio_buffer` 做一次 ASR 结果输出（短句 / 子句）。
   - 将 `audio_buffer` 追加到 `history_buffer`。
   - 对 `history_buffer` 按样本数裁剪，只保留最近 N 秒。
   - 清空 `audio_buffer`。
4. 【说话人识别触发】（可由 VAD 边界、时间间隔或业务逻辑决定）
   - 从 `history_buffer` 和 `audio_buffer` 取出所有帧，合并为一段连续音频。
   - 若合并后的有效语音时长 >= MIN_SPEECH_MS（例如 1200 ms），则送入 Speaker 模型。
   - 否则跳过本次说话人识别，保留现有说话人 embedding。

> 注意：ASR 每次仍可只依赖 `audio_buffer` 保持较低延迟，而 Speaker 可使用更长的历史窗口，二者逻辑解耦。

---

## 3. 落地步骤（实现级别 Checklist）

本节适合直接交给开发人员作为执行清单。

### Step 1：在流式 ASR 结构体中增加历史缓冲区

示例（Rust）—— 请按实际项目结构调整：

```rust
pub struct WhisperAsrStreaming {
    /// 实时短片段缓冲，用于当前一次 ASR 推理
    audio_buffer: Arc<Mutex<Vec<AudioFrame>>>,
    /// 历史缓冲区，用于为 Speaker / ASR 提供足够长的上下文
    history_buffer: Arc<Mutex<Vec<AudioFrame>>>,
    // 其他字段省略……
}
```

`AudioFrame` 至少需要包含：

- PCM 数据或其引用（样本数可知）；
- 可选：时间戳、VAD 标签（voiced/silent），用于调试或过滤。

### Step 2：新增“裁剪历史缓冲区”的工具函数

目标：当 `history_buffer` 的总样本数超过阈值（例如 3 秒），从 **最旧** 的帧开始丢弃，形成一个“滑动窗口”。

```rust
const SAMPLE_RATE: usize = 16_000;
const HISTORY_SECONDS: f32 = 3.0;
const MAX_HISTORY_SAMPLES: usize = (SAMPLE_RATE as f32 * HISTORY_SECONDS) as usize;

fn trim_history_buffer(history: &mut Vec<AudioFrame>) {
    // 计算总样本数（假设每个 frame 都有 len() 方法返回样本数）
    let mut total_samples: usize = history.iter().map(|f| f.len()).sum();

    while total_samples > MAX_HISTORY_SAMPLES {
        if history.is_empty() {
            break;
        }
        // 丢弃最旧的一帧
        let first_len = history[0].len();
        history.remove(0);
        total_samples = total_samples.saturating_sub(first_len);
    }
}
```

### Step 3：重写“清空缓冲区”的逻辑为“保存并清空”

将原先类似：

```rust
fn clear_buffer(&mut self) {
    self.audio_buffer.lock().unwrap().clear();
}
```

调整为：

```rust
fn save_and_clear_buffer(&mut self) {
    let mut audio = self.audio_buffer.lock().unwrap();
    let mut history = self.history_buffer.lock().unwrap();

    // 1. 将当前 audio_buffer 中的帧追加到历史缓冲区
    if !audio.is_empty() {
        history.extend(audio.iter().cloned());
    }

    // 2. 控制历史缓冲区长度，只保留最近 N 秒
    trim_history_buffer(&mut history);

    // 3. 清空当前片段，供下一次使用
    audio.clear();
}
```

然后在 VAD 判断“段落结束”后，所有原来调用 `clear_buffer()` 的地方统一替换为 `save_and_clear_buffer()`。

### Step 4：为说话人识别提供合并后的音频窗口

新增一个工具方法，用于为 Speaker 模型生成输入：

```rust
impl WhisperAsrStreaming {
    /// 获取用于说话人识别的音频窗口：history + current
    pub fn get_speaker_embedding_frames(&self) -> Vec<AudioFrame> {
        let history = self.history_buffer.lock().unwrap();
        let audio = self.audio_buffer.lock().unwrap();

        // 简单拼接：旧在前，新在后，保持时间顺序
        let mut combined = Vec::with_capacity(history.len() + audio.len());
        combined.extend(history.iter().cloned());
        combined.extend(audio.iter().cloned());

        combined
    }
}
```

若需要进一步只保留“有声帧”（依赖 VAD 标记）：

```rust
pub fn get_voiced_speaker_frames(&self) -> Vec<AudioFrame> {
    self.get_speaker_embedding_frames()
        .into_iter()
        .filter(|f| f.is_voiced())
        .collect()
}
```

### Step 5：为说话人识别增加“最小时长”保护

增加一个工具方法，用于计算帧列表对应的时长：

```rust
fn frames_duration_ms(frames: &[AudioFrame]) -> u32 {
    let total_samples: usize = frames.iter().map(|f| f.len()).sum();
    (total_samples as u32 * 1000) / SAMPLE_RATE as u32
}
```

在真正调用 Speaker 模型前：

```rust
const MIN_SPEECH_MS_FOR_SPEAKER: u32 = 1000; // 1 秒，可根据实验调整为 1200–1500 ms

pub fn maybe_run_speaker_embedding(&self) -> SpeakerResult {
    let frames = self.get_speaker_embedding_frames();
    let dur_ms = frames_duration_ms(&frames);

    if dur_ms < MIN_SPEECH_MS_FOR_SPEAKER {
        log::warn!(
            "[SPEAKER] audio too short for embedding: {} ms < {} ms, skip",
            dur_ms,
            MIN_SPEECH_MS_FOR_SPEAKER,
        );
        return SpeakerResult::UsePreviousOrDefault;
    }

    // TODO: 将 frames 拼成连续 PCM buffer，送入 Speaker 模型做推理
    // run_speaker_model(pcm_buffer)...

    SpeakerResult::Success(/* ... */)
}
```

调用时机可以是：

- 每次 VAD 触发“段落结束”后；
- 或按时间间隔（例如每 3 秒）轮询。

### Step 6：日志与调试建议

为了验证方案生效，建议在关键节点打印结构化日志：

1. 分段前后：`audio_buffer` / `history_buffer` 的时长（ms）和帧数；
2. `get_speaker_embedding_frames` 合并后时长；
3. Speaker 模型实际成功 / 跳过的次数与原因；
4. ASR 片段的长度分布（用于观察是否仍存在大量过短片段）。

---

## 4. Rust 代码示例（汇总示例）

> 以下为一个简化的、可编译方向的示例，需根据项目实际类型定义进行调整（尤其是 `AudioFrame`、Speaker 模型调用部分）。

```rust
use std::sync::{Arc, Mutex};

const SAMPLE_RATE: usize = 16_000;
const HISTORY_SECONDS: f32 = 3.0;
const MAX_HISTORY_SAMPLES: usize = (SAMPLE_RATE as f32 * HISTORY_SECONDS) as usize;
const MIN_SPEECH_MS_FOR_SPEAKER: u32 = 1000; // 最小 1s 语音才做说话人识别

#[derive(Clone)]
pub struct AudioFrame {
    // 示例：实际项目中可包含时间戳、VAD 标签等
    pub samples: Vec<f32>,
    pub is_voiced: bool,
}

impl AudioFrame {
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_voiced(&self) -> bool {
        self.is_voiced
    }
}

pub enum SpeakerResult {
    Success,               // 正常返回 embedding（此处省略具体类型）
    UsePreviousOrDefault,  // 保持上次 embedding 或退回默认
}

pub struct WhisperAsrStreaming {
    audio_buffer: Arc<Mutex<Vec<AudioFrame>>>,
    history_buffer: Arc<Mutex<Vec<AudioFrame>>>,
}

impl WhisperAsrStreaming {
    pub fn new() -> Self {
        Self {
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            history_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 外部录音回调将帧写入 audio_buffer
    pub fn push_frame(&self, frame: AudioFrame) {
        let mut audio = self.audio_buffer.lock().unwrap();
        audio.push(frame);
    }

    /// 在 VAD 认为“当前段结束”后调用：
    /// 1. Whisper 用当前 audio_buffer 做一次 ASR；
    /// 2. 再调用本函数保存历史并清空。
    pub fn save_and_clear_buffer(&self) {
        let mut audio = self.audio_buffer.lock().unwrap();
        let mut history = self.history_buffer.lock().unwrap();

        if !audio.is_empty() {
            history.extend(audio.iter().cloned());
        }

        trim_history_buffer(&mut history);

        audio.clear();
    }

    /// 获取 history + current 的合并帧，用于说话人识别
    pub fn get_speaker_embedding_frames(&self) -> Vec<AudioFrame> {
        let history = self.history_buffer.lock().unwrap();
        let audio = self.audio_buffer.lock().unwrap();

        let mut combined = Vec::with_capacity(history.len() + audio.len());
        combined.extend(history.iter().cloned());
        combined.extend(audio.iter().cloned());
        combined
    }

    /// 可选：只返回有声帧
    pub fn get_voiced_speaker_frames(&self) -> Vec<AudioFrame> {
        self.get_speaker_embedding_frames()
            .into_iter()
            .filter(|f| f.is_voiced())
            .collect()
    }

    /// 上层在合适的时机触发说话人识别
    pub fn maybe_run_speaker_embedding(&self) -> SpeakerResult {
        let frames = self.get_voiced_speaker_frames();
        let dur_ms = frames_duration_ms(&frames);

        if dur_ms < MIN_SPEECH_MS_FOR_SPEAKER {
            log::warn!(
                "[SPEAKER] audio too short for embedding: {} ms < {} ms, skip",
                dur_ms,
                MIN_SPEECH_MS_FOR_SPEAKER,
            );
            return SpeakerResult::UsePreviousOrDefault;
        }

        // TODO: 将多帧拼接成一个连续的 PCM buffer，然后调用实际的 Speaker 模型
        // let pcm: Vec<f32> = frames.into_iter().flat_map(|f| f.samples).collect();
        // run_speaker_model(&pcm);

        SpeakerResult::Success
    }
}

/// 裁剪历史缓冲区，只保留最近 N 秒
pub fn trim_history_buffer(history: &mut Vec<AudioFrame>) {
    let mut total_samples: usize = history.iter().map(|f| f.len()).sum();

    while total_samples > MAX_HISTORY_SAMPLES {
        if history.is_empty() {
            break;
        }
        let first_len = history[0].len();
        history.remove(0);
        total_samples = total_samples.saturating_sub(first_len);
    }
}

/// 计算一组帧的时长（毫秒）
pub fn frames_duration_ms(frames: &[AudioFrame]) -> u32 {
    let total_samples: usize = frames.iter().map(|f| f.len()).sum();
    (total_samples as u32 * 1000) / SAMPLE_RATE as u32
}
```

---

## 5. 实现过程中需要注意的坑

### 5.1 历史缓冲区过长导致内存浪费

- 必须通过样本数控制历史窗口长度，例如限制为 2–3 秒；
- 不建议简单用“帧数量”限制，因为不同帧长度可能不一致。

### 5.2 合并顺序错误导致时间反转

- 合并时必须保证顺序为 **history → current**；
- 若顺序反向，Speaker 模型看到的音频时间线会被打乱，可能影响特征稳定性。

### 5.3 说话人识别在语音不足时仍被频繁调用

- 若不加最小时长过滤，模型会反复在 200–500 ms 的短音上推理：
  - 既浪费算力；
  - 又容易得到噪声很大的 embedding，反而拉低效果。
- 必须加上「不足 1–1.5 秒则跳过」的保护。

### 5.4 ASR 与 Speaker 逻辑耦合过紧

- 历史缓冲区方案的目标之一，是 **弱化 ASR 与 Speaker 之间的耦合**：
  - ASR 照旧可以以较小分片频率输出转写结果，保证交互实时性；
  - Speaker 通过更长的历史窗口单独判断何时做识别。
- 不建议在 ASR 的每一次输出处都同步强制触发 Speaker 推理，可以：
  - 只在“端点较明显”的场景触发；
  - 或按时间间隔触发。

### 5.5 VAD 参数与历史缓冲区方案要联合调优

- 如果 VAD 的静音阈值过短（例如 400–500 ms）:
  - 即便历史缓冲区工作正常，也可能得到很多“半句 + 停顿”的组合；
- 建议：
  - `min_silence_duration_ms` 适当拉长到 800–1500 ms；
  - 若有可能，结合业务场景做 A/B 测试，观察 ASR 句子完整度和用户体验。

---

## 6. 验收与测试建议

### 6.1 日志级别验收

至少验证以下指标：

1. **Speaker 输入时长分布**：
   - 统计合并后用于 Speaker 的音频时长（ms）；
   - 合理的分布应主要集中在 1000–3000 ms 区间；
   - 若仍以 < 800 ms 为主，说明历史缓冲区或 VAD 参数需要继续调整。

2. **ASR 片段与真实句子的映射关系**：
   - 长句子应不再被切成 4–5 段短片，而是 1–2 段可理解的子句；
   - 且在文本层面可通过简单拼接获得自然连贯的句子。

3. **Speaker 回退默认音色的频率**：
   - 方案生效后，`UsePreviousOrDefault` 的比例应明显下降；
   - 正常说话 1–2 句后，应能稳定得到当前说话人的 embedding。

### 6.2 人工体验测试

- 准备一些带自然停顿的长句语料（如 8–12 秒）；
- 对比：
  - 历史缓冲区方案开启 / 关闭时，ASR 输出的句子完整度；
  - 说话人识别是否稳定锁定主讲人。

---

## 7. 总结

历史缓冲区方案通过在流式 ASR 结构中引入 `history_buffer`：

1. 避免了每次分段后信息被完全丢弃的问题；
2. 为说话人识别提供了 1–3 秒的稳定上下文窗口；
3. 兼顾了实时性（ASR 仍可用短片段输出）与完整性（Speaker / 上层逻辑可基于更长上下文判断）；
4. 具备较小的入侵性，主要改动集中于缓冲区管理和推理入口层。

建议在当前工程中以本方案为基础落地，并结合实际业务场景，通过参数调优与日志分析进一步优化体验。

