# Marian Decoder 导出脚本修复

**日期**: 2025-11-21  
**问题**: Encoder KV cache 输入形状错误  
**状态**: ✅ **已修复**

---

## 问题描述

重新导出的模型只有 16 个输入（缺少 encoder KV cache），导致运行时错误：
```
Error: "input name cannot be empty"
```

## 根本原因

在 `export_marian_decoder_ir9_fixed.py` 中，所有 KV cache（包括 encoder KV）都使用了 `past_seq=1` 作为第三个维度：

```python
# ❌ 错误：所有 KV 都使用 past_seq
for _kind in range(4):  # self_k, self_v, cross_k, cross_v
    past_list.append(
        torch.zeros(
            (batch_size, num_heads, past_seq, head_dim),  # ❌ encoder KV 也用了 past_seq
            dtype=encoder_hidden_states.dtype,
        )
    )
```

但 encoder KV 应该使用 `encoder_seq_len`（即 `src_seq`）作为第三个维度。

## 修复方案

### 1. 修正 KV cache 形状构造

```python
# ✅ 正确：decoder KV 使用 past_seq，encoder KV 使用 encoder_seq_len
for _layer in range(num_kv_layers):
    # self_k (decoder key): [batch, num_heads, past_seq, head_dim]
    past_list.append(
        torch.zeros(
            (batch_size, num_heads, past_seq, head_dim),
            dtype=encoder_hidden_states.dtype,
        )
    )
    # self_v (decoder value): [batch, num_heads, past_seq, head_dim]
    past_list.append(
        torch.zeros(
            (batch_size, num_heads, past_seq, head_dim),
            dtype=encoder_hidden_states.dtype,
        )
    )
    # cross_k (encoder key): [batch, num_heads, encoder_seq_len, head_dim]
    past_list.append(
        torch.zeros(
            (batch_size, num_heads, src_seq, head_dim),  # ✅ 使用 src_seq
            dtype=encoder_hidden_states.dtype,
        )
    )
    # cross_v (encoder value): [batch, num_heads, encoder_seq_len, head_dim]
    past_list.append(
        torch.zeros(
            (batch_size, num_heads, src_seq, head_dim),  # ✅ 使用 src_seq
            dtype=encoder_hidden_states.dtype,
        )
    )
```

### 2. 修正动态轴设置

```python
# ✅ 正确：decoder KV 和 encoder KV 使用不同的动态轴
for name in input_names:
    if name.startswith("past_"):
        if "decoder" in name:
            # Decoder KV: 第三个维度是 past_seq
            dynamic_axes[name] = {0: "batch", 2: "past_seq"}
        elif "encoder" in name:
            # Encoder KV: 第三个维度是 src_seq (encoder_seq_len)
            dynamic_axes[name] = {0: "batch", 2: "src_seq"}
```

## 验证

修复后，重新导出模型应该：
1. ✅ 有 28 个输入（3 基础 + 12 decoder KV + 12 encoder KV + 1 use_cache_branch）
2. ✅ Encoder KV 的形状为 `[batch, num_heads, encoder_seq_len, head_dim]`
3. ✅ Decoder KV 的形状为 `[batch, num_heads, past_seq, head_dim]`

## 参考

工作模型 `marian-en-zh` 的导出脚本（`scripts/export_marian_onnx.py`）正确使用了不同的形状：

```python
past_key_values.append((
    torch.zeros(batch_size, num_heads, past_decoder_seq_len, head_dim),  # decoder key
    torch.zeros(batch_size, num_heads, past_decoder_seq_len, head_dim),  # decoder value
    torch.zeros(batch_size, num_heads, encoder_seq_len, head_dim),      # encoder key ✅
    torch.zeros(batch_size, num_heads, encoder_seq_len, head_dim),      # encoder value ✅
))
```

---

**最后更新**: 2025-11-21

