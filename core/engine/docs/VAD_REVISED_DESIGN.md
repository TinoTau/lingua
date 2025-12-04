# VAD 边界检测机制修订版（开发用）

本文档提供对当前 VAD 边界检测逻辑的统一、矛盾修复、去重、优化设计方案。  
目标：让边界检测稳定、不反复矛盾、不重复调参，并避免误切短句。

---

# 1. 主要问题总结（来自上一版设计）

## 1.1 明显矛盾
- 条件 **“ASR 字符数 > 50（文本过长）”** 同时出现在：
  - BoundaryTooShort（边界过短）
  - BoundaryTooLong（边界过长）

这两者调参方向相反，导致逻辑冲突。

## 1.2 重复触发 / 多次调参
同一段音频可能同时触发：
- 文本太短
- 翻译比例异常
- 困惑度异常
- 低概率词
→ **导致一次样本引发多个 BoundaryTooShort 调参**。

## 1.3 两个机制都直接操作同一个 absolute 阈值（400–800ms）
- 语速自适应在改阈值
- 质量反馈也在改阈值  
→ 两个机制容易“互相抢阈值”，造成抖动。

## 1.4 静音阈值上限偏小
400–800ms 的阈值范围过窄，尤其中文口语停顿通常 700–1200ms。

---

# 2. 修订后方案总览

核心理念：

- **机制 1 负责生成 base 阈值（基于语速）**
- **机制 2 负责生成 delta 偏移量（基于质量反馈）**
- Final effective threshold = base_threshold + delta_clamped

并且：
- 去除矛盾条件
- 把多个质量异常合并为单一标签“BadBoundary”
- 把“文本过长”统一归类为 BoundaryTooLong
- 扩大静音阈值整体可调范围

---

# 3. 修订版阈值计算流程（最终结构）

```
step 1: 语速 → base_threshold  (600–1100ms)
step 2: 质量反馈 → delta       (-300ms ~ +300ms)
step 3: effective_threshold = clamp(base_threshold + delta, 500–1500ms)
step 4: 用 effective_threshold 判断本次边界
```

---

# 4. 修订后的触发条件（统一、无矛盾）

## 4.1 BoundaryTooShort（边界过短 → 让边界更晚出现）
适用条件：
- 文本太短： ASR < 3–5 字
- 文本无意义：如括号笑声、非常短片段
- 翻译长度比例异常： ratio < 0.3 或 > 3.0
- 困惑度高 / 最小概率低
- **仅限“文本短 / 质量低”的场景**

**不再包含：文本 > 50 字（已移除）**

→ Action：delta += +150ms（提高阈值）

---

## 4.2 BoundaryTooLong（边界过长 → 让边界更早出现）
适用条件：
- **文本过长（> 50 字）**  
  → 多个句子被粘在一起，应提早切句  
- 说话速度很快（语速自适应已经调低 base，但依旧过长）

→ Action：delta += -150ms（降低阈值）

---

## 4.3 BadBoundary（综合质量异常）
如果一段音频同时符合多个条件，不做多次调参，而是记为单次 BadBoundary：

触发条件（满足任意一条）：
- 文本过短
- 翻译比例异常
- 困惑度异常
- 最小概率异常
- 无意义文本

**只执行一次 delta += +150ms**

---

# 5. 去重逻辑（防止连环触发）

伪代码：

```
feedbacks = collect_feedback_signals()

if "BoundaryTooLong" in feedbacks:
    delta -= 150
else:
    if any_short_conditions:
        delta += 150
```

确保：
- 文本过长优先 → TooLong
- 非太长场景 → TooShort
- 同时触发多个质量异常 → 只累计一次 delta

---

# 6. 新参数建议

| 参数 | 新值 | 说明 |
|------|------|------|
| base_threshold_range | 600–1100ms | 语速自适应输出的基础范围 |
| delta_range | -300〜+300ms | 质量反馈偏移量 |
| final_threshold_range | 500–1500ms | 实际使用的有效范围 |
| min_utterance_ms | 1500ms | 防止半句话被切掉，至关重要 |
| too_long_text_length | >50 字 | 统一归为 BoundaryTooLong |
| too_short_text_length | <3–5 字 | 归为 BoundaryTooShort |

---

# 7. 最终流程图（可贴到代码旁边）

```
------------- Audio Segment ----------------
                 │
                 ▼
        Whisper ASR (short result)
                 │
                 ▼
      [质量分析 + 长度判断] ─────────────┐
                 │                      │
                 │                      ▼
         classify feedback     BoundaryTooLong?
                 │                      │
                 ▼                      ▼
        if short → TooShort       if long → TooLong
                 │                      │
                 └──────┬──────────────┘
                        ▼
               apply delta (once)
                        ▼
         effective_threshold = clamp(base + delta)
                        ▼
         VAD 判断下一段边界使用此阈值
```

---

# 8. 给开发的交付总结（可直接贴 Jira）

1. 删除所有重复 / 矛盾条件（尤其“文本 >50 字”不得出现于 TooShort）。  
2. 新增“BadBoundary”合并判断，只触发一次调参。  
3. 语速机制生成 base_threshold，质量反馈只生成 delta。  
4. 静音阈值最终允许在 500–1500ms 动态变化。  
5. 增加 min_utterance_ms=1500ms 防止半句截断。  
6. BoundaryTooLong → δ = -150ms；BoundaryTooShort → δ = +150ms。  
7. 执行 order：TooLong 优先，其次 TooShort，BadBoundary 只执行一次。  

