# Marian Decoder 模型验证报告

**日期**: 2025-11-21  
**模型路径**: `core/engine/models/nmt/marian-zh-en/model.onnx`  
**用途**: 提交给决策部门确认模型是否正确

---

## 1. Decoder 模型输入签名（Input Signature）

### 1.1 完整输入节点列表

**总输入数**: 28 个

| 索引 | 输入节点名 | 类型 | Shape |
|------|-----------|------|-------|
| 0 | `encoder_attention_mask` | tensor(int64) | `['batch', 'src_seq']` |
| 1 | `input_ids` | tensor(int64) | `['batch', 'tgt_seq']` |
| 2 | `encoder_hidden_states` | tensor(float) | `['batch', 'src_seq', 512]` |
| 3 | `past_key_values.0.decoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 4 | `past_key_values.0.decoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 5 | `past_key_values.0.encoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 6 | `past_key_values.0.encoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 7 | `past_key_values.1.decoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 8 | `past_key_values.1.decoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 9 | `past_key_values.1.encoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 10 | `past_key_values.1.encoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 11 | `past_key_values.2.decoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 12 | `past_key_values.2.decoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 13 | `past_key_values.2.encoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 14 | `past_key_values.2.encoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 15 | `past_key_values.3.decoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 16 | `past_key_values.3.decoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 17 | `past_key_values.3.encoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 18 | `past_key_values.3.encoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 19 | `past_key_values.4.decoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 20 | `past_key_values.4.decoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 21 | `past_key_values.4.encoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 22 | `past_key_values.4.encoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 23 | `past_key_values.5.decoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 24 | `past_key_values.5.decoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 25 | `past_key_values.5.encoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 26 | `past_key_values.5.encoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 27 | `use_cache_branch` | tensor(bool) | `[1]` |

### 1.2 输入节点分类统计

| 类型 | 数量 | 输入索引 | 节点名模式 |
|------|------|---------|-----------|
| **基础输入** | 3 | Input[0-2] | `encoder_attention_mask`, `input_ids`, `encoder_hidden_states` |
| **Decoder KV Cache** | 12 | Input[3,4,7,8,11,12,15,16,19,20,23,24] | `past_key_values.{0-5}.decoder.{key,value}` |
| **Encoder KV Cache** | 12 | Input[5,6,9,10,13,14,17,18,21,22,25,26] | `past_key_values.{0-5}.encoder.{key,value}` |
| **控制标志** | 1 | Input[27] | `use_cache_branch` |
| **总计** | **28** | - | - |

### 1.3 输入节点名列表（按顺序）

```
encoder_attention_mask
input_ids
encoder_hidden_states
past_key_values.0.decoder.key
past_key_values.0.decoder.value
past_key_values.0.encoder.key
past_key_values.0.encoder.value
past_key_values.1.decoder.key
past_key_values.1.decoder.value
past_key_values.1.encoder.key
past_key_values.1.encoder.value
past_key_values.2.decoder.key
past_key_values.2.decoder.value
past_key_values.2.encoder.key
past_key_values.2.encoder.value
past_key_values.3.decoder.key
past_key_values.3.decoder.value
past_key_values.3.encoder.key
past_key_values.3.encoder.value
past_key_values.4.decoder.key
past_key_values.4.decoder.value
past_key_values.4.encoder.key
past_key_values.4.encoder.value
past_key_values.5.decoder.key
past_key_values.5.decoder.value
past_key_values.5.encoder.key
past_key_values.5.encoder.value
use_cache_branch
```

---

## 2. Decoder 模型输出签名（Output Signature）

### 2.1 完整输出节点列表

**总输出数**: 25 个

| 索引 | 输出节点名 | 类型 | Shape |
|------|-----------|------|-------|
| 0 | `logits` | tensor(float) | `['batch', 'tgt_seq', 65001]` |
| 1 | `present.0.decoder.key` | tensor(float) | `['batch', 8, 'present_seq', 64]` |
| 2 | `present.0.decoder.value` | tensor(float) | `['batch', 8, 'present_seq', 64]` |
| 3 | `present.0.encoder.key` | tensor(float) | `['batch', 'Transposepresent.0.encoder.key_dim_1', 'present_seq', 'Transposepresent.0.encoder.key_dim_3']` |
| 4 | `present.0.encoder.value` | tensor(float) | `['batch', 'Transposepresent.0.encoder.value_dim_1', 'present_seq', 'Transposepresent.0.encoder.value_dim_3']` |
| 5-24 | `present.{1-5}.{decoder,encoder}.{key,value}` | tensor(float) | 类似结构 |

---

## 3. KV Cache 构建日志（Shape 信息）

### 3.1 第一步（use_cache_branch=false）

#### 基础输入 Shape

```
[decoder_step] step input_ids_len=1, use_cache_branch=false, has_decoder_kv=false
[Input Construction] Basic inputs:
  - encoder_attention_mask: shape [1, 29]
  - decoder_input_ids: shape [1, 1]
  - encoder_hidden_states: shape [1, 29, 512]
  - use_cache_branch: false
```

#### Decoder KV Cache 构建

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

**Decoder KV Cache 总结**:
- 层数: 6 层
- 每层 KV 数: 2 个（key, value）
- Shape: `[1, 8, 1, 64]` = `[batch, num_heads, past_seq, head_dim]`
- 总 KV 数: 12 个（6 层 × 2 KV）

#### Encoder KV Cache 构建

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

