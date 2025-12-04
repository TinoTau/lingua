# VAD 反馈调整机制工作流程

本文档详细说明基于 ASR/NMT 反馈的 VAD 阈值自适应调整机制的工作流程。

## 整体架构

```
音频输入 → VAD边界检测 → ASR识别 → NMT翻译 → 质量评估 → VAD阈值调整
   ↑                                                              ↓
   └──────────────────────────────────────────────────────────────┘
                        反馈循环
```

## 详细工作流程

### 阶段 1: 音频处理和边界检测

```
1. 音频帧到达
   └─> CoreEngine::process_audio_frame()
       └─> VAD::detect() 
           └─> 检测到边界（is_boundary = true）
               └─> 触发 ASR 处理
```

**关键点**：
- VAD 使用当前阈值（初始值：400ms，动态调整范围：400ms~800ms）
- 当静音时长 ≥ 当前阈值时，触发边界检测

### 阶段 2: ASR 识别

```
2. ASR 处理
   └─> WhisperAsrStreaming::infer_on_boundary()
       └─> 识别音频内容
           └─> 返回 AsrResult {
                 final_transcript: "识别的文本",
                 ...
               }
```

**关键点**：
- ASR 识别结果包含文本、置信度、语言等信息
- 如果识别结果被过滤（无意义文本），会提前返回，不进入后续流程

### 阶段 3: NMT 翻译和质量指标计算

```
3. NMT 翻译
   └─> CoreEngine::translate_and_publish()
       └─> NmtClientAdapter::translate()
           └─> LocalM2m100HttpClient::translate()
               └─> HTTP POST /v1/translate
                   └─> Python NMT Service
                       ├─> model.generate(output_scores=True)
                       ├─> 计算困惑度 (perplexity)
                       ├─> 计算平均概率 (avg_probability)
                       └─> 计算最小概率 (min_probability)
                           └─> 返回 TranslationResponse {
                                 translated_text: "翻译结果",
                                 quality_metrics: {
                                   perplexity: 25.3,
                                   avg_probability: 0.15,
                                   min_probability: 0.08
                                 }
                               }
```

**关键点**：
- Python 服务在生成翻译时计算质量指标
- 质量指标通过 HTTP 响应传递回 Rust 客户端
- `NmtClientAdapter` 将质量指标传递到 `TranslationResponse`

### 阶段 4: 反馈评估和阈值调整

```
4. 反馈评估
   └─> CoreEngine::adjust_vad_threshold_by_feedback()
       ├─> 判断 1: ASR 结果被过滤？
       │   └─> 是 → 边界过短 → 提高阈值
       ├─> 判断 2: ASR 结果太短（<3字符）？
       │   └─> 是 → 边界过短 → 提高阈值
       ├─> 判断 3: ASR 结果太长（>50字符）？
       │   └─> 是 → 边界过长 → 降低阈值
       ├─> 判断 4: 翻译长度比例异常？
       │   └─> 是 → 边界过短 → 提高阈值
       └─> 判断 5: 质量指标异常？
           ├─> 困惑度 > 100 → 边界过短 → 提高阈值
           ├─> 平均概率 < 0.05 → 边界过短 → 提高阈值
           └─> 最小概率 < 0.001 → 边界过短 → 提高阈值
```

**关键点**：
- 多个判断条件按优先级执行，一旦触发就立即调整并返回
- 调整幅度：10-15%（通过 `adjustment_factor` 控制）

### 阶段 5: VAD 阈值更新

```
5. VAD 阈值更新
   └─> CoreEngine::apply_vad_feedback()
       └─> SileroVad::adjust_threshold_by_feedback()
           └─> 更新 adjusted_duration_ms
               ├─> BoundaryTooLong → 降低阈值（减少等待时间）
               └─> BoundaryTooShort → 提高阈值（增加等待时间）
                   └─> 限制在 [400ms, 800ms] 范围内
```

**关键点**：
- 阈值调整是立即生效的
- 下次边界检测时就会使用新的阈值
- 调整后的阈值会持续影响后续的边界检测

## 完整数据流示例

### 场景 1: 边界过长（多个短句被合并）

```
时间线：
T0: 用户说 "你好"
T1: 用户停顿 500ms
T2: 用户说 "今天天气不错"
T3: 用户停顿 500ms
T4: 用户说 "我们去公园吧"
T5: 用户停顿 600ms（超过阈值 400ms）
T6: VAD 检测到边界

处理流程：
1. VAD 检测到边界（阈值=400ms，实际静音=600ms）
2. ASR 识别：识别出 "你好 今天天气不错 我们去公园吧"（50+ 字符）
3. NMT 翻译：翻译结果正常
4. 反馈评估：
   - 判断 3: ASR 结果太长（>50字符）→ 触发
   - 结论：边界过长，多个短句被合并
5. VAD 调整：
   - 类型：BoundaryTooLong
   - 调整：降低阈值 15%（400ms → 340ms）
6. 下次边界检测：
   - 使用新阈值 340ms
   - 更早检测到边界，避免短句合并
```

### 场景 2: 边界过短（识别结果混乱）

```
时间线：
T0: 用户说 "你好"
T1: 用户停顿 200ms（很短）
T2: VAD 检测到边界（阈值=400ms，但可能因为其他原因提前触发）

处理流程：
1. VAD 检测到边界
2. ASR 识别：识别出 "你"（只有1个字符，太短）
3. NMT 翻译：翻译结果异常（困惑度=150，平均概率=0.02）
4. 反馈评估：
   - 判断 2: ASR 结果太短（<3字符）→ 触发
   - 判断 5.1: 困惑度 > 100 → 触发
   - 判断 5.2: 平均概率 < 0.05 → 触发
   - 结论：边界过短，导致识别不完整
5. VAD 调整：
   - 类型：BoundaryTooShort
   - 调整：提高阈值 10%（400ms → 440ms）
6. 下次边界检测：
   - 使用新阈值 440ms
   - 等待更长的静音，确保识别完整
```

