# NMT 质量指标实现方案

本文档说明如何实现以下三个功能来评估 NMT 翻译质量，用于 VAD 边界调整：

1. **困惑度（Perplexity）分数**
2. **语言模型的概率分数**
3. **语法正确性检查**

## 1. 困惑度（Perplexity）分数

### 可行性：✅ **可以实现**

### 实现方法

M2M100 模型在生成时可以返回每个 token 的 logits（未归一化的概率），我们可以计算困惑度：

```python
# 在 nmt_service.py 中修改 generate 调用
with torch.no_grad():
    outputs = model.generate(
        **encoded,
        forced_bos_token_id=forced_bos,
        num_beams=4,
        no_repeat_ngram_size=3,
        repetition_penalty=1.2,
        max_new_tokens=256,
        early_stopping=False,
        output_scores=True,  # 返回每个 token 的分数
        return_dict_in_generate=True,  # 返回详细信息
    )

# 计算困惑度
scores = outputs.scores  # List[Tensor]，每个元素是 [batch_size, vocab_size]
generated_ids = outputs.sequences[0]  # 生成的 token IDs

# 提取生成部分的 token IDs（跳过输入部分）
input_length = encoded['input_ids'].shape[1]
generated_token_ids = generated_ids[input_length:]

# 计算困惑度
log_probs = []
for i, token_id in enumerate(generated_token_ids):
    if i < len(scores):
        # 获取该 token 的 logits
        logits = scores[i][0]  # [vocab_size]
        # 计算该 token 的 log 概率
        log_probs.append(torch.log_softmax(logits, dim=0)[token_id].item())

# 困惑度 = exp(-平均 log 概率)
if len(log_probs) > 0:
    avg_log_prob = sum(log_probs) / len(log_probs)
    perplexity = math.exp(-avg_log_prob)
else:
    perplexity = float('inf')
```

### 优点
- 直接使用模型内部信息，无需额外模型
- 困惑度越低，翻译质量通常越高
- 计算开销较小（只需要额外的 logits 计算）

### 缺点
- 需要修改 `model.generate()` 调用，可能影响性能
- 困惑度只能反映模型对生成序列的置信度，不能直接反映翻译准确性

### 阈值建议
- 正常翻译：困惑度 < 50
- 可疑翻译：困惑度 50-100
- 低质量翻译：困惑度 > 100

---

## 2. 语言模型的概率分数

### 可行性：✅ **可以实现**

### 实现方法

与困惑度类似，我们可以获取每个 token 的概率分数：

```python
# 在 generate 时获取分数
outputs = model.generate(
    ...,
    output_scores=True,
    return_dict_in_generate=True,
)

# 计算平均概率分数
scores = outputs.scores
generated_token_ids = outputs.sequences[0][input_length:]

token_probs = []
for i, token_id in enumerate(generated_token_ids):
    if i < len(scores):
        logits = scores[i][0]
        probs = torch.softmax(logits, dim=0)
        token_probs.append(probs[token_id].item())

avg_prob = sum(token_probs) / len(token_probs) if token_probs else 0.0
min_prob = min(token_probs) if token_probs else 0.0
```

### 优点
- 可以识别低概率的 token（可能是翻译错误）
- 可以计算最小概率（最不确定的部分）
- 计算开销小

### 缺点
- 概率分数受模型训练数据影响，可能不准确
- 需要设置合理的阈值

### 阈值建议
- 高质量翻译：平均概率 > 0.1，最小概率 > 0.01
- 中等质量：平均概率 0.05-0.1
- 低质量：平均概率 < 0.05 或最小概率 < 0.001

---

## 3. 语法正确性检查

### 可行性：⚠️ **需要额外工具**

### 实现方法

有几种方案：

#### 方案 A：使用语言模型进行语法检查（推荐）

使用一个预训练的语言模型（如 GPT-2、BERT）来评估翻译结果的语法正确性：

```python
from transformers import AutoModelForCausalLM, AutoTokenizer

# 加载语言模型（仅用于语法检查）
grammar_model = AutoModelForCausalLM.from_pretrained("gpt2")
grammar_tokenizer = AutoTokenizer.from_pretrained("gpt2")

def check_grammar(text: str, target_lang: str) -> float:
    """检查语法正确性，返回 0-1 的分数"""
    # 编码文本
    inputs = grammar_tokenizer(text, return_tensors="pt")
    
    # 计算困惑度（语法正确的文本应该有较低的困惑度）
    with torch.no_grad():
        outputs = grammar_model(**inputs, labels=inputs["input_ids"])
        loss = outputs.loss
        perplexity = math.exp(loss.item())
    
    # 将困惑度转换为 0-1 分数（困惑度越低，分数越高）
    # 正常文本的困惑度通常在 10-100 之间
    score = 1.0 / (1.0 + perplexity / 50.0)
    return score
```

#### 方案 B：使用专门的语法检查库

对于英文，可以使用 `language_tool_python` 或 `grammarly` API：

```python
import language_tool_python

tool = language_tool_python.LanguageTool('en-US')

def check_grammar(text: str) -> float:
    """检查语法错误，返回 0-1 的分数"""
    matches = tool.check(text)
    error_count = len(matches)
    word_count = len(text.split())
    
    # 错误率越低，分数越高
    error_rate = error_count / max(word_count, 1)
    score = max(0.0, 1.0 - error_rate * 10)  # 每个错误降低 10% 分数
    return score
```

