# VAD 反馈机制如何获取 NMT 检测结果

## 问题

反馈机制如何获得NMT的检测结果？

## 答案

反馈机制通过**函数参数直接传递**的方式获取NMT的检测结果，不需要额外的通信机制。

## 详细流程

### 1. NMT翻译执行并返回结果

```rust
// 在 process_audio_frame() 中
let mut translation_result = self.translate_and_publish(&personalized_with_speaker, vad_result.frame.timestamp_ms).await.ok();
```

**`translate_and_publish()` 方法**：
- 调用 `self.nmt.translate(translation_request).await?`
- NMT服务返回 `TranslationResponse`，其中包含：
  - `translated_text`: 翻译后的文本
  - `quality_metrics`: 质量指标（困惑度、概率分数等）
  - 其他字段...

**`TranslationResponse` 结构**：
```rust
pub struct TranslationResponse {
    pub translated_text: String,
    pub is_stable: bool,
    pub speaker_id: Option<String>,
    pub source_audio_duration_ms: Option<u64>,
    pub source_text: Option<String>,
    /// 翻译质量指标（可选，用于 VAD 边界调整）
    pub quality_metrics: Option<QualityMetrics>,  // ← 关键字段
}
```

**`QualityMetrics` 结构**：
```rust
pub struct QualityMetrics {
    /// 困惑度（越低越好，正常范围：10-100）
    pub perplexity: Option<f32>,
    /// 平均概率（越高越好，正常范围：0.05-0.5）
    pub avg_probability: Option<f32>,
    /// 最小概率（越高越好，正常范围：0.001-0.1）
    pub min_probability: Option<f32>,
}
```

### 2. 反馈机制接收NMT结果

```rust
// 在 process_audio_frame() 中，NMT翻译完成后
self.adjust_vad_threshold_by_feedback(
    &asr_result,                                    // ASR识别结果
    translation_stable.as_ref(),                    // 翻译文本（StableTranscript格式）
    translation_result.as_ref(),                    // ← 完整的 TranslationResponse（包含质量指标）
    vad_result.frame.timestamp_ms,                  // 边界时间戳
    vad_result.frame.timestamp_ms,                  // ASR开始时间戳
);
```

**关键点**：
- `translation_result` 是 `Option<TranslationResponse>` 类型
- 通过 `.as_ref()` 转换为 `Option<&TranslationResponse>`
- 作为函数参数直接传递给反馈机制

### 3. 反馈机制使用质量指标

```rust
// 在 adjust_vad_threshold_by_feedback() 方法中
fn adjust_vad_threshold_by_feedback(
    &self,
    asr_result: &AsrResult,
    translation_stable: Option<&StableTranscript>,
    translation_response: Option<&TranslationResponse>,  // ← 接收NMT结果
    _boundary_timestamp_ms: u64,
    _asr_start_timestamp_ms: u64,
) {
    // ... 其他判断逻辑 ...
    
    // 判断5：基于NMT质量指标（困惑度、概率分数）
    if let Some(ref translation_resp) = translation_response {  // ← 检查是否有NMT结果
        if let Some(ref metrics) = translation_resp.quality_metrics {  // ← 检查是否有质量指标
            // 5.1. 检查困惑度
            if let Some(perplexity) = metrics.perplexity {
                if perplexity > 100.0 {
                    eprintln!("[VAD Feedback] ⚠️  High perplexity ({:.2}), suggesting ASR may be inaccurate (boundary too short?)", perplexity);
                    self.apply_vad_feedback(VadFeedbackType::BoundaryTooShort, 0.1);
                    return;
                }
            }
            
            // 5.2. 检查平均概率
            if let Some(avg_prob) = metrics.avg_probability {
                if avg_prob < 0.05 {
                    eprintln!("[VAD Feedback] ⚠️  Low average probability ({:.4}), suggesting ASR may be inaccurate (boundary too short?)", avg_prob);
                    self.apply_vad_feedback(VadFeedbackType::BoundaryTooShort, 0.1);
                    return;
                }
            }
            
            // 5.3. 检查最小概率
            if let Some(min_prob) = metrics.min_probability {
                if min_prob < 0.001 {
                    eprintln!("[VAD Feedback] ⚠️  Very low min probability ({:.6}), suggesting some tokens are highly uncertain (boundary too short?)", min_prob);
                    self.apply_vad_feedback(VadFeedbackType::BoundaryTooShort, 0.08);
                    return;
                }
            }
        }
    }
}
```

## 完整数据流

```
┌─────────────────────────────────────────────────────────────┐
│                    CoreEngine                                │
│                                                              │
│  1. ASR识别完成                                              │
│     └─> asr_result: AsrResult                               │
│                                                              │
│  2. 调用 translate_and_publish()                            │
│     └─> self.nmt.translate(translation_request)             │
│         │                                                    │
│         ├─> NMT服务（Python）                                │
│         │   └─> 计算质量指标                                 │
│         │       ├─> perplexity                              │
│         │       ├─> avg_probability                         │
│         │       └─> min_probability                         │
│         │                                                    │
│         └─> 返回 TranslationResponse                        │
│             └─> quality_metrics: Option<QualityMetrics>     │
│                                                              │
│  3. 保存 translation_result                                  │
│     └─> let mut translation_result = ...                    │
│                                                              │
│  4. 调用反馈机制                                              │
│     └─> self.adjust_vad_threshold_by_feedback(              │
│             &asr_result,                                     │
│             translation_stable.as_ref(),                     │
│             translation_result.as_ref(),  ← 传递NMT结果      │
│             ...                                              │
│         )                                                    │
│                                                              │
│  5. 反馈机制使用质量指标                                      │
│     └─> if let Some(ref metrics) =                          │
│            translation_resp.quality_metrics {                │
│         ├─> 检查 perplexity                                 │
│         ├─> 检查 avg_probability                            │
│         └─> 检查 min_probability                            │
│     }                                                        │
│                                                              │
│  6. 调整VAD阈值                                              │
│     └─> self.apply_vad_feedback(...)                        │
│         └─> VAD.adjust_threshold_by_feedback(...)           │
└─────────────────────────────────────────────────────────────┘
```