**Encoder KV Cache 总结**:
- 层数: 6 层
- 每层 KV 数: 2 个（key, value）
- Shape: `[1, 8, 29, 64]` = `[batch, num_heads, encoder_seq_len, head_dim]`
- 总 KV 数: 12 个（6 层 × 2 KV）
- **注意**: 模型期望的第三个维度是 `past_seq`（动态），但代码传递的是 `encoder_seq_len`（固定值 29）

#### 输入组装

```
[KV Cache] Assembling KV cache inputs for 6 layers...
[KV Cache] Total KV cache inputs: 24 (6 layers × 4 KV per layer)
[Input Construction] Total inputs prepared: 28
[Input Construction] Input order: encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.* (24 KV), use_cache_branch
```

### 3.2 后续步骤（use_cache_branch=true）

#### Decoder KV Cache

```
[KV Cache] Using existing decoder KV cache
```

#### Encoder KV Cache

```
[KV Cache] Building static encoder KV cache for encoder_seq_len=29...
[KV Cache] Encoder KV cache built: 6 layers, shape [1, 8, 29, 64]
```

#### 输入组装

```
[KV Cache] Total KV cache inputs: 24 (6 layers × 4 KV per layer)
[Input Construction] Total inputs prepared: 28
```

### 3.3 KV Cache Shape 对比表

| KV Cache 类型 | 层数 | 每层 KV 数 | 代码构建 Shape | 模型期望 Shape | 状态 |
|--------------|------|-----------|---------------|---------------|------|
| **Decoder KV** | 6 | 2 (key, value) | `[1, 8, 1, 64]` | `['batch', 8, 'past_seq', 64]` | ✅ **匹配** |
| **Encoder KV** | 6 | 2 (key, value) | `[1, 8, 29, 64]` | `['batch', 8, 'past_seq', 64]` | ⚠️ **可能不匹配** |
| **总计** | 6 | 4 | - | - | - |

**关键问题**:
- **Decoder KV**: 代码构建的 `past_seq=1` 与模型期望的动态 `past_seq` 匹配 ✅
- **Encoder KV**: 代码构建的 `encoder_seq_len=29`（固定值）与模型期望的动态 `past_seq` **可能不匹配** ⚠️

---

## 4. 关键发现和问题

### 4.1 输入数量 ✅

| 项目 | 当前模型 | 代码期望 | 状态 |
|------|---------|---------|------|
| 基础输入 | 3 | 3 | ✅ 匹配 |
| Decoder KV | 12 | 12 | ✅ 匹配 |
| Encoder KV | 12 | 12 | ✅ 匹配 |
| use_cache_branch | 1 | 1 | ✅ 匹配 |
| **总计** | **28** | **28** | ✅ **完全匹配** |

### 4.2 输入节点名 ✅

所有输入节点名与代码期望完全匹配：
- ✅ 基础输入: `encoder_attention_mask`, `input_ids`, `encoder_hidden_states`
- ✅ Decoder KV: `past_key_values.{0-5}.decoder.{key,value}`
- ✅ Encoder KV: `past_key_values.{0-5}.encoder.{key,value}`
- ✅ 控制标志: `use_cache_branch`

### 4.3 Encoder KV Shape 问题 ⚠️

**潜在问题**: 
- **模型定义**: Encoder KV 的第三个维度为 `past_seq`（动态维度）
- **代码传递**: 固定值 `encoder_seq_len` (29)
- **影响**: 虽然第一次运行时可能工作，但模型可能期望 `past_seq` 与 decoder KV 的 `past_seq` 保持一致，而不是使用 `encoder_seq_len`

**需要确认**:
1. 模型是否接受 encoder KV 的第三个维度为 `encoder_seq_len` 而不是 `past_seq`？
2. 或者导出脚本应该将 encoder KV 的第三个维度也设置为动态的 `src_seq`？
3. 工作模型（`marian-en-zh`）中 encoder KV 的 shape 定义是什么？

---

## 5. 总结

### 5.1 成功点 ✅

- ✅ **输入数量正确**: 28 个输入，完全匹配代码期望
- ✅ **所有输入都存在**: 包括 12 个 Encoder KV cache 输入
- ✅ **输入顺序正确**: 与代码期望的顺序一致
- ✅ **输入节点名正确**: 所有节点名与代码期望完全匹配
- ✅ **KV Cache 构建正确**: 代码正确构建了所有 24 个 KV cache

### 5.2 需要确认的问题 ⚠️

- ⚠️ **Encoder KV Shape**: 模型期望 `past_seq`（动态），代码传递 `encoder_seq_len`（固定值 29）
- ⚠️ **运行时行为**: 需要确认模型是否能正确处理 encoder KV 的 shape
- ⚠️ **翻译质量**: 当前翻译结果重复且无意义，可能与 shape 不匹配有关

### 5.3 建议

1. **对比工作模型**: 检查 `marian-en-zh` 模型中 encoder KV 的 shape 定义
2. **验证导出脚本**: 确认 `export_marian_decoder_ir9_fixed.py` 中 encoder KV 的 dynamic axes 设置是否正确
3. **测试运行时行为**: 运行完整的集成测试，观察模型是否能正常工作
4. **如果问题持续**: 考虑修改导出脚本，将 encoder KV 的第三个维度设置为 `src_seq` 而不是 `past_seq`

---

**最后更新**: 2025-11-21  
**状态**: ✅ **输入数量和节点名正确，但需要确认 Encoder KV Shape 兼容性**

