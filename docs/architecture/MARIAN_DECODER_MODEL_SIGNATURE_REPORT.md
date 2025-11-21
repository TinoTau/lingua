# Marian Decoder 模型签名和 KV Cache 构建报告

**日期**: 2025-11-21  
**模型**: `marian-zh-en` (IR 7, Opset 12)  
**状态**: 🔴 **发现严重不匹配问题**

---

## 1. 模型输入签名（实际导出）

### 1.1 输入节点列表（从 ONNX Runtime 获取）

**总输入数**: 15 个 ❌ **不匹配**

```
Input[0]  name="encoder_attention_mask"              input_type=Int64
Input[1]  name="input_ids"                           input_type=Int64
Input[2]  name="encoder_hidden_states"               input_type=Float32
Input[3]  name="past_key_values.0.decoder.key"       input_type=Float32
Input[4]  name="past_key_values.0.decoder.value"     input_type=Float32
Input[5]  name="past_key_values.1.decoder.key"       input_type=Float32
Input[6]  name="past_key_values.1.decoder.value"     input_type=Float32
Input[7]  name="past_key_values.2.decoder.key"       input_type=Float32
Input[8]  name="past_key_values.2.decoder.value"     input_type=Float32
Input[9]  name="past_key_values.3.decoder.key"       input_type=Float32
Input[10] name="past_key_values.3.decoder.value"     input_type=Float32
Input[11] name="past_key_values.4.decoder.key"       input_type=Float32
Input[12] name="past_key_values.4.decoder.value"     input_type=Float32
Input[13] name="past_key_values.5.decoder.key"       input_type=Float32
Input[14] name="past_key_values.5.decoder.value"     input_type=Float32
```

**缺失的输入**:
- ❌ `past_key_values.{0-5}.encoder.key` (6 个)
- ❌ `past_key_values.{0-5}.encoder.value` (6 个)
- ❌ `use_cache_branch` (1 个)

**总计缺失**: 13 个输入

### 1.2 模型输出签名

**总输出数**: 25 个

```
Output[0]  name="logits"  type=tensor(float)  shape=['batch', 'tgt_seq', 65001]
Output[1-24] name="present.{layer}.{decoder|encoder}.{key|value}"  (6 层 × 4 KV = 24 个)
```

---

## 2. 代码期望的输入签名

### 2.1 期望的输入顺序和数量

**总输入数**: 28 个

```
1. encoder_attention_mask          - [1, encoder_seq_len] (i64)
2. input_ids                       - [1, decoder_seq_len] (i64)
3. encoder_hidden_states           - [1, encoder_seq_len, 512] (f32)
4-27. past_key_values.*            - 6 层 × 4 KV = 24 个
   - 每层 4 个: dec_k, dec_v, enc_k, enc_v
28. use_cache_branch               - [1] (bool)
```

### 2.2 期望的 KV Cache 输入

**每层 4 个 KV cache**:
- `past_key_values.{layer}.decoder.key`   - [1, 8, past_seq, 64] (f32)
- `past_key_values.{layer}.decoder.value` - [1, 8, past_seq, 64] (f32)
- `past_key_values.{layer}.encoder.key`   - [1, 8, encoder_seq_len, 64] (f32)  ❌ **缺失**
- `past_key_values.{layer}.encoder.value` - [1, 8, encoder_seq_len, 64] (f32)  ❌ **缺失**

**总计**: 6 层 × 4 KV = 24 个 KV cache 输入

---

## 3. 问题分析 🔴

### 3.1 输入数量不匹配

| 项目 | 实际模型 | 代码期望 | 差异 |
|------|---------|---------|------|
| 基础输入 | 3 | 3 | ✅ 匹配 |
| Decoder KV | 12 (6 层 × 2) | 12 (6 层 × 2) | ✅ 匹配 |
| Encoder KV | 0 | 12 (6 层 × 2) | ❌ **缺失 12 个** |
| use_cache_branch | 0 | 1 | ❌ **缺失 1 个** |
| **总计** | **15** | **28** | ❌ **缺少 13 个输入** |

### 3.2 根本原因

**导出脚本问题**: `export_marian_decoder_ir9_fixed.py` 只导出了 decoder KV cache，**没有导出 encoder KV cache 和 use_cache_branch**。

**证据**:
- 模型只有 15 个输入（3 基础 + 12 decoder KV）
- 代码期望 28 个输入（3 基础 + 12 decoder KV + 12 encoder KV + 1 use_cache_branch）
- 缺少 13 个输入

---

## 4. KV Cache 构建代码和 Shape

### 4.1 Decoder KV Cache 构建

**函数**: `build_zero_decoder_kv()`

**Shape**: `[1, 8, 1, 64]` (batch, num_heads, past_seq, head_dim)

**代码**:
```rust
let zeros_dec = Array4::<f32>::zeros((batch, Self::NUM_HEADS, dec_len, Self::HEAD_DIM));
// batch = 1, NUM_HEADS = 8, dec_len = 1, HEAD_DIM = 64
// Shape: [1, 8, 1, 64]
```

**每层**: 2 个 KV（dec_k, dec_v）  
**总层数**: 6 层  
**总计**: 12 个 Decoder KV cache 输入

### 4.2 Encoder KV Cache 构建

**函数**: `build_static_encoder_kv(encoder_seq_len)`

