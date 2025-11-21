# Marian Decoder 当前模型状态报告

**日期**: 2025-11-21  
**模型路径**: `core/engine/models/nmt/marian-zh-en/model.onnx`  
**用途**: 提交给决策部门确认

---

## 1. Decoder 模型输入签名（当前状态）

### 1.1 从 ONNX Runtime 获取的输入节点（实际模型）

**总输入数**: 16 个 ❌ **不完整**

```
Input[0]  name="encoder_attention_mask"              type=tensor(int64)  shape=['batch', 'src_seq']
Input[1]  name="input_ids"                           type=tensor(int64)  shape=['batch', 'tgt_seq']
Input[2]  name="encoder_hidden_states"               type=tensor(float)  shape=['batch', 'src_seq', 512]
Input[3]  name="past_key_values.0.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[4]  name="past_key_values.0.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[5]  name="past_key_values.1.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[6]  name="past_key_values.1.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[7]  name="past_key_values.2.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[8]  name="past_key_values.2.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[9]  name="past_key_values.3.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[10] name="past_key_values.3.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[11] name="past_key_values.4.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[12] name="past_key_values.4.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[13] name="past_key_values.5.decoder.key"       type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[14] name="past_key_values.5.decoder.value"     type=tensor(float)  shape=['batch', 8, 'past_seq', 64]
Input[15] name="use_cache_branch"                    type=tensor(bool)   shape=[1]
```

### 1.2 缺失的输入

**缺失的 Encoder KV Cache 输入**（12 个）:
- ❌ `past_key_values.0.encoder.key`
- ❌ `past_key_values.0.encoder.value`
- ❌ `past_key_values.1.encoder.key`
- ❌ `past_key_values.1.encoder.value`
- ❌ `past_key_values.2.encoder.key`
- ❌ `past_key_values.2.encoder.value`
- ❌ `past_key_values.3.encoder.key`
- ❌ `past_key_values.3.encoder.value`
- ❌ `past_key_values.4.encoder.key`
- ❌ `past_key_values.4.encoder.value`
- ❌ `past_key_values.5.encoder.key`
- ❌ `past_key_values.5.encoder.value`

**总计缺失**: 12 个 Encoder KV cache 输入

### 1.3 Decoder 模型输出签名（实际模型）

**总输出数**: 25 个

```
Output[0]  name="logits"                             type=tensor(float)  shape=['batch', 'tgt_seq', 65001]
Output[1]  name="present.0.decoder.key"              type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[2]  name="present.0.decoder.value"            type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[3]  name="present.0.encoder.key"              type=tensor(float)  shape=['batch', 'Transposepresent.0.encoder.key_dim_1', 'present_seq', 'Transposepresent.0.encoder.key_dim_3']
Output[4]  name="present.0.encoder.value"            type=tensor(float)  shape=['batch', 'Transposepresent.0.encoder.value_dim_1', 'present_seq', 'Transposepresent.0.encoder.value_dim_3']
Output[5]  name="present.1.decoder.key"              type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[6]  name="present.1.decoder.value"            type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[7]  name="present.1.encoder.key"              type=tensor(float)  shape=['batch', 'Transposepresent.1.encoder.key_dim_1', 'present_seq', 'Transposepresent.1.encoder.key_dim_3']
Output[8]  name="present.1.encoder.value"            type=tensor(float)  shape=['batch', 'Transposepresent.1.encoder.value_dim_1', 'present_seq', 'Transposepresent.1.encoder.value_dim_3']
Output[9]  name="present.2.decoder.key"              type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[10] name="present.2.decoder.value"            type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[11] name="present.2.encoder.key"              type=tensor(float)  shape=['batch', 'Transposepresent.2.encoder.key_dim_1', 'present_seq', 'Transposepresent.2.encoder.key_dim_3']
Output[12] name="present.2.encoder.value"            type=tensor(float)  shape=['batch', 'Transposepresent.2.encoder.value_dim_1', 'present_seq', 'Transposepresent.2.encoder.value_dim_3']
Output[13] name="present.3.decoder.key"              type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[14] name="present.3.decoder.value"            type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[15] name="present.3.encoder.key"              type=tensor(float)  shape=['batch', 'Transposepresent.3.encoder.key_dim_1', 'present_seq', 'Transposepresent.3.encoder.key_dim_3']
Output[16] name="present.3.encoder.value"            type=tensor(float)  shape=['batch', 'Transposepresent.3.encoder.value_dim_1', 'present_seq', 'Transposepresent.3.encoder.value_dim_3']
Output[17] name="present.4.decoder.key"              type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[18] name="present.4.decoder.value"            type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[19] name="present.4.encoder.key"              type=tensor(float)  shape=['batch', 'Transposepresent.4.encoder.key_dim_1', 'present_seq', 'Transposepresent.4.encoder.key_dim_3']
Output[20] name="present.4.encoder.value"            type=tensor(float)  shape=['batch', 'Transposepresent.4.encoder.value_dim_1', 'present_seq', 'Transposepresent.4.encoder.value_dim_3']
Output[21] name="present.5.decoder.key"              type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[22] name="present.5.decoder.value"            type=tensor(float)  shape=['batch', 8, 'present_seq', 64]
Output[23] name="present.5.encoder.key"              type=tensor(float)  shape=['batch', 'Transposepresent.5.encoder.key_dim_1', 'present_seq', 'Transposepresent.5.encoder.key_dim_3']
Output[24] name="present.5.encoder.value"            type=tensor(float)  shape=['batch', 'Transposepresent.5.encoder.value_dim_1', 'present_seq', 'Transposepresent.5.encoder.value_dim_3']
```