### 场景 3: 正常情况（无需调整）

```
时间线：
T0: 用户说 "今天天气不错"
T1: 用户停顿 500ms
T2: VAD 检测到边界

处理流程：
1. VAD 检测到边界（阈值=400ms，实际静音=500ms）
2. ASR 识别：识别出 "今天天气不错"（6个字符，正常）
3. NMT 翻译：翻译结果正常（困惑度=25，平均概率=0.15）
4. 反馈评估：
   - 判断 1: ASR 结果未被过滤 ✓
   - 判断 2: ASR 结果长度正常（≥3字符）✓
   - 判断 3: ASR 结果长度正常（≤50字符）✓
   - 判断 4: 翻译长度比例正常（0.5-2.0）✓
   - 判断 5: 质量指标正常 ✓
   - 结论：无需调整
5. VAD 调整：
   - 不触发调整
   - 保持当前阈值
```

## 判断条件优先级

反馈评估按以下顺序执行，**一旦触发就立即调整并返回**：

1. **判断 1**: ASR 结果被过滤（最高优先级）
   - 原因：明显无意义，立即调整
   - 调整：BoundaryTooShort，10%

2. **判断 2**: ASR 结果太短（<3字符）
   - 原因：识别不完整
   - 调整：BoundaryTooShort，10%

3. **判断 3**: ASR 结果太长（>50字符）
   - 原因：多个短句被合并
   - 调整：BoundaryTooLong，15%

4. **判断 4**: 翻译长度比例异常
   - 原因：识别可能不准确
   - 调整：BoundaryTooShort，10%

5. **判断 5**: 质量指标异常（最低优先级）
   - 5.1: 困惑度 > 100 → BoundaryTooShort，10%
   - 5.2: 平均概率 < 0.05 → BoundaryTooShort，10%
   - 5.3: 最小概率 < 0.001 → BoundaryTooShort，8%

## 阈值调整机制

### 调整公式

```rust
// 在 SileroVad::adjust_threshold_by_feedback() 中
let adjustment = match feedback_type {
    BoundaryTooLong => -(old_threshold * factor),  // 降低阈值
    BoundaryTooShort => (old_threshold * factor),  // 提高阈值
};

let new_threshold = (old_threshold + adjustment)
    .clamp(min_duration_ms, max_duration_ms);  // 限制在 400ms~800ms
```

### 调整因子

- **默认调整因子**: 0.1（10%）
- **长文本合并**: 0.15（15%，更激进的调整）
- **最小概率异常**: 0.08（8%，轻微调整）

### 阈值限制

- **最小值**: 400ms（`adaptive_min_duration_ms`）
- **最大值**: 800ms（`adaptive_max_duration_ms`）
- **基础值**: 400ms（`min_silence_duration_ms`）

## 与语速自适应机制的协同

VAD 有两个自适应机制：

1. **语速自适应**（基于历史语速）
   - 触发时机：每次 ASR 识别完成后
   - 调整方式：根据平均语速计算 sigmoid 函数，平滑调整
   - 调整速度：每次调整 40%（`adaptive_rate = 0.4`）

2. **反馈自适应**（基于质量指标）
   - 触发时机：检测到质量问题时
   - 调整方式：立即调整固定百分比
   - 调整速度：每次调整 8-15%

**协同工作**：
- 语速自适应提供**基础调整**（适应说话者的正常语速）
- 反馈自适应提供**纠错调整**（纠正异常情况）
- 两者可以同时工作，但反馈调整会覆盖语速调整的结果

## 性能影响

### 额外开销

1. **NMT 质量指标计算**：
   - 时间：+10-20ms（主要是 logits 计算）
   - 内存：几乎无（只是返回额外的分数）

2. **反馈评估**：
   - 时间：< 1ms（简单的条件判断）
   - 内存：几乎无

3. **阈值调整**：
   - 时间：< 0.1ms（简单的数值计算）
   - 内存：几乎无

**总开销**：约 +10-20ms，对整体延迟影响很小。

## 日志输出

系统会输出详细的日志，方便调试：

```
[VAD Feedback] ⚠️  ASR result too long (65 chars), suggesting boundary may be too long (multiple sentences merged)
[SileroVad] 🔧 Threshold adjusted by feedback: 400ms -> 340ms (type=BoundaryTooLong, factor=15.0%, change=-15.0%)

[VAD Feedback] ⚠️  High perplexity (125.30), suggesting ASR may be inaccurate (boundary too short?)
[SileroVad] 🔧 Threshold adjusted by feedback: 400ms -> 440ms (type=BoundaryTooShort, factor=10.0%, change=+10.0%)
```

## 总结

反馈机制通过以下方式工作：

1. **实时监控**：每次翻译完成后，自动评估质量
2. **多维度判断**：从 ASR 结果、翻译长度、质量指标等多个角度判断
3. **立即调整**：发现问题后立即调整阈值，下次边界检测生效
4. **持续优化**：通过反馈循环，逐步优化边界检测的准确性

这样可以有效解决：
- ✅ 多个短句被合并的问题（通过检测长文本）
- ✅ 识别结果混乱的问题（通过质量指标）
- ✅ 边界检测不准确的问题（通过动态调整）

