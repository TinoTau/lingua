# KV Cache 说明和修复方案

## 什么是 KV Cache？

**KV Cache (Key-Value Cache)** 是 Transformer 模型在增量解码（incremental decoding）时使用的优化技术。

### 工作原理

1. **Attention 机制**：
   - Transformer 的 Attention 需要计算 Query、Key、Value
   - 对于每个 token，需要与之前所有 token 的 Key 和 Value 计算 attention

2. **问题**：
   - 在增量解码中，每次生成新 token 时，如果重新计算所有 token 的 Key 和 Value，会非常耗时
   - 例如：生成第 10 个 token 时，前 9 个 token 的 Key/Value 已经被计算过了

3. **解决方案 - KV Cache**：
   - 缓存之前计算过的 Key 和 Value
   - 每次迭代只计算新 token 的 Key 和 Value
   - 将新的 Key/Value 追加到缓存中，供下次迭代使用

### 在我们的实现中

```rust
// 第一次迭代：
// 输入: decoder_start_token_id
// 输出: logits + present KV cache (包含第一个 token 的 K/V)

// 第二次迭代：
// 输入: 新 token + past KV cache (第一次迭代的 present KV cache)
// 输出: logits + present KV cache (包含前两个 token 的 K/V)

// 第三次迭代：
// 输入: 新 token + past KV cache (第二次迭代的 present KV cache)
// 输出: logits + present KV cache (包含前三个 token 的 K/V)
// ...
```

## 当前问题

### 问题描述

在提取 `present KV cache` 时，`ort` crate 的 `try_extract_tensor` 方法会出现内存安全问题：

```
unsafe precondition(s) violated: slice::from_raw_parts requires the pointer to 
be aligned and non-null, and the total size of the slice not to exceed `isize::MAX`
```

### 问题原因

1. **生命周期问题**：
   - `try_extract_tensor` 返回的 slice 可能引用了 `outputs` 内部的数据
   - 当我们在循环中多次提取时，可能会出现悬垂引用

2. **内存对齐问题**：
   - ONNX Runtime 返回的数据可能没有正确对齐
   - 或者数据大小超过了 `isize::MAX`

3. **ort crate 版本问题**：
   - 当前使用的是 `ort 2.0.0-rc.10`（候选版本）
   - 可能存在已知的内存安全问题

## 修复方案

### 方案 1：升级 ort crate（推荐）

检查是否有更新的稳定版本：

```bash
cargo update ort
```

或者查看 ort crate 的 GitHub issues，看是否有相关的修复。

### 方案 2：使用不同的 API

尝试使用 `ort::Value` 的其他方法提取数据，例如：

```rust
// 尝试使用 to_owned() 或其他方法
let value_owned = value.to_owned();
```

### 方案 3：一次性提取所有数据

在循环外先提取所有需要的数据，避免多次访问 `outputs`：

```rust
// 先提取所有 KV cache 数据到 Vec
let mut all_kv_data = Vec::new();
for layer_idx in 0..6 {
    // 提取并立即复制数据
    let (shape, slice) = outputs[format!("present.{}.decoder.key", layer_idx)]
        .try_extract_tensor::<f32>()?;
    all_kv_data.push((shape, slice.to_vec()));
}
// 然后再构建 Array4
```

### 方案 4：使用 unsafe 代码（不推荐）

如果其他方案都不可行，可以考虑使用 unsafe 代码手动管理内存，但这会降低代码安全性。

### 方案 5：暂时跳过 KV cache 更新（当前方案）

目前我们暂时跳过了 KV cache 更新，这会导致：
- ✅ 基本翻译流程可以运行
- ❌ 翻译结果不准确（每次迭代使用相同的 KV cache）
- ❌ 性能较差（没有利用 KV cache 优化）

## 当前状态

- ✅ Encoder 推理：正常工作
- ✅ Decoder 推理：正常工作
- ✅ Token 选择：正常工作
- ❌ KV Cache 更新：暂时跳过（存在内存安全问题）

## 下一步

1. 检查 `ort` crate 是否有更新版本
2. 查看 `ort` crate 的文档和示例，寻找正确的 tensor 提取方式
3. 如果问题持续，考虑向 `ort` crate 提交 issue
4. 或者考虑使用其他 ONNX Runtime 的 Rust 绑定（如 `onnxruntime-rs`）

## 参考

- [Transformer KV Cache 原理](https://lilianweng.github.io/posts/2023-01-10-inference-optimization/)
- [ort crate 文档](https://docs.rs/ort/)
- [ONNX Runtime 文档](https://onnxruntime.ai/docs/)