#### 方案 C：使用 NMT 模型的双向翻译

将翻译结果反向翻译回源语言，比较与原文的相似度：

```python
def check_grammar_by_backtranslation(original: str, translation: str, 
                                     src_lang: str, tgt_lang: str) -> float:
    """通过反向翻译检查语法"""
    # 将翻译结果反向翻译回源语言
    backtranslated = model.translate(translation, tgt_lang, src_lang)
    
    # 计算相似度（可以使用 BLEU、ROUGE 或简单的字符相似度）
    similarity = calculate_similarity(original, backtranslated)
    return similarity
```

### 优点
- 方案 A：使用现有模型，无需额外服务
- 方案 B：专门针对语法检查，准确性高
- 方案 C：利用翻译模型本身，无需额外工具

### 缺点
- 方案 A：需要加载额外的语言模型，增加内存开销
- 方案 B：需要安装额外库，可能不支持所有语言
- 方案 C：需要额外的反向翻译，增加延迟

### 推荐方案

**推荐使用方案 A（语言模型困惑度）**，因为：
1. 可以使用轻量级模型（如 DistilGPT-2）
2. 支持多种语言
3. 计算开销相对较小
4. 不需要额外的 API 或服务

---

## 实现步骤

### 步骤 1：修改 NMT 服务（Python）

在 `services/nmt_m2m100/nmt_service.py` 中：

1. 修改 `model.generate()` 调用，添加 `output_scores=True` 和 `return_dict_in_generate=True`
2. 计算困惑度和平均概率分数
3. （可选）添加语法检查功能

### 步骤 2：修改响应结构

在 `TranslateResponse` 中添加质量指标：

```python
class TranslateResponse(BaseModel):
    ok: bool
    text: Optional[str] = None
    model: Optional[str] = None
    provider: str = "local-m2m100"
    extra: Optional[Dict[str, Any]] = None
    error: Optional[str] = None
    # 新增质量指标
    quality_metrics: Optional[Dict[str, float]] = None  # {"perplexity": 25.3, "avg_prob": 0.15, "grammar_score": 0.85}
```

### 步骤 3：修改 Rust 客户端

在 `core/engine/src/nmt_client/types.rs` 中：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NmtTranslateResponse {
    pub ok: bool,
    pub text: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub extra: Option<serde_json::Value>,
    pub error: Option<String>,
    // 新增质量指标
    pub quality_metrics: Option<QualityMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub perplexity: Option<f32>,
    pub avg_probability: Option<f32>,
    pub min_probability: Option<f32>,
    pub grammar_score: Option<f32>,
}
```

### 步骤 4：在 VAD 反馈中使用质量指标

在 `core/engine/src/bootstrap/engine.rs` 的 `adjust_vad_threshold_by_feedback()` 中：

```rust
// 判断5：基于 NMT 质量指标
if let Some(ref translation) = translation_result {
    if let Some(ref metrics) = translation.quality_metrics {
        // 检查困惑度
        if let Some(perplexity) = metrics.perplexity {
            if perplexity > 100.0 {
                eprintln!("[VAD Feedback] ⚠️  High perplexity ({:.2}), suggesting ASR may be inaccurate", perplexity);
                self.apply_vad_feedback(VadFeedbackType::BoundaryTooShort, 0.1);
                return;
            }
        }
        
        // 检查平均概率
        if let Some(avg_prob) = metrics.avg_probability {
            if avg_prob < 0.05 {
                eprintln!("[VAD Feedback] ⚠️  Low average probability ({:.3}), suggesting ASR may be inaccurate", avg_prob);
                self.apply_vad_feedback(VadFeedbackType::BoundaryTooShort, 0.1);
                return;
            }
        }
        
        // 检查语法分数
        if let Some(grammar_score) = metrics.grammar_score {
            if grammar_score < 0.5 {
                eprintln!("[VAD Feedback] ⚠️  Low grammar score ({:.2}), suggesting translation may be incorrect", grammar_score);
                self.apply_vad_feedback(VadFeedbackType::BoundaryTooShort, 0.1);
                return;
            }
        }
    }
}
```

---

## 性能影响评估

### 困惑度和概率分数
- **额外计算时间**：+10-20ms（主要是 logits 计算）
- **内存开销**：几乎无（只是返回额外的分数）
- **推荐**：✅ 实现，开销小，收益大

### 语法检查（方案 A）
- **额外计算时间**：+50-100ms（需要额外的前向传播）
- **内存开销**：+500MB-1GB（需要加载额外的语言模型）
- **推荐**：⚠️ 可选，如果性能允许可以添加

---

## 总结

| 功能 | 可行性 | 实现难度 | 性能影响 | 推荐度 |
|------|--------|----------|----------|--------|
| 困惑度分数 | ✅ 高 | 低 | 小（+10-20ms） | ⭐⭐⭐⭐⭐ |
| 概率分数 | ✅ 高 | 低 | 小（+10-20ms） | ⭐⭐⭐⭐⭐ |
| 语法检查 | ⚠️ 中 | 中 | 中（+50-100ms） | ⭐⭐⭐ |

**建议优先实现困惑度和概率分数**，这两个功能实现简单、性能影响小，且能有效识别低质量的翻译结果。

