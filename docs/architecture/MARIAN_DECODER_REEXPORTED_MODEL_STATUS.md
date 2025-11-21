# Marian Decoder 重新导出模型状态报告

**日期**: 2025-11-21  
**模型路径**: `core/engine/models/nmt/marian-zh-en/model.onnx`  
**状态**: ✅ **重新导出成功，输入数量正确**  
**用途**: 提交给决策部门确认

---

## 1. Decoder 模型输入签名（重新导出后）

### 1.1 从 ONNX Runtime 获取的输入节点

**总输入数**: 28 个 ✅ **正确**

```
Input[0]  name="encoder_attention_mask"              type=tensor(int64)  shape=['batch', 'src_seq']
Input[1]  name="input_ids"                           type=tensor(int64)  shape=['batch', 'tgt_seq']
Input[2]  name="encoder_hidden_states"               type=tensor(float)  shape=['batch', 'src_seq', 512]
Input[3]  name="past_key_values.0.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[4]  name="past_key_values.0.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[5]  name="past_key_values.0.encoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[6]  name="past_key_values.0.encoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[7]  name="past_key_values.1.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[8]  name="past_key_values.1.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[9]  name="past_key_values.1.encoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[10] name="past_key_values.1.encoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[11] name="past_key_values.2.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[12] name="past_key_values.2.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[13] name="past_key_values.2.encoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[14] name="past_key_values.2.encoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[15] name="past_key_values.3.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[16] name="past_key_values.3.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[17] name="past_key_values.3.encoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[18] name="past_key_values.3.encoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[19] name="past_key_values.4.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[20] name="past_key_values.4.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[21] name="past_key_values.4.encoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[22] name="past_key_values.4.encoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[23] name="past_key_values.5.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[24] name="past_key_values.5.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[25] name="past_key_values.5.encoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[26] name="past_key_values.5.encoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[27] name="use_cache_branch"                    type=tensor(bool)   shape=[1]
```

### 1.2 输入节点分类统计

| 类型 | 数量 | 输入索引 |
|------|------|---------|
| 基础输入 | 3 | Input[0-2] |
| Decoder KV | 12 | Input[3,4,7,8,11,12,15,16,19,20,23,24] |
| Encoder KV | 12 | Input[5,6,9,10,13,14,17,18,21,22,25,26] |
| use_cache_branch | 1 | Input[27] |
| **总计** | **28** | - |

### 1.3 重要发现 ⚠️

**Encoder KV 的 Shape 问题**:
- 模型定义: `['batch', 8, 'past_seq', 64]` 
- 代码构建: `[1, 8, encoder_seq_len, 64]` (例如 `[1, 8, 29, 64]`)

**问题**: 模型期望 encoder KV 的第三个维度是 `past_seq`（动态），但代码传递的是 `encoder_seq_len`（固定值 29）。这可能导致形状不匹配。

---

## 2. Decoder 模型输出签名

### 2.1 输出节点列表

**总输出数**: 25 个

```
Output[0]  name="logits"                             type=tensor(float)  shape=['batch', 'tgt_seq', 65001]
Output[1]  name="present.0.decoder.key"              type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[2]  name="present.0.decoder.value"            type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[3]  name="present.0.encoder.key"              type=tensor(float)  shape=['batch', 'Transposepresent.0.encoder.key_dim_1', 'present_seq', 'Transposepresent.0.encoder.key_dim_3']
Output[4]  name="present.0.encoder.value"            type=tensor(float)  shape=['batch', 'Transposepresent.0.encoder.value_dim_1', 'present_seq', 'Transposepresent.0.encoder.value_dim_3']
Output[5-24]: 类似结构，每层 4 个输出（decoder key/value, encoder key/value）
```

---

## 3. KV Cache 构建 Shape 日志（实际运行）

### 3.1 第一步（use_cache_branch=false）

**基础输入**:
```
[decoder_step] step input_ids_len=1, use_cache_branch=false, has_decoder_kv=false
[Input Construction] Basic inputs:
  - encoder_attention_mask: shape [1, 29]
  - decoder_input_ids: shape [1, 1]
  - encoder_hidden_states: shape [1, 29, 512]
  - use_cache_branch: false
```

**Decoder KV Cache**:
```
[KV Cache] Building zero decoder KV cache...
[KV Cache] Decoder KV cache built: 6 layers, shape [1, 8, 1, 64]
[KV Cache] Layer 0: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 1: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 2: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 3: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 4: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 5: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
```

