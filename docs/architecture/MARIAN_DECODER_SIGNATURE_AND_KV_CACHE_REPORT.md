# Marian Decoder 模型签名和 KV Cache 构建报告

**日期**: 2025-11-21  
**模型**: `marian-zh-en` (IR 7, Opset 12)  
**状态**: 🔴 **发现根本问题：模型输入数量不匹配**

---

## 1. Decoder 模型输入签名（实际导出）

### 1.1 从 ONNX Runtime 获取的输入节点

**总输入数**: 15 个 ❌ **不匹配**

```
--- Decoder ONNX Model Inputs ---
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

### 1.2 Decoder 模型输出签名

**总输出数**: 25 个

```
--- Decoder ONNX Model Outputs ---
Output[0]  name="logits"                             output_type=Float32
Output[1]  name="present.0.decoder.key"              output_type=Float32
Output[2]  name="present.0.decoder.value"            output_type=Float32
Output[3]  name="present.0.encoder.key"              output_type=Float32
Output[4]  name="present.0.encoder.value"            output_type=Float32
Output[5]  name="present.1.decoder.key"              output_type=Float32
Output[6]  name="present.1.decoder.value"            output_type=Float32
Output[7]  name="present.1.encoder.key"              output_type=Float32
Output[8]  name="present.1.encoder.value"            output_type=Float32
Output[9]  name="present.2.decoder.key"              output_type=Float32
Output[10] name="present.2.decoder.value"            output_type=Float32
Output[11] name="present.2.encoder.key"              output_type=Float32
Output[12] name="present.2.encoder.value"            output_type=Float32
Output[13] name="present.3.decoder.key"              output_type=Float32
Output[14] name="present.3.decoder.value"            output_type=Float32
Output[15] name="present.3.encoder.key"              output_type=Float32
Output[16] name="present.3.encoder.value"            output_type=Float32
Output[17] name="present.4.decoder.key"              output_type=Float32
Output[18] name="present.4.decoder.value"            output_type=Float32
Output[19] name="present.4.encoder.key"              output_type=Float32
Output[20] name="present.4.encoder.value"            output_type=Float32
Output[21] name="present.5.decoder.key"              output_type=Float32
Output[22] name="present.5.decoder.value"            output_type=Float32
Output[23] name="present.5.encoder.key"              output_type=Float32
Output[24] name="present.5.encoder.value"            output_type=Float32
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

### 2.2 期望的 KV Cache 输入详情

**每层 4 个 KV cache**:
- `past_key_values.{layer}.decoder.key`   - [1, 8, past_seq, 64] (f32) ✅ 存在
- `past_key_values.{layer}.decoder.value` - [1, 8, past_seq, 64] (f32) ✅ 存在
- `past_key_values.{layer}.encoder.key`   - [1, 8, encoder_seq_len, 64] (f32) ❌ **缺失**
- `past_key_values.{layer}.encoder.value` - [1, 8, encoder_seq_len, 64] (f32) ❌ **缺失**

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

**导出脚本问题**: `export_marian_decoder_ir9_fixed.py` 的 Wrapper 类虽然定义了输入名称，但实际导出时 PyTorch 的 `torch.onnx.export` 可能没有正确导出所有输入。

**证据**:
- 输入名称列表包含 encoder KV 和 use_cache_branch（`build_io_names` 函数）
- 但实际导出的模型只有 15 个输入
- 说明 Wrapper 的 `forward` 方法可能没有正确接受这些输入

---

## 4. KV Cache 构建代码和 Shape

### 4.1 Decoder KV Cache 构建

**函数**: `build_zero_decoder_kv()`

**代码位置**: `core/engine/src/nmt_incremental/decoder.rs:63-98`

**Shape**: `[1, 8, 1, 64]` (batch, num_heads, past_seq, head_dim)

**常量**:
- `NUM_LAYERS = 6`
- `NUM_HEADS = 8`
- `HEAD_DIM = 64`

**构建过程**:
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

**代码位置**: `core/engine/src/nmt_incremental/decoder.rs:17-57`

**Shape**: `[1, 8, encoder_seq_len, 64]` (batch, num_heads, encoder_seq_len, head_dim)

**构建过程**:
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

### 4.3 输入构造顺序（代码）

**代码位置**: `core/engine/src/nmt_incremental/decoder.rs:160-208`

```rust
// 1. 基础输入（3 个）
input_values.push(encoder_mask_value);      // [1, 29] (i64)
input_values.push(input_ids_value);         // [1, 1] (i64)
input_values.push(encoder_states_value);    // [1, 29, 512] (f32)

// 2. KV Cache（24 个，每层 4 个）
for layer in 0..6 {
    input_values.push(dec_k);  // [1, 8, 1, 64] (f32) ✅ 模型中有
    input_values.push(dec_v);  // [1, 8, 1, 64] (f32) ✅ 模型中有
    input_values.push(enc_k);  // [1, 8, 29, 64] (f32) ❌ 模型中缺失
    input_values.push(enc_v);  // [1, 8, 29, 64] (f32) ❌ 模型中缺失
}

// 3. use_cache_branch（1 个）
input_values.push(use_cache_value);  // [1] (bool) ❌ 模型中缺失
```

