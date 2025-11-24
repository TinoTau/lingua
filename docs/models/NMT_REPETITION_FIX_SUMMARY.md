# NMT 重复 Token 问题修复总结

**日期**: 2025-11-21  
**状态**: ✅ **已实施修复措施**

---

## 🔍 问题描述

### 现象

M2M100 NMT 翻译出现严重的重复 token 问题：
- **英文→中文翻译**: token 序列 `[22, 3]` 重复出现
- **中文→英文翻译**: 也出现重复 token 模式
- **解码结果**: 包含大量 `<unk>` 和乱码

### 根本原因

从日志分析，问题出现在解码过程中：
- Step 2: 输入 token `22`，模型选择 token `3`
- Step 3: 输入 token `3`，模型选择 token `22` ← **开始循环**
- Step 4: 输入 token `22`，模型选择 token `3` ← **继续循环**

这说明模型的 logits 分布导致某些 token 对互相选择，形成无限循环。

---

## ✅ 已实施的修复措施

### 1. 重复惩罚机制（Repetition Penalty）

**实现位置**: `core/engine/src/nmt_incremental/m2m100_translation.rs`

**机制**:
- 在贪婪解码中，对最近出现的 token 降低其 logits
- 惩罚系数: 1.2（基础惩罚）
- 检查窗口: 5 个 token
- 对最近 2 个 token 使用更强的惩罚（1.5x = 1.8）

**代码**:
```rust
let repetition_penalty = 1.2;
let repetition_window = 5;

// 对最近出现的 token 应用惩罚
for (i, &token_id) in recent_tokens.iter().enumerate() {
    let penalty = if i < 2 { repetition_penalty * 1.5 } else { repetition_penalty };
    if penalized_logits[token_idx] > 0.0 {
        penalized_logits[token_idx] /= penalty;
    }
}
```

**效果**: 
- 重复次数从 129 个 token 减少到 8-6 个 token
- 但问题仍然存在

### 2. 增强的重复检测

**实现位置**: `core/engine/src/nmt_incremental/m2m100_translation.rs`

**机制**:
- 在添加新 token 之前检查 2-token 循环模式
- 如果检测到 `[A, B, A]` 且 `next_token_id == B`，立即停止解码
- 在添加新 token 后再次检查 `[A, B, A, B]` 模式

**代码**:
```rust
// 在添加新 token 之前检查
if state.generated_ids.len() >= 3 {
    let last_three: Vec<i64> = state.generated_ids.iter().rev().take(3).copied().collect();
    if last_three[0] == last_three[2] && next_token_id == last_three[1] {
        println!("检测到 2-token 重复模式，停止解码");
        break;
    }
}
```

**效果**: 
- 能够及时检测并停止重复循环
- 避免生成大量重复 token

### 3. 详细的调试日志

**实现位置**: 
- `core/engine/src/nmt_incremental/m2m100_translation.rs`
- `core/engine/src/nmt_incremental/m2m100_decoder.rs`

**日志内容**:
- 每个解码步骤的输入和输出
- KV cache 的形状和内容
- 生成的 token 序列
- 重复模式检测

**效果**: 
- 便于诊断问题
- 跟踪解码过程

---

## 📊 测试结果

### 修复前
- **重复 token 数量**: 129 个
- **解码结果**: 大量 `<unk>` 和乱码
- **问题**: 无限循环

### 修复后
- **重复 token 数量**: 6-8 个（被及时检测并停止）
- **解码结果**: 仍然包含 `<unk>`，但重复问题得到缓解
- **问题**: 部分缓解，但根本问题（模型 logits 分布）仍然存在

---

## 🔍 根本问题分析

### 可能的原因

1. **模型质量问题**
   - M2M100 模型可能在某些输入下产生不稳定的 logits 分布
   - 可能需要使用更大的模型或更好的模型

2. **解码策略问题**
   - 贪婪解码可能不适合所有情况
   - 可能需要使用采样策略（temperature sampling, top-k, top-p）

3. **输入格式问题**
   - 输入文本的格式可能不符合模型的期望
   - 可能需要预处理或后处理

### 建议的进一步改进

1. **使用采样策略**
   - 实现 temperature sampling
   - 实现 top-k 或 top-p 采样
   - 避免总是选择最高 logits 的 token

2. **调整重复惩罚参数**
   - 根据测试结果调整惩罚系数
   - 可能需要动态调整惩罚强度

3. **使用更好的模型**
   - 考虑使用更大的 M2M100 模型
   - 或使用其他翻译模型

---

## 📚 参考

- 重复惩罚机制: [Hugging Face Transformers - GenerationConfig](https://huggingface.co/docs/transformers/main/en/main_classes/text_generation#transformers.GenerationConfig)
- M2M100 模型文档: [Facebook M2M100](https://github.com/facebookresearch/fairseq/tree/main/examples/m2m_100)

---

## 🎯 结论

**当前状态**: ✅ **重复 token 问题已部分缓解**

**已实施措施**:
- ✅ 重复惩罚机制
- ✅ 增强的重复检测
- ✅ 详细的调试日志

**剩余问题**:
- ⚠️ 解码结果仍然包含 `<unk>` 和乱码
- ⚠️ 根本问题（模型 logits 分布）仍然存在

**建议**:
- 继续优化重复惩罚参数
- 考虑使用采样策略替代贪婪解码
- 验证模型质量和输入格式

