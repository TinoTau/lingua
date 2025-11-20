# KV Cache 说明和修复方�?

**鏈€鍚庢洿鏂?*: 2024-12-19

---


**������**: 2024-12-19

---

# KV Cache 说明和修复方�?

## 什么是 KV Cache�?

**KV Cache (Key-Value Cache)** �?Transformer 模型在增量解码（incremental decoding）时使用的优化技术�?

### 工作原理

1. **Attention 机制**�?
   - Transformer �?Attention 需要计�?Query、Key、Value
   - 对于每个 token，需要与之前所�?token �?Key �?Value 计算 attention

2. **问题**�?
   - 在增量解码中，每次生成新 token 时，如果重新计算所�?token �?Key �?Value，会非常耗时
   - 例如：生成第 10 �?token 时，�?9 �?token �?Key/Value 已经被计算过�?

3. **解决方案 - KV Cache**�?
   - 缓存之前计算过的 Key �?Value
   - 每次迭代只计算新 token �?Key �?Value
   - 将新�?Key/Value 追加到缓存中，供下次迭代使用

### 在我们的实现�?

```rust
// 第一次迭代：
// 输入: decoder_start_token_id
// 输出: logits + present KV cache (包含第一�?token �?K/V)

// 第二次迭代：
// 输入: �?token + past KV cache (第一次迭代的 present KV cache)
// 输出: logits + present KV cache (包含前两�?token �?K/V)

// 第三次迭代：
// 输入: �?token + past KV cache (第二次迭代的 present KV cache)
// 输出: logits + present KV cache (包含前三�?token �?K/V)
// ...
```

## 当前问题

### 问题描述

在提�?`present KV cache` 时，`ort` crate �?`try_extract_tensor` 方法会出现内存安全问题：

```
unsafe precondition(s) violated: slice::from_raw_parts requires the pointer to 
be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`
```

### 问题原因

1. **生命周期问题**�?
   - `try_extract_tensor` 返回�?slice 可能引用�?`outputs` 内部的数�?
   - 当我们在循环中多次提取时，可能会出现悬垂引用

2. **内存对齐问题**�?
   - ONNX Runtime 返回的数据可能没有正确对�?
   - 或者数据大小超过了 `isize::MAX`

3. **ort crate 版本问题**�?
   - 当前使用的是 `ort 2.0.0-rc.10`（候选版本）
   - 可能存在已知的内存安全问�?

## 修复方案

### 方案 1：升�?ort crate（推荐）

检查是否有更新的稳定版本：

```bash
cargo update ort
```

或者查�?ort crate �?GitHub issues，看是否有相关的修复�?

### 方案 2：使用不同的 API

尝试使用 `ort::Value` 的其他方法提取数据，例如�?

```rust
// 尝试使用 to_owned() 或其他方�?
let value_owned = value.to_owned();
```

### 方案 3：一次性提取所有数�?

在循环外先提取所有需要的数据，避免多次访�?`outputs`�?

```rust
// 先提取所�?KV cache 数据�?Vec
let mut all_kv_data = Vec::new();
for layer_idx in 0..6 {
    // 提取并立即复制数�?
    let (shape, slice) = outputs[format!("present.{}.decoder.key", layer_idx)]
        .try_extract_tensor::<f32>()?;
    all_kv_data.push((shape, slice.to_vec()));
}
// 然后再构�?Array4
```

### 方案 4：使�?unsafe 代码（不推荐�?

如果其他方案都不可行，可以考虑使用 unsafe 代码手动管理内存，但这会降低代码安全性�?

### 方案 5：暂时跳�?KV cache 更新（当前方案）

目前我们暂时跳过�?KV cache 更新，这会导致：
- �?基本翻译流程可以运行
- �?翻译结果不准确（每次迭代使用相同�?KV cache�?
- �?性能较差（没有利�?KV cache 优化�?

## 当前状�?

- �?Encoder 推理：正常工�?
- �?Decoder 推理：正常工�?
- �?Token 选择：正常工�?
- �?KV Cache 更新：暂时跳过（存在内存安全问题�?

## 下一�?

1. 检�?`ort` crate 是否有更新版�?
2. 查看 `ort` crate 的文档和示例，寻找正确的 tensor 提取方式
3. 如果问题持续，考虑�?`ort` crate 提交 issue
4. 或者考虑使用其他 ONNX Runtime �?Rust 绑定（如 `onnxruntime-rs`�?

## 参�?

- [Transformer KV Cache 原理](https://lilianweng.github.io/posts/2023-01-10-inference-optimization/)
- [ort crate 文档](https://docs.rs/ort/)
- [ONNX Runtime 文档](https://onnxruntime.ai/docs/)