**注意**: 
- 模型输出包含 encoder KV cache（`present.*.encoder.key/value`），但输入缺少对应的 `past_key_values.*.encoder.key/value`
- Encoder KV 输出的形状使用了复杂的动态维度名称，可能表示这些输出在模型内部计算，不需要作为输入传入

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

## 3. KV Cache 构建 Shape 日志（实际运行）

### 3.1 第一步（use_cache_branch=false）

**基础输入**:
```
[Input Construction] Basic inputs:
  - encoder_attention_mask: shape [1, 29]
  - decoder_input_ids: shape [1, 1]
  - encoder_hidden_states: shape [1, 29, 512]
  - use_cache_branch: false
```

**注意**: 由于日志输出被截断，基础输入的详细日志可能未完全显示。

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
[Decoder] Calling decoder_session.run() with 28 inputs...
```

### 3.2 KV Cache Shape 总结

| KV Cache 类型 | 层数 | 每层 KV 数 | Shape | 总数量 |
|--------------|------|-----------|-------|--------|
| Decoder KV | 6 | 2 (key, value) | [1, 8, 1, 64] | 12 个 ✅ |
| Encoder KV | 6 | 2 (key, value) | [1, 8, 29, 64] | 12 个 ❌ **缺失** |
| **总计** | 6 | 4 | - | **24 个** |

---

## 4. 问题分析

### 4.1 输入数量不匹配

| 项目 | 当前模型 | 代码期望 | 差异 |
|------|---------|---------|------|
| 基础输入 | 3 | 3 | ✅ 匹配 |
| Decoder KV | 12 (6 层 × 2) | 12 (6 层 × 2) | ✅ 匹配 |
| Encoder KV | 0 | 12 (6 层 × 2) | ❌ **缺失 12 个** |
| use_cache_branch | 1 | 1 | ✅ 匹配 |
| **总计** | **16** | **28** | ❌ **缺少 12 个输入** |

### 4.2 运行时错误

当代码尝试传递 28 个输入给只有 16 个输入的模型时：

```
Error: "Translation failed: Translation failed: failed to run decoder model: Failed to run inference on model: input name cannot be empty"
```

**错误原因**: ONNX Runtime 在尝试匹配输入时，发现某些输入没有对应的名称（因为模型中没有定义这些输入）。

### 4.3 根本原因

**导出脚本问题**: 虽然 `build_io_names` 函数定义了 encoder KV 的输入名称，但实际导出时：
1. Encoder KV 的 dummy 输入形状可能不正确（使用了 `past_seq` 而不是 `encoder_seq_len`）
2. PyTorch 的 ONNX 导出可能因为形状不匹配而忽略了这些输入
3. 或者导出脚本的 Wrapper 类没有正确接受 encoder KV 作为输入

---

## 5. 对比：工作模型（marian-en-zh）

### 5.1 工作模型的输入签名

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

### 5.2 关键差异

| 项目 | marian-en-zh (工作) | marian-zh-en IR 7 (当前) |
|------|---------------------|--------------------------|
| 总输入数 | 28 | 16 |
| Encoder KV | ✅ 12 个 | ❌ 0 个 |
| use_cache_branch | ✅ 1 个 | ✅ 1 个 |
| 状态 | ✅ 正常工作 | ❌ 无法使用 |

---

## 6. 修复状态

### 6.1 已修复的导出脚本

**文件**: `export_marian_decoder_ir9_fixed.py`

**修复内容**:
1. ✅ Encoder KV cache 形状从 `past_seq` 改为 `encoder_seq_len`
2. ✅ 动态轴设置区分 decoder KV 和 encoder KV

**修复位置**:
- 第 252-281 行: KV cache 形状构造
- 第 303-310 行: 动态轴设置

### 6.2 待执行操作

1. **重新导出模型**:
   ```bash
   python export_marian_decoder_ir9_fixed.py
   ```

2. **验证模型输入数量**:
   ```bash
   python scripts/get_decoder_model_signature.py
   # 应该显示: Total inputs: 28
   ```

3. **重新运行集成测试**:
   ```bash
   cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav
   ```

---

## 7. 总结

### 7.1 当前状态

- ❌ **模型输入不完整**: 只有 16 个输入，缺少 12 个 Encoder KV cache 输入
- ✅ **代码逻辑正确**: 正确构造了 28 个输入（包括 Encoder KV）
- ✅ **导出脚本已修复**: 修复了 Encoder KV 的形状问题
- ⏳ **待重新导出**: 需要重新运行导出脚本生成正确的模型

### 7.2 建议

1. **确认导出脚本修复是否正确**
2. **重新导出模型并验证输入数量**
3. **如果问题仍然存在，检查 PyTorch ONNX 导出是否支持这种输入结构**

---

**最后更新**: 2025-11-21  
**状态**: ⏳ **等待重新导出模型并验证**

