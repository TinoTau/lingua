# KV Cache 方案 1 失败分析报告

## 📊 失败时间
2024-12-19

## ❌ 失败信息

### 错误详情

```
Failed to run decoder model: Failed to run inference on model: Non-zero status code returned while running If node. 
Name:'optimum::if' 
Status Message: Non-zero status code returned while running Reshape node. 
Name:'/model/decoder/layers.0/encoder_attn/Reshape_4' 
Status Message: C:\__w\1\s\onnxruntime\onnxruntime\core\providers\cpu\tensor\reshape_helper.h:30 
onnxruntime::ReshapeHelper::ReshapeHelper i < input_shape.NumDimensions() was false. 
The dimension with value zero exceeds the dimension size of the input tensor.
```

### 失败位置

- **失败步骤**: Step 2（第三步，索引为 2）
- **成功步骤**: 
  - ✅ Step 0（第一步，`use_cache_branch=false`）- 成功提取 KV cache
  - ✅ Step 1（第二步，`use_cache_branch=true`）- 成功使用 KV cache
  - ❌ Step 2（第三步，`use_cache_branch=true`）- **失败**

---

## 🔍 问题分析

### 关键发现：Python 测试成功，但发现了关键问题

**Python 测试输出（Step 1，`use_cache_branch=True`）**：

```
present.0.decoder.key: shape=(1, 8, 2, 64)  ✅ 正常累积
present.0.decoder.value: shape=(1, 8, 2, 64)  ✅ 正常累积
present.0.encoder.key: shape=(0, 8, 1, 64)  ⚠️ **第一个维度是 0！**
present.0.encoder.value: shape=(0, 8, 1, 64)  ⚠️ **第一个维度是 0！**
```

### 根本原因

**当 `use_cache_branch=True` 时，模型输出的 `present.*.encoder.*` 的第一个维度是 0！**

这意味着：
1. **Decoder KV cache 正常累积**：
   - Step 0: `present.*.decoder.*` = `(1, 8, 1, 64)` ✅
   - Step 1: `present.*.decoder.*` = `(1, 8, 2, 64)` ✅
   - Step 2: `present.*.decoder.*` = `(1, 8, 3, 64)` ✅

2. **Encoder KV cache 在 `use_cache_branch=True` 时变成空**：
   - Step 0: `present.*.encoder.*` = `(1, 8, 4, 64)` ✅（正常）
   - Step 1: `present.*.encoder.*` = `(0, 8, 1, 64)` ❌（**第一个维度是 0**）
   - Step 2: 如果使用 Step 1 的 `present.*.encoder.*` 作为 `past_key_values.*.encoder.*`，就会导致 Reshape 错误

3. **为什么会出现 Reshape 错误**：
   - Step 2 时，Rust 代码使用 Step 1 的 `present.*.encoder.*`（形状为 `(0, 8, 1, 64)`）作为 `past_key_values.*.encoder.*` 输入
   - 但模型期望的 `past_key_values.*.encoder.*` 形状应该是 `(1, 8, 4, 64)`（与 encoder 序列长度一致）
   - 当模型尝试 Reshape 时，发现第一个维度是 0，导致错误

---

## 🔧 修复方案

### 方案：Encoder KV Cache 应该保持不变

**关键理解**：
- **Decoder KV cache**：需要累积（每次步骤都更新）
- **Encoder KV cache**：**不需要累积，应该保持不变**（只在第一次创建时使用）

**修复步骤**：

1. **修改 `decoder_step` 中的 KV cache 提取逻辑**：
   ```rust
   if state.use_cache_branch {
       // 正常模式（第二步及以后）：提取 KV cache 供下一步使用
       let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
       for _layer in 0..Self::NUM_LAYERS {
           let dec_k = iter.next().expect("missing present.*.decoder.key");
           let dec_v = iter.next().expect("missing present.*.decoder.value");
           let enc_k = iter.next().expect("missing present.*.encoder.key");
           let enc_v = iter.next().expect("missing present.*.encoder.value");
           
           // ⚠️ 关键修复：当 use_cache_branch=true 时，present.*.encoder.* 的第一个维度是 0
           // 我们不能使用这些空的 encoder KV cache，应该保持使用初始的 encoder KV cache
           // 解决方案：从 state.kv_cache 中获取 encoder KV cache（保持不变）
           let [old_dec_k, old_dec_v, old_enc_k, old_enc_v] = &state.kv_cache.as_ref().expect("kv_cache should exist")[_layer];
           
           // 只更新 decoder KV cache，保持 encoder KV cache 不变
           next_kv.push([dec_k, dec_v, old_enc_k.clone(), old_enc_v.clone()]);
       }
       state.kv_cache = Some(next_kv);
       state.use_cache_branch = true;  // 保持启用状态
   } else {
       // 第一步（use_cache_branch=false）：提取 KV cache 供下一步使用
       // 这一步的 present.*.encoder.* 是正常的，可以全部提取
       let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
       for _layer in 0..Self::NUM_LAYERS {
           let dec_k = iter.next().expect("missing present.*.decoder.key");
           let dec_v = iter.next().expect("missing present.*.decoder.value");
           let enc_k = iter.next().expect("missing present.*.encoder.key");
           let enc_v = iter.next().expect("missing present.*.encoder.value");
           next_kv.push([dec_k, dec_v, enc_k, enc_v]);
       }
       state.kv_cache = Some(next_kv);
       state.use_cache_branch = true;  // 下一步启用 KV cache
   }
   ```

