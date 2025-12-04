# VAD 边界识别调整机制总结

## 概述

目前 VAD（Voice Activity Detection）系统共有 **2 种主要的边界识别调整机制**，它们协同工作以优化边界检测的准确性。

## 机制 1：基于语速的自适应调整（Speech Rate Adaptive）

### 工作原理

根据用户的**实际语速历史**动态调整静音时长阈值。

- **快语速** → 更短的阈值（说话者句子之间停顿短）
- **慢语速** → 更长的阈值（说话者可能在句子中间思考停顿）

### 实现细节

**触发时机**：每次 ASR 识别完成后，根据识别文本和音频时长计算语速

**调整算法**：
1. 计算语速：`speech_rate = 字符数 / 音频时长（秒）`
2. 过滤异常语速：只接受 0.5-50.0 字符/秒的合理范围
3. 更新语速历史：保留最近 20 个样本
4. 计算平均语速：使用 EWMA（指数加权移动平均）
5. 使用 sigmoid 函数计算阈值倍数：
   ```rust
   multiplier = 1.0 + (1.0 / (1.0 + exp(-k * (avg_rate - center))))
   ```
   - 快语速（>10 chars/s）→ multiplier < 1.0 → 阈值降低
   - 慢语速（<5 chars/s）→ multiplier > 1.0 → 阈值提高
6. 应用调整：`adjusted_threshold = base_threshold * multiplier`
7. 限制范围：确保在 `adaptive_min_duration_ms` (400ms) 和 `adaptive_max_duration_ms` (800ms) 之间

### 关键方法

- `SileroVad::update_speech_rate(text, audio_duration_ms)` - 更新语速并调整阈值
- `SpeakerAdaptiveState::update_speech_rate(speech_rate, config)` - 内部状态更新
- `SpeakerAdaptiveState::get_adjusted_duration(config)` - 获取调整后的阈值

### 配置参数

```rust
adaptive_enabled: true              // 是否启用自适应
adaptive_min_samples: 1             // 至少需要1个样本才开始调整
adaptive_rate: 0.4                  // 每次调整40%（调整速度）
adaptive_min_duration_ms: 400       // 最小阈值（毫秒）
adaptive_max_duration_ms: 800       // 最大阈值（毫秒）
```

### 优势

- ✅ **自动适应**：无需手动配置，根据用户实际语速自动调整
- ✅ **平滑过渡**：使用 sigmoid 函数，避免阈值突变
- ✅ **全局优化**：使用全局语速历史，适用于单个用户的语速变化

### 限制

- ⚠️ 需要至少 1 个样本才开始调整
- ⚠️ 异常语速会被过滤（防止误识别影响调整）

---

## 机制 2：基于反馈的调整（Feedback-Based Adjustment）

### 工作原理

根据 **ASR 和 NMT 的质量反馈**直接调整阈值，用于纠正语速自适应可能无法覆盖的问题。

### 反馈类型

#### 2.1 BoundaryTooShort（边界过短）

**触发条件**：
1. ASR 结果被过滤（无意义文本，如 "(笑)"、"謝謝大家收看" 等）
2. ASR 结果太短（<3个字符）
3. ASR 结果很长（>50个字符，多个短句被合并）← **注意：这个实际上应该触发 BoundaryTooLong**
4. 翻译长度比例异常（>3倍或<0.3倍）
5. NMT 困惑度过高（>100）
6. NMT 平均概率过低（<0.05）
7. NMT 最小概率过低（<0.001）

**调整方向**：**提高阈值**（增加等待时间，让边界更长）

**调整幅度**：5%-30%（根据具体场景）

#### 2.2 BoundaryTooLong（边界过长）

**触发条件**：
1. ASR 结果很长（>50个字符，多个短句被合并）

**调整方向**：**降低阈值**（减少等待时间，让边界更短）

**调整幅度**：5%-30%（根据具体场景）

### 实现细节

**触发时机**：每次 NMT 翻译完成后，评估 ASR 和 NMT 的质量

**调整算法**：
```rust
// BoundaryTooShort: 提高阈值
adjustment = old_threshold * adjustment_factor  // 例如：500ms * 0.1 = 50ms
new_threshold = old_threshold + adjustment      // 500ms + 50ms = 550ms

// BoundaryTooLong: 降低阈值
adjustment = old_threshold * adjustment_factor  // 例如：500ms * 0.15 = 75ms
new_threshold = old_threshold - adjustment      // 500ms - 75ms = 425ms

// 限制范围
new_threshold = clamp(new_threshold, min=400ms, max=800ms)
```

