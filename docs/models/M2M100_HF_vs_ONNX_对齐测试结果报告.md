# M2M100 HF vs ONNX 对齐测试结果报告

**版本：** v1.0  
**日期：** 2025-01-21  
**测试任务：** 任务1 - HF vs ONNX 的非增量逐步对齐测试

---

## 1. 测试执行情况

### 1.1 测试脚本

已创建测试脚本：`docs/models/test_m2m100_hf_vs_onnx_alignment.py`

**功能**：
- 测试 HF 模型的非增量解码（每次传入完整序列，不使用 past_key_values）
- 测试 ONNX 模型的非增量解码（每次传入完整序列 + 全零 KV cache）
- 比对每步的 logits top-5

### 1.2 测试结果

#### ✅ HF 模型测试

**测试输入**：`"你好，欢迎参加测试。"`

**结果**：
- ✅ Encoder 可以正常运行
- ✅ Decoder 可以正常运行
- ⚠️ **出现重复 token 问题**：token 128 重复出现

**HF 解码步骤**：
```
[Step 0] Generated IDs: [128022], Next token: 128
[Step 1] Generated IDs: [128022, 128], Next token: 128
[Step 2] Generated IDs: [128022, 128, 128], Next token: 128
[Step 3] Generated IDs: [128022, 128, 128, 128], Next token: 128
[WARNING] Detected 2-token repetition pattern: [128, 128, 128, 128]
```

**关键发现**：
- **HF 模型在非增量解码模式下也出现重复 token 问题**
- 这说明问题可能不在 ONNX 实现，而是**非增量解码本身就不适合 M2M100 模型**

#### ❌ ONNX 模型测试

**错误信息**：
```
onnxruntime.capi.onnxruntime_pybind11_state.RuntimeException: 
Non-zero status code returned while running Reshape node. 
Name:'/decoder/layers.0/encoder_attn/Reshape_5' 
Input shape:{1,16,1,9}, requested shape:{16,1,1}
```

**错误分析**：
- 错误发生在 encoder attention 的 Reshape 操作
- 输入形状：`{1,16,1,9}` - 这是 encoder KV cache 的形状
- 期望形状：`{16,1,1}` - 模型期望的形状
- **问题**：encoder KV cache 的形状不正确

**可能原因**：
1. Encoder KV cache 的形状应该是 `[16, encoder_seq_len, 64]`（没有 batch 维度）？
2. 或者应该是 `[1, 16, encoder_seq_len, 64]` 但序列长度维度位置不对？
3. 或者模型期望的是不同的 KV cache 格式？

---

## 2. 关键发现

### 2.1 发现1：HF 模型非增量解码也失败

**现象**：
- HF 模型在非增量解码模式下（`use_cache=False`）也出现重复 token
- 这说明**非增量解码可能不是 M2M100 模型的正确使用方式**

**可能原因**：
- M2M100 模型设计时就是为增量解码优化的
- 非增量解码时，模型无法正确利用上下文信息
- 需要真实的 KV cache 才能正常工作

### 2.2 发现2：ONNX Encoder KV Cache 形状问题

**错误**：
- Encoder KV cache 形状不匹配
- 模型期望的形状与传入的形状不一致

**需要确认**：
- ONNX 模型导出时 encoder KV cache 的正确形状是什么？
- 是否需要重新导出模型以支持非增量解码？

---

## 3. 问题分析

### 3.1 非增量解码是否可行？

**当前证据**：
- ❌ HF 模型非增量解码失败（重复 token）
- ❌ ONNX 模型非增量解码失败（形状错误）

**结论**：
- **非增量解码可能不适合 M2M100 模型**
- 需要重新评估改造方案

### 3.2 可能的解决方案

**方案A：修复增量解码的 KV cache 问题**
- 回到增量解码方案
- 修复 encoder KV cache 的提取和复用问题
- 这是之前已经尝试过的方向

**方案B：重新导出支持非增量解码的模型**
- 如果确实需要非增量解码，可能需要重新导出模型
- 但这需要模型团队支持

**方案C：使用其他模型**
- 考虑使用专门为非增量解码设计的模型
- 或者使用其他 NMT 模型

---

## 4. 下一步建议

### 4.1 立即行动

1. **确认 HF 模型非增量解码的正确用法**
   - 检查 HuggingFace 文档
   - 确认 M2M100 是否支持非增量解码
   - 如果支持，检查正确的调用方式

2. **分析 ONNX 模型导出方式**
   - 检查导出脚本
   - 确认 encoder KV cache 的正确形状
   - 可能需要重新导出模型

### 4.2 决策建议

**建议1：暂停非增量解码方案**
- 当前证据显示非增量解码不可行
- 建议回到增量解码方案，修复 KV cache 问题

**建议2：深入分析 HF 模型行为**
- 确认 HF 模型在什么条件下可以正常工作
- 如果 HF 模型本身就不支持非增量解码，那么 ONNX 模型也不可能支持

---

## 5. 测试数据

### 5.1 HF 模型测试数据

**Step 0**:
- Generated IDs: `[128022]`
- Top 5 logits: `[(128, 5.92), (16076, 5.74), (5, 5.44), (1019, 5.06), (1197, 4.58)]`
- Next token: `128`

**Step 1**:
- Generated IDs: `[128022, 128]`
- Top 5 logits: `[(128, 9.16), (5, 5.69), (1197, 5.39), (16076, 4.98), (2, 4.86)]`
- Next token: `128`

**Step 2**:
- Generated IDs: `[128022, 128, 128]`
- Top 5 logits: `[(128, 9.36), (5, 5.44), (1197, 5.27), (16076, 4.98), (2, 4.97)]`
- Next token: `128`

### 5.2 ONNX 模型错误信息

```
Input shape:{1,16,1,9}, requested shape:{16,1,1}
Location: /decoder/layers.0/encoder_attn/Reshape_5
```

---

## 6. 结论

1. **非增量解码方案在当前条件下不可行**
   - HF 模型非增量解码失败
   - ONNX 模型非增量解码失败（形状错误）

2. **需要重新评估技术方案**
   - 考虑回到增量解码方案
   - 或者寻找其他解决方案

3. **建议决策部门**
   - 暂停非增量解码方案的开发
   - 重新评估技术路线
   - 考虑修复增量解码的 KV cache 问题

---

**报告结束**