2. **或者，更简单的方案：在 `use_cache_branch=true` 时跳过 encoder KV cache 的提取**：
   ```rust
   if state.use_cache_branch {
       // 正常模式（第二步及以后）：只提取 decoder KV cache
       let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
       for _layer in 0..Self::NUM_LAYERS {
           let dec_k = iter.next().expect("missing present.*.decoder.key");
           let dec_v = iter.next().expect("missing present.*.decoder.value");
           iter.next(); // 跳过 present.*.encoder.key（形状为 (0, 8, 1, 64)，不可用）
           iter.next(); // 跳过 present.*.encoder.value（形状为 (0, 8, 1, 64)，不可用）
           
           // 从旧的 KV cache 中获取 encoder KV cache（保持不变）
           let [old_dec_k, old_dec_v, old_enc_k, old_enc_v] = &state.kv_cache.as_ref().expect("kv_cache should exist")[_layer];
           
           // 只更新 decoder KV cache，保持 encoder KV cache 不变
           next_kv.push([dec_k, dec_v, old_enc_k.clone(), old_enc_v.clone()]);
       }
       state.kv_cache = Some(next_kv);
       state.use_cache_branch = true;  // 保持启用状态
   } else {
       // 第一步（use_cache_branch=false）：提取所有 KV cache
       let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
       for _layer in 0..Self::NUM_LAYERS {
           let dec_k = iter.next().expect("missing present.*.decoder.key");
           let dec_v = iter.next().expect("missing present.*.decoder.value");
           let enc_k = iter.next().expect("missing present.*.encoder.key");
           let enc_v = iter.next().expect("missing present.*.encoder.value");
           next_kv.push([dec_k, dec_v, enc_k, enc_v]);
       }
       state.kv_cache = Some(next_kv);
       state.use_cache_branch = true;  // 下一步启用 KV cache
   }
   ```

---

## 📋 修复实施

### 步骤 1：修改 `decoder_step` 方法

在 `core/engine/src/nmt_incremental/mod.rs` 的 `decoder_step` 方法中，修改 KV cache 提取逻辑：

```rust
// KV cache：处理 present.* 输出
if state.use_cache_branch {
    // 正常模式（第二步及以后）：只提取 decoder KV cache，保持 encoder KV cache 不变
    let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
    for layer_idx in 0..Self::NUM_LAYERS {
        let dec_k = iter.next().expect("missing present.*.decoder.key");
        let dec_v = iter.next().expect("missing present.*.decoder.value");
        iter.next(); // 跳过 present.*.encoder.key（use_cache_branch=true 时形状为 (0, 8, 1, 64)，不可用）
        iter.next(); // 跳过 present.*.encoder.value（use_cache_branch=true 时形状为 (0, 8, 1, 64)，不可用）
        
        // 从旧的 KV cache 中获取 encoder KV cache（保持不变）
        let [old_dec_k, old_dec_v, old_enc_k, old_enc_v] = &state.kv_cache.as_ref().expect("kv_cache should exist")[layer_idx];
        
        // 只更新 decoder KV cache，保持 encoder KV cache 不变
        next_kv.push([dec_k, dec_v, old_enc_k.clone(), old_enc_v.clone()]);
    }
    state.kv_cache = Some(next_kv);
    state.use_cache_branch = true;  // 保持启用状态
} else {
    // 第一步（use_cache_branch=false）：提取所有 KV cache
    let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
    for _layer in 0..Self::NUM_LAYERS {
        let dec_k = iter.next().expect("missing present.*.decoder.key");
        let dec_v = iter.next().expect("missing present.*.decoder.value");
        let enc_k = iter.next().expect("missing present.*.encoder.key");
        let enc_v = iter.next().expect("missing present.*.encoder.value");
        next_kv.push([dec_k, dec_v, enc_k, enc_v]);
    }
    state.kv_cache = Some(next_kv);
    state.use_cache_branch = true;  // 下一步启用 KV cache
}
```

### 步骤 2：测试修复

运行测试：
```bash
cargo test --test nmt_quick_test -- --nocapture
```

---

## 📝 总结

### 根本原因

**当 `use_cache_branch=True` 时，模型输出的 `present.*.encoder.*` 的第一个维度是 0，不能用作下一步的 `past_key_values.*.encoder.*` 输入。**

### 解决方案

**Encoder KV cache 应该保持不变，只在第一次（`use_cache_branch=false`）时提取，后续步骤（`use_cache_branch=true`）只更新 decoder KV cache。**

### 修复状态

- ✅ 问题已定位
- ⏳ 等待实施修复
- ⏳ 等待测试验证

---

**最后更新**: 2024-12-19