---

## 5. KV Cache Shape 日志（预期输出）

### 5.1 第一步（use_cache_branch=false）

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

**错误**: 代码尝试传递 28 个输入，但模型只有 15 个输入，导致访问违规错误。

---

## 6. 对比：工作模型（marian-en-zh）

### 6.1 工作模型的输入签名

**总输入数**: 28 个 ✅

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

### 6.2 关键差异

| 项目 | marian-en-zh (工作) | marian-zh-en IR 7 (失败) |
|------|---------------------|--------------------------|
| 总输入数 | 28 | 15 |
| Encoder KV | ✅ 12 个 | ❌ 0 个 |
| use_cache_branch | ✅ 1 个 | ❌ 0 个 |
| 状态 | ✅ 正常工作 | ❌ 无法使用 |

---

## 7. 问题总结

### 7.1 核心问题

**导出脚本没有正确导出 Encoder KV Cache 和 use_cache_branch 输入**

- ❌ 模型只有 15 个输入
- ❌ 代码期望 28 个输入
- ❌ 缺少 12 个 Encoder KV cache 输入
- ❌ 缺少 1 个 use_cache_branch 输入

### 7.2 为什么会导致访问违规错误

当代码尝试传递 28 个输入给只有 15 个输入的模型时：
- ONNX Runtime 在 `decoder_session.run(input_values)` 时尝试访问不存在的输入
- 导致内存访问违规（STATUS_ACCESS_VIOLATION, 0xc0000005）

### 7.3 修复方案

**必须修改导出脚本** `export_marian_decoder_ir9_fixed.py`:

1. **检查 Wrapper 类的 forward 方法**
   - 确保接受 encoder KV cache 作为输入
   - 确保接受 use_cache_branch 作为输入

2. **检查导出时的 dummy_inputs**
   - 确保包含 encoder KV cache 张量
   - 确保包含 use_cache_branch 张量

3. **验证导出的模型**
   - 导出后立即检查输入数量（应该是 28 个）

---

## 8. 完整的输入构造代码

### 8.1 Rust 代码位置

`core/engine/src/nmt_incremental/decoder.rs:108-213`

### 8.2 关键代码片段

```rust
// 1. 基础输入准备
let decoder_input_ids = Array2::<i64>::from_shape_vec((batch_size, cur_len), state.input_ids.clone())?;
let use_cache_array = Array1::<bool>::from_vec(vec![state.use_cache_branch]);

// 2. 转换为 ONNX Value
let input_ids_value = array_to_value!(decoder_input_ids.clone(), i64)?;
let encoder_states_value = array_to_value!(encoder_hidden_states.clone(), f32)?;
let encoder_mask_value = array_to_value!(encoder_attention_mask.clone(), i64)?;
let use_cache_value = array_to_value!(use_cache_array, bool)?;

// 3. 组织输入顺序
let mut input_values: Vec<Value<'static>> = Vec::new();
input_values.push(encoder_mask_value);      // [1, 29] (i64)
input_values.push(input_ids_value);         // [1, 1] (i64)
input_values.push(encoder_states_value);    // [1, 29, 512] (f32)

// 4. KV Cache（每层 4 个：dec_k, dec_v, enc_k, enc_v）
let decoder_kv = self.build_zero_decoder_kv()?;  // 12 个 (6 层 × 2)
let static_enc_kv = self.build_static_encoder_kv(encoder_seq_len)?;  // 12 个 (6 层 × 2)

for layer_idx in 0..6 {
    let (dec_k, dec_v) = decoder_kv_iter.next()?;
    input_values.push(dec_k);   // [1, 8, 1, 64] ✅
    input_values.push(dec_v);   // [1, 8, 1, 64] ✅
    
    let (enc_k, enc_v) = static_enc_kv_iter.next()?;
    input_values.push(enc_k);   // [1, 8, 29, 64] ❌ 模型中没有
    input_values.push(enc_v);   // [1, 8, 29, 64] ❌ 模型中没有
}

input_values.push(use_cache_value);  // [1] (bool) ❌ 模型中没有

// 5. 调用 session.run（错误发生在这里）
let outputs = decoder_session.run(input_values)?;  // ❌ 28 输入 vs 15 输入
```

---

## 9. 修复建议

### 9.1 立即修复

1. **检查导出脚本的 Wrapper.forward 方法**
   - 确保接受所有 28 个输入
   - 确保 encoder KV cache 被正确传递

2. **检查导出时的 dummy_inputs**
   - 确保包含 encoder KV cache（12 个张量）
   - 确保包含 use_cache_branch（1 个张量）

3. **重新导出模型**
   - 使用修复后的脚本
   - 验证导出后的模型有 28 个输入

### 9.2 验证步骤

```bash
# 1. 重新导出模型
python export_marian_decoder_ir9_fixed.py --output_dir core/engine/models/nmt/marian-zh-en

# 2. 验证输入数量
python scripts/get_decoder_model_signature.py
# 应该显示: Total inputs: 28

# 3. 运行测试
cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav
```

---

**最后更新**: 2025-11-21  
**状态**: 🔴 **根本原因已确定：导出脚本没有正确导出 Encoder KV Cache 和 use_cache_branch 输入**