### 关键方法

- `CoreEngine::adjust_vad_threshold_by_feedback()` - 评估反馈并决定调整
- `CoreEngine::apply_vad_feedback()` - 应用反馈调整
- `SileroVad::adjust_threshold_by_feedback(feedback_type, adjustment_factor)` - VAD 层面的调整

### 判断逻辑

```rust
// 判断1：ASR结果被过滤
if is_meaningless_transcript(text) {
    apply_feedback(BoundaryTooShort, 0.1);
    return;
}

// 判断2：ASR结果太短
if text_len < 3 {
    apply_feedback(BoundaryTooShort, 0.1);
    return;
}

// 判断3：ASR结果太长
if text_len > 50 {
    apply_feedback(BoundaryTooLong, 0.15);
    return;
}

// 判断4：翻译长度比例异常
if length_ratio > 3.0 || length_ratio < 0.3 {
    apply_feedback(BoundaryTooShort, 0.1);
    return;
}

// 判断5：NMT质量指标
if perplexity > 100.0 {
    apply_feedback(BoundaryTooShort, 0.1);
    return;
}
if avg_probability < 0.05 {
    apply_feedback(BoundaryTooShort, 0.1);
    return;
}
if min_probability < 0.001 {
    apply_feedback(BoundaryTooShort, 0.08);
    return;
}
```

### 优势

- ✅ **质量驱动**：基于实际识别和翻译质量，而非假设
- ✅ **快速响应**：立即调整，无需等待语速历史积累
- ✅ **多维度评估**：结合 ASR 和 NMT 的多个指标

### 限制

- ⚠️ 需要 ASR 和 NMT 完成才能评估
- ⚠️ 调整幅度有限（5%-30%），避免过度调整

---

## 两种机制的协同工作

### 工作流程

```
1. 用户说话
   ↓
2. VAD 使用当前阈值检测边界
   ↓
3. ASR 识别
   ↓
4. 机制1：基于语速调整（如果语速合理）
   └─> 更新语速历史，调整阈值
   ↓
5. NMT 翻译
   ↓
6. 机制2：基于反馈调整（如果质量异常）
   └─> 评估质量指标，直接调整阈值
   ↓
7. 下次检测使用新阈值
```

### 优先级

1. **机制1（语速自适应）**：持续运行，平滑调整
2. **机制2（反馈调整）**：仅在检测到问题时触发，快速纠正

### 互补关系

- **机制1**：处理**正常情况**下的语速变化
- **机制2**：处理**异常情况**下的边界问题

### 示例场景

**场景1：用户语速变快**
- 机制1：检测到语速提高 → 降低阈值（例如：500ms → 450ms）
- 机制2：如果识别质量正常，不触发

**场景2：边界过短导致误识别**
- 机制1：可能无法及时检测（需要语速历史）
- 机制2：检测到无意义文本 → 提高阈值（例如：500ms → 550ms）

**场景3：边界过长导致多个短句合并**
- 机制1：可能无法检测（语速可能正常）
- 机制2：检测到文本过长 → 降低阈值（例如：500ms → 425ms）

---

## 配置总结

### 基础配置

```rust
min_silence_duration_ms: 400  // 基础阈值（毫秒）
adaptive_min_duration_ms: 400 // 最小阈值（毫秒）
adaptive_max_duration_ms: 800 // 最大阈值（毫秒）
```

### 机制1配置

```rust
adaptive_enabled: true         // 启用自适应
adaptive_min_samples: 1        // 最小样本数
adaptive_rate: 0.4             // 调整速率（40%）
```

### 机制2配置

```rust
adjustment_factor: 0.05-0.3    // 调整幅度（5%-30%）
```

---

## 总结

### 机制数量

**2 种主要机制**：
1. **基于语速的自适应调整**（Speech Rate Adaptive）
2. **基于反馈的调整**（Feedback-Based Adjustment）

### 调整范围

- **最小阈值**：400ms
- **最大阈值**：800ms
- **基础阈值**：400ms
- **动态范围**：400ms ~ 800ms（±100%）

### 调整速度

- **机制1**：渐进式调整（每次 40% 向目标值靠近）
- **机制2**：立即调整（每次 5%-30%）

### 适用场景

- **机制1**：正常语速变化、长期适应
- **机制2**：异常识别、快速纠正

这两种机制相互补充，共同确保 VAD 边界检测的准确性和适应性。