**Encoder KV Cache**:
```
[KV Cache] Building static encoder KV cache for encoder_seq_len=29...
[KV Cache] Encoder KV cache built: 6 layers, shape [1, 8, 29, 64]
[KV Cache] Layer 0: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 1: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 2: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 3: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 4: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 5: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
```

**输入组装**:
```
[KV Cache] Assembling KV cache inputs for 6 layers...
[KV Cache] Total KV cache inputs: 24 (6 layers × 4 KV per layer)
[Input Construction] Total inputs prepared: 28
[Input Construction] Input order: encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.* (24 KV), use_cache_branch
```

### 3.2 后续步骤（use_cache_branch=true）

**Decoder KV Cache**:
```
[KV Cache] Using existing decoder KV cache
```

**Encoder KV Cache**:
```
[KV Cache] Building static encoder KV cache for encoder_seq_len=29...
[KV Cache] Encoder KV cache built: 6 layers, shape [1, 8, 29, 64]
```

**输入组装**:
```
[KV Cache] Total KV cache inputs: 24 (6 layers × 4 KV per layer)
[Input Construction] Total inputs prepared: 28
```

### 3.3 KV Cache Shape 总结

| KV Cache 类型 | 层数 | 每层 KV 数 | 代码构建 Shape | 模型期望 Shape | 状态 |
|--------------|------|-----------|---------------|---------------|------|
| Decoder KV | 6 | 2 (key, value) | [1, 8, 1, 64] | ['batch', 8, 'past_seq', 64] | ✅ 匹配 |
| Encoder KV | 6 | 2 (key, value) | [1, 8, 29, 64] | ['batch', 8, 'past_seq', 64] | ⚠️ **可能不匹配** |
| **总计** | 6 | 4 | - | - | - |

**注意**: Encoder KV 的第三个维度：
- 代码构建: 固定值 `29` (encoder_seq_len)
- 模型期望: 动态值 `past_seq`

---

## 4. 问题分析

### 4.1 输入数量 ✅

| 项目 | 当前模型 | 代码期望 | 状态 |
|------|---------|---------|------|
| 基础输入 | 3 | 3 | ✅ 匹配 |
| Decoder KV | 12 | 12 | ✅ 匹配 |
| Encoder KV | 12 | 12 | ✅ 匹配 |
| use_cache_branch | 1 | 1 | ✅ 匹配 |
| **总计** | **28** | **28** | ✅ **完全匹配** |

### 4.2 Encoder KV Shape 问题 ⚠️

**潜在问题**: 
- 模型定义 encoder KV 的第三个维度为 `past_seq`（动态）
- 代码传递的是固定值 `encoder_seq_len` (29)
- 虽然第一次运行时 `past_seq=1` 和 `encoder_seq_len=29` 可能都能工作，但后续步骤可能有问题

**需要确认**:
1. 模型是否接受 encoder KV 的第三个维度为 `encoder_seq_len` 而不是 `past_seq`？
2. 或者导出脚本应该将 encoder KV 的第三个维度也设置为动态的 `src_seq`？

---

## 5. 对比：工作模型（marian-en-zh）

### 5.1 工作模型的输入签名

**总输入数**: 28 个 ✅

**Encoder KV Shape**: 需要确认工作模型中 encoder KV 的 shape 定义

---

## 6. 总结

### 6.1 成功点 ✅

- ✅ **输入数量正确**: 28 个输入，完全匹配代码期望
- ✅ **所有输入都存在**: 包括 12 个 Encoder KV cache 输入
- ✅ **输入顺序正确**: 与代码期望的顺序一致
- ✅ **KV Cache 构建正确**: 代码正确构建了所有 24 个 KV cache

### 6.2 需要确认的问题 ⚠️

- ⚠️ **Encoder KV Shape**: 模型期望 `past_seq`，代码传递 `encoder_seq_len`
- ⚠️ **运行时行为**: 需要确认模型是否能正确处理 encoder KV 的 shape

### 6.3 建议

1. **验证运行时行为**: 运行完整的集成测试，确认模型是否能正常工作
2. **检查工作模型**: 对比 `marian-en-zh` 模型中 encoder KV 的 shape 定义
3. **如果需要**: 修改导出脚本，将 encoder KV 的第三个维度设置为 `src_seq` 而不是 `past_seq`

---

**最后更新**: 2025-11-21  
**状态**: ✅ **模型输入数量正确，但需要确认 Encoder KV Shape 兼容性**