**Shape**: `[1, 8, encoder_seq_len, 64]` (batch, num_heads, encoder_seq_len, head_dim)

**代码**:
```rust
let zeros_enc = Array4::<f32>::zeros((batch, Self::NUM_HEADS, enc_len, Self::HEAD_DIM));
// batch = 1, NUM_HEADS = 8, enc_len = encoder_seq_len, HEAD_DIM = 64
// Shape: [1, 8, encoder_seq_len, 64]
```

**每层**: 2 个 KV（enc_k, enc_v）  
**总层数**: 6 层  
**总计**: 12 个 Encoder KV cache 输入

**示例**（encoder_seq_len = 29）:
- Shape: `[1, 8, 29, 64]`

### 4.3 输入构造顺序

```rust
// 1. 基础输入（3 个）
input_values.push(encoder_mask_value);      // [1, 29] (i64)
input_values.push(input_ids_value);         // [1, 1] (i64)
input_values.push(encoder_states_value);    // [1, 29, 512] (f32)

// 2. KV Cache（24 个，每层 4 个）
for layer in 0..6 {
    input_values.push(dec_k);  // [1, 8, 1, 64] (f32)
    input_values.push(dec_v);  // [1, 8, 1, 64] (f32)
    input_values.push(enc_k);  // [1, 8, 29, 64] (f32)  ❌ 模型中没有
    input_values.push(enc_v);  // [1, 8, 29, 64] (f32)  ❌ 模型中没有
}

// 3. use_cache_branch（1 个）
input_values.push(use_cache_value);  // [1] (bool)  ❌ 模型中没有
```

---

## 5. 对比：工作模型（marian-en-zh）

### 5.1 工作模型的输入签名

**总输入数**: 28 个

```
Input[0]  encoder_attention_mask
Input[1]  input_ids
Input[2]  encoder_hidden_states
Input[3]  past_key_values.0.decoder.key
Input[4]  past_key_values.0.decoder.value
Input[5]  past_key_values.0.encoder.key      ✅ 存在
Input[6]  past_key_values.0.encoder.value    ✅ 存在
Input[7]  past_key_values.1.decoder.key
Input[8]  past_key_values.1.decoder.value
Input[9]  past_key_values.1.encoder.key      ✅ 存在
Input[10] past_key_values.1.encoder.value    ✅ 存在
... (重复 6 层)
Input[27] use_cache_branch                   ✅ 存在
```

### 5.2 关键差异

| 项目 | marian-en-zh (工作) | marian-zh-en IR 7 (失败) |
|------|---------------------|--------------------------|
| 总输入数 | 28 | 15 |
| Encoder KV | ✅ 12 个 | ❌ 0 个 |
| use_cache_branch | ✅ 1 个 | ❌ 0 个 |
| 状态 | ✅ 正常工作 | ❌ 无法使用 |

---

## 6. KV Cache Shape 日志（代码输出）

### 6.1 第一步（use_cache_branch=false）

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

**输入构造**:
```
[Input Construction] Basic inputs:
  - encoder_attention_mask: shape [1, 29]
  - decoder_input_ids: shape [1, 1]
  - encoder_hidden_states: shape [1, 29, 512]
  - use_cache_branch: false

[KV Cache] Assembling KV cache inputs for 6 layers...
[KV Cache] Total KV cache inputs: 24 (6 layers × 4 KV per layer)
[Input Construction] Total inputs prepared: 28
[Input Construction] Input order: encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.* (24 KV), use_cache_branch
[Decoder] Calling decoder_session.run() with 28 inputs...
```

**注意**: 代码尝试传递 28 个输入，但模型只有 15 个输入，导致访问违规错误。

---

## 7. 问题总结

### 7.1 核心问题

**导出脚本缺少 Encoder KV Cache 和 use_cache_branch 输入**

- ❌ 模型只有 15 个输入
- ❌ 代码期望 28 个输入
- ❌ 缺少 12 个 Encoder KV cache 输入
- ❌ 缺少 1 个 use_cache_branch 输入

### 7.2 为什么会导致访问违规错误

当代码尝试传递 28 个输入给只有 15 个输入的模型时：
- ONNX Runtime 可能尝试访问不存在的输入
- 导致内存访问违规（STATUS_ACCESS_VIOLATION）

### 7.3 修复方案

**必须修改导出脚本** `export_marian_decoder_ir9_fixed.py`:

1. **添加 Encoder KV Cache 输入**（12 个）
   - 每层 2 个：`past_key_values.{layer}.encoder.key`, `past_key_values.{layer}.encoder.value`
   - Shape: `[1, 8, encoder_seq_len, 64]`

2. **添加 use_cache_branch 输入**（1 个）
   - 类型: `bool` 或 `int64`
   - Shape: `[1]`

3. **修正 Wrapper 类的 forward 方法**
   - 接受 encoder KV cache 作为输入
   - 接受 use_cache_branch 作为输入

4. **修正输入名称和顺序**
   - 确保与代码期望完全匹配

---

## 8. 参考：工作模型的导出脚本

参考 `scripts/export_marian_onnx.py` 中的 `export_decoder_with_past` 函数，它正确导出了：
- ✅ Encoder KV cache 输入
- ✅ use_cache_branch 输入
- ✅ 正确的输入顺序

---

**最后更新**: 2025-11-21  
**状态**: 🔴 **发现根本原因：导出脚本缺少 Encoder KV Cache 和 use_cache_branch 输入**