## 关键设计点

### 1. 同步传递（无延迟）

- NMT翻译完成后，结果立即通过函数参数传递给反馈机制
- 不需要事件总线、消息队列或进程间通信
- 零延迟，立即生效

### 2. 可选字段（容错性）

```rust
pub quality_metrics: Option<QualityMetrics>,  // 可选字段
```

- 如果NMT服务不支持质量指标，`quality_metrics` 为 `None`
- 反馈机制会检查 `if let Some(ref metrics) = ...`，安全处理缺失情况
- 不会因为质量指标缺失而导致系统崩溃

### 3. 嵌套可选（多层检查）

```rust
if let Some(ref translation_resp) = translation_response {  // 第1层：检查是否有NMT结果
    if let Some(ref metrics) = translation_resp.quality_metrics {  // 第2层：检查是否有质量指标
        if let Some(perplexity) = metrics.perplexity {  // 第3层：检查是否有困惑度值
            // 使用困惑度
        }
    }
}
```

- 三层可选检查，确保安全访问
- 任何一层缺失都不会导致panic

### 4. 直接访问（无序列化开销）

- `TranslationResponse` 在内存中直接传递
- 不需要序列化/反序列化
- 不需要网络传输
- 性能开销最小

## NMT服务如何生成质量指标

### Python服务端（M2M100）

```python
# 在 services/nmt_m2m100/nmt_service.py 中
def translate(text: str, src_lang: str, tgt_lang: str):
    # 1. 调用模型生成翻译
    outputs = model.generate(
        inputs,
        output_scores=True,  # ← 启用分数输出
        return_dict_in_generate=True,  # ← 返回完整字典
    )
    
    # 2. 从输出中提取token分数
    scores = outputs.scores  # List[Tensor]
    
    # 3. 计算质量指标
    perplexity = calculate_perplexity(scores)
    avg_probability = calculate_avg_probability(scores)
    min_probability = calculate_min_probability(scores)
    
    # 4. 构造响应
    return TranslateResponse(
        text=translated_text,
        quality_metrics=QualityMetrics(
            perplexity=perplexity,
            avg_probability=avg_probability,
            min_probability=min_probability,
        ),
    )
```

### Rust客户端接收

```rust
// 在 core/engine/src/nmt_client/local_m2m100.rs 中
let response: NmtTranslateResponse = serde_json::from_str(&response_text)?;

// quality_metrics 自动反序列化
// response.quality_metrics: Option<QualityMetrics>
```

### 适配器传递

```rust
// 在 core/engine/src/nmt_client/adapter.rs 中
pub async fn translate(&self, request: TranslationRequest) -> EngineResult<TranslationResponse> {
    let response = self.client.translate(&nmt_request).await?;
    
    Ok(TranslationResponse {
        translated_text: response.text.unwrap_or_default(),
        is_stable: true,
        speaker_id: request.speaker_id.clone(),
        quality_metrics: response.quality_metrics.clone(),  // ← 传递质量指标
        // ...
    })
}
```

## 总结

### 反馈机制获取NMT结果的方式

1. **直接函数参数传递**：NMT结果通过函数参数直接传递给反馈机制
2. **同步执行**：在同一个调用栈中，无延迟
3. **可选字段**：质量指标是可选字段，缺失时不影响系统运行
4. **多层检查**：使用 `if let Some(...)` 安全访问嵌套的可选字段
5. **零开销**：内存中直接传递，无需序列化或网络传输

### 数据流向

```
NMT服务（Python）
  ↓ (HTTP响应，JSON序列化)
Rust NMT客户端
  ↓ (反序列化为 NmtTranslateResponse)
NMT适配器
  ↓ (转换为 TranslationResponse)
CoreEngine::translate_and_publish()
  ↓ (返回 TranslationResponse)
CoreEngine::process_audio_frame()
  ↓ (保存为 translation_result)
CoreEngine::adjust_vad_threshold_by_feedback()
  ↓ (通过函数参数接收)
反馈机制使用质量指标
  ↓ (评估并调整)
VAD阈值更新
```

### 关键代码位置

- **NMT结果生成**：`services/nmt_m2m100/nmt_service.py`
- **NMT结果接收**：`core/engine/src/nmt_client/local_m2m100.rs`
- **NMT结果传递**：`core/engine/src/nmt_client/adapter.rs`
- **反馈机制调用**：`core/engine/src/bootstrap/engine.rs:523`
- **反馈机制实现**：`core/engine/src/bootstrap/engine.rs:2270-2386`

