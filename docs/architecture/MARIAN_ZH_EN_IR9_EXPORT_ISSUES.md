# Marian zh-en IR 9 导出方案问题报告

**日期**: 2025-11-21  
**状态**: 🔴 发现严重问题，需要修复

---

## 问题概述

`export_marian_decoder_ir9.py` 脚本导出的 Decoder 模型**缺少 KV cache 支持**，无法与现有代码兼容。

---

## 详细问题

### 1. Decoder 输入不匹配 ❌

**脚本导出** (`export_marian_decoder_ir9.py:100`):
```python
input_names=["decoder_input_ids", "encoder_hidden_states", "encoder_attention_mask"]
# 只有 3 个输入
```

**代码期望** (`decoder.rs:161-208`):
```rust
// 输入顺序：encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.*, use_cache_branch
// 总共 28 个输入：
//   1. encoder_attention_mask
//   2. input_ids
//   3. encoder_hidden_states
//   4-27. past_key_values.* (每层 4 个：dec_k, dec_v, enc_k, enc_v，共 6 层 = 24 个)
//   28. use_cache_branch
```

**问题**:
- ❌ 缺少 24 个 KV cache 输入
- ❌ 缺少 `use_cache_branch` 输入
- ❌ 输入顺序不对

### 2. Decoder 输出不匹配 ❌

**脚本导出** (`export_marian_decoder_ir9.py:101`):
```python
output_names=["logits"]
# 只有 1 个输出
```

**代码期望** (`decoder.rs:217-244`):
```rust
// 输出：
//   1. logits
//   2-25. present.* (每层 4 个：dec_k, dec_v, enc_k, enc_v，共 6 层 = 24 个)
// 总共 25 个输出
```

**问题**:
- ❌ 缺少 24 个 KV cache 输出

### 3. 现有模型结构参考

**`marian-en-zh` 模型**（正常工作）:
- ✅ 28 个输入（包含完整的 KV cache）
- ✅ 25 个输出（包含完整的 KV cache）
- ✅ 支持增量解码

**检查命令**:
```bash
python -c "import onnxruntime as ort; sess = ort.InferenceSession('core/engine/models/nmt/marian-en-zh/model.onnx'); print('Inputs:', len(sess.get_inputs())); print('Outputs:', len(sess.get_outputs()))"
```

**结果**: 28 个输入，25 个输出

---

## 修复方案

### 方案 1: 修改 `export_marian_decoder_ir9.py` ⭐ 推荐

**参考**: `scripts/export_marian_onnx.py` 的 `export_decoder_with_past` 函数

**需要修改**:

1. **添加 KV cache 输入**:
   ```python
   # 为每层创建 past_key_values
   past_key_values = []
   for _ in range(num_layers):  # 6 层
       past_key_values.append((
           torch.zeros(batch_size, num_heads, past_decoder_seq_len, head_dim),  # decoder key
           torch.zeros(batch_size, num_heads, past_decoder_seq_len, head_dim),  # decoder value
           torch.zeros(batch_size, num_heads, encoder_seq_len, head_dim),  # encoder key
           torch.zeros(batch_size, num_heads, encoder_seq_len, head_dim),  # encoder value
       ))
   ```

2. **添加 use_cache_branch 输入**:
   ```python
   dummy_use_cache = torch.tensor([True], dtype=torch.bool)
   ```

3. **修正输入顺序**:
   ```python
   inputs = [encoder_attention_mask, decoder_input_ids, encoder_hidden_states]
   for layer_kv in past_key_values:
       inputs.extend(layer_kv)
   inputs.append(dummy_use_cache)
   ```

4. **添加输入名称**:
   ```python
   input_names = ["encoder_attention_mask", "input_ids", "encoder_hidden_states"]
   for i in range(num_layers):
       input_names.extend([
           f"past_key_values.{i}.decoder.key",
           f"past_key_values.{i}.decoder.value",
           f"past_key_values.{i}.encoder.key",
           f"past_key_values.{i}.encoder.value",
       ])
   input_names.append("use_cache_branch")
   ```

5. **添加输出名称**:
   ```python
   output_names = ["logits"]
   for i in range(num_layers):
       output_names.extend([
           f"present.{i}.decoder.key",
           f"present.{i}.decoder.value",
           f"present.{i}.encoder.key",
           f"present.{i}.encoder.value",
       ])
   ```

6. **修改 Wrapper 类**:
   - 需要处理 KV cache 输入
   - 需要返回 KV cache 输出
   - 参考 `scripts/export_marian_onnx.py` 的 `DecoderWrapper`

7. **使用 opset_version=12**:
   ```python
   opset_version=12,  # 而不是 14
   ```

### 方案 2: 修改现有脚本

**修改 `scripts/export_marian_onnx.py`**:
- 将 `opset_version=14` 改为 `opset_version=12`
- 在 Python 3.10 + PyTorch 1.13.1 环境中运行

**优点**:
- ✅ 脚本已经支持 KV cache
- ✅ 只需要修改 opset 版本

**缺点**:
- ⚠️ 需要确保在旧版本 PyTorch 环境中运行

---

## 修复后的验证

### 1. 检查模型结构

```bash
python -c "
import onnxruntime as ort
sess = ort.InferenceSession('core/engine/models/nmt/marian-zh-en/model.onnx')
print('Inputs:', len(sess.get_inputs()))
print('Outputs:', len(sess.get_outputs()))
print('Input names:', [i.name for i in sess.get_inputs()])
print('Output names:', [o.name for o in sess.get_outputs()])
"
```

**期望结果**:
- 28 个输入
- 25 个输出
- 输入名称与代码期望匹配

### 2. 检查 IR 版本

```bash
python -c "import onnx; m = onnx.load('core/engine/models/nmt/marian-zh-en/model.onnx'); print(f'IR: {m.ir_version}, Opset: {m.opset_import[0].version}')"
```

**期望结果**:
- IR ≤ 9
- Opset = 12

### 3. 测试加载

```bash
cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav
```

---

## 总结

### 当前状态

- ✅ Encoder 导出脚本：正确
- ❌ Decoder 导出脚本：**缺少 KV cache 支持，无法使用**

### 必须修复

1. **添加 KV cache 输入**（24 个）
2. **添加 use_cache_branch 输入**（1 个）
3. **添加 KV cache 输出**（24 个）
4. **修正输入顺序**
5. **使用 opset_version=12**

### 推荐方案

**修改 `export_marian_decoder_ir9.py`**，参考 `scripts/export_marian_onnx.py` 的 `export_decoder_with_past` 函数，但使用 `opset_version=12`。

---

**最后更新**: 2025-11-21  
**状态**: 🔴 Decoder 导出脚本需要修复才能使用

