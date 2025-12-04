# VAD 自适应调整的作用范围

## 两种方案对比

### 方案 1：按用户单独调整（Per-Speaker）

**原理**：
- 为每个说话者（speaker_id）维护独立的语速统计和 VAD 参数
- 根据每个说话者的实际语速动态调整其对应的 VAD 阈值

**优点**：
- ✅ 精确：不同语速的用户使用不同的阈值
- ✅ 个性化：适应每个用户的说话习惯
- ✅ 适合多人场景：不同人语速差异大时效果好

**缺点**：
- ❌ 实现复杂：需要为每个说话者维护独立状态
- ❌ 内存开销：需要存储每个说话者的统计数据
- ❌ 初始化问题：新用户需要一定时间才能稳定

**示例场景**：
```
用户A（快语速）：min_silence_duration_ms = 400ms
用户B（慢语速）：min_silence_duration_ms = 800ms
用户C（正常）：min_silence_duration_ms = 600ms
```

**实现方式**：
```rust
struct PerSpeakerVadState {
    speaker_states: HashMap<String, SpeakerVadState>,
}

struct SpeakerVadState {
    speech_rate_history: VecDeque<f32>,  // 语速历史
    adjusted_duration_ms: u64,           // 调整后的阈值
    sample_count: usize,                 // 样本数量
}
```

### 方案 2：对整个场景统一调整（Global）

**原理**：
- 统计所有说话者的平均语速
- 使用统一的 VAD 参数，根据整体语速动态调整

**优点**：
- ✅ 实现简单：只需维护一个全局状态
- ✅ 内存开销小：只需存储一份统计数据
- ✅ 快速适应：可以快速响应整体语速变化

**缺点**：
- ❌ 不够精确：如果用户语速差异大，可能不够准确
- ❌ 平均化问题：快语速和慢语速用户可能都不满意

**示例场景**：
```
整体平均语速（快）：min_silence_duration_ms = 450ms（所有人使用）
整体平均语速（慢）：min_silence_duration_ms = 750ms（所有人使用）
```

**实现方式**：
```rust
struct GlobalVadState {
    speech_rate_history: VecDeque<f32>,  // 所有用户的语速历史
    adjusted_duration_ms: u64,           // 调整后的阈值（全局）
    sample_count: usize,                 // 总样本数量
}
```

## 推荐方案

### 场景分析

根据你的描述：
- "多人场景的人是固定的，在整个交流过程中人员变化不大"
- "根据不同用户的实际语速进行调整"

**建议：按用户单独调整（方案 1）**

**理由**：
1. ✅ 你明确提到"不同用户"，说明希望个性化
2. ✅ 人员固定，适合维护每个用户的状态
3. ✅ 多人场景中，不同人语速差异通常较大
4. ✅ 可以更好地适应每个用户的说话习惯

### 混合方案（推荐）

**原理**：
- 默认使用全局调整（简单快速）
- 如果检测到多个说话者且语速差异大，自动切换到按用户调整
- 可以配置选择使用哪种模式

**优点**：
- ✅ 灵活性：可以根据场景选择
- ✅ 渐进式：从简单到复杂
- ✅ 可配置：用户可以选择

**配置示例**：
```toml
[vad]
type = "silero"
adaptive_mode = "per_speaker"  # global, per_speaker, auto

# 全局模式参数
[vad.adaptive.global]
enabled = true
min_samples = 5  # 至少5个样本才开始调整

# 按用户模式参数
[vad.adaptive.per_speaker]
enabled = true
min_samples_per_speaker = 3  # 每个用户至少3个样本
max_speakers = 10  # 最多跟踪10个用户
```

## 实现建议

### 阶段 1：全局调整（快速实现）

先实现全局调整，验证效果：
- 统计所有用户的平均语速
- 根据平均语速调整全局阈值
- 简单快速，适合验证

### 阶段 2：按用户调整（完整实现）

在全局调整基础上，添加按用户调整：
- 为每个说话者维护独立状态
- 根据每个用户的语速调整阈值
- 更精确，适合多人场景

### 阶段 3：混合模式（可选）

添加自动切换逻辑：
- 检测用户语速差异
- 差异大时使用按用户调整
- 差异小时使用全局调整

## 语速计算方式

### 方法 1：基于文本长度和音频时长

```rust
speech_rate = text_length / audio_duration_seconds
```

- **优点**：简单直接
- **缺点**：不同语言字符长度不同（中文 vs 英文）

### 方法 2：基于音节/词数（推荐）

```rust
// 中文：按字符数
// 英文：按词数或音节数
speech_rate = word_count / audio_duration_seconds
```

- **优点**：更准确，跨语言可比
- **缺点**：需要分词/分音节

### 方法 3：基于音频特征（高级）

```rust
// 使用音频能量、过零率等特征
speech_rate = estimate_from_audio_features(audio)
```

- **优点**：不依赖ASR结果
- **缺点**：实现复杂，可能不够准确

## 推荐实现

**第一步：全局调整**
- 使用所有用户的平均语速
- 根据平均语速调整全局阈值
- 快速验证效果

**第二步：按用户调整**
- 为每个说话者维护独立状态
- 根据每个用户的语速调整阈值
- 更精确的个性化

**配置建议**：
```toml
[vad]
type = "silero"
adaptive_mode = "per_speaker"  # 先实现 per_speaker

[vad.adaptive]
enabled = true
min_samples = 3  # 至少3个样本才开始调整
adjustment_rate = 0.1  # 每次调整的幅度（10%）
min_duration_ms = 300  # 最小阈值（300ms）
max_duration_ms = 1200  # 最大阈值（1200ms）
```

## 问题确认

请确认你希望使用哪种方案：

1. **按用户单独调整**：每个用户有自己的阈值
2. **全局统一调整**：所有用户使用相同的阈值
3. **混合模式**：根据情况自动选择

我建议使用**方案 1（按用户单独调整）**，因为：
- 你提到"不同用户的实际语速"
- 多人场景中语速差异通常较大
- 人员固定，适合维护每个用户的状态

