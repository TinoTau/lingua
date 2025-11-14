# Recommended Solution for Marian NMT Decoder KV Cache Handling  
(Using No-KV Step0 + KV StepN Approach)

## Overview
This guide provides the **recommended solution** to fix the KV cache rank mismatch in your current Marian NMT ONNX decoder implementation.  
It is based on your GitHub code state, ONNX Runtime 1.16.3, and the actual shape expectations of Marian models.

---

## 1. Problem Summary

### ❌ What went wrong

Your manually-created `empty_kv` becomes **rank = 1** after passing through:

```rust
Value::from_array(allocator, arr)
```

The model expects a 4D tensor:

```text
[batch, num_heads, seq_len, head_dim]   // 4D Tensor
```

But your empty KV arrives as:

```text
[f32; N]   // 1D Tensor
```

This causes shape mismatch and an invalid KV state when the decoder tries to use it.

### ✅ Current status of your code

From your latest commits and test logs:

- Encoder and decoder ONNX models **load successfully** (file mode, `Session`)
- Encoder forward pass is **correct**, output shape `[1, 3, 512]`
- ONNX Runtime crate has been updated to **ort = 1.16.3**
- You have already:
  - Switched to **file mode** (`with_model_from_file`)
  - Fixed `OrtOwnedTensor` usage:
    - `try_extract::<OrtOwnedTensor<_, IxDyn>>()`
    - `.view() + into_dimensionality() + to_owned()`
  - Fixed `session.run()` inputs to use the proper `inputs!` macro / `Vec<(String, Value)>`
  - Fixed KV cache **lifetime** issues (no more unsafe slicing)
  - Fixed `use_cache_branch` type (from `bool` to `f32`)

The **only remaining issue** is:

> KV cache shape (rank) is wrong when manually created as empty.

---

## 2. Recommended Approach (Best Solution)

### ✔ Core idea

> **Do NOT manually construct any empty KV tensors.**  
> Let the ONNX decoder graph **initialize KV cache internally** on the first step, then reuse its outputs for subsequent steps.

Your Marian decoder has inputs like:

- `encoder_attention_mask`
- `input_ids`
- `encoder_hidden_states`
- `past_key_values.*` (lots of them)
- `use_cache_branch`

This usually means the model was exported with **two branches**:

1. **No-cache branch (step 0)**  
   - `use_cache_branch = 0.0`
   - `past_key_values.*` are ignored or treated as absent
   - Model internally initializes KV cache from scratch
   - Outputs:
     - `logits`
     - `present.0.decoder.key`
     - `present.0.decoder.value`
     - `present.0.encoder.key`
     - `present.0.encoder.value`
     - … for all layers

2. **Cache branch (step N ≥ 1)**  
   - `use_cache_branch = 1.0`
   - `past_key_values.*` must be provided from previous step’s `present.*`
   - Model uses KV cache for efficient incremental decoding

### ✅ What this means for you

- **Step 0**:  
  - Do **not** provide any KV cache.  
  - Only provide `input_ids`, `encoder_hidden_states`, `encoder_attention_mask`, `use_cache_branch = 0.0`.  
  - Use decoder outputs `present.*` as the first KV cache.

- **Step N (N ≥ 1)**:  
  - Use `use_cache_branch = 1.0`.  
  - Feed previous step’s `present.*` tensors back as `past_key_values.*`.  
  - Again, do **not** inspect / reshape KV; treat them as black-box `Value`s.

This way:

- You never construct KV tensors yourself.
- KV shape is always correct (4D) because it’s produced by the model.
- You don’t need to use `try_extract_tensor` on KV outputs at all.

---

## 3. Step 0 Decoder Implementation (No KV Inputs)

Below is a template function for **step 0** (first token) of decoder:

```rust
use anyhow::Result;
use ndarray::{Array2, Array3, Ix3, IxDyn};
use ort::tensor::OrtOwnedTensor;
use ort::value::Value;
use ort::session::Session;

pub fn decoder_first_step(
    session: &mut Session,
    input_ids: Array2<i64>,
    encoder_hidden: Value,
    encoder_attention_mask: Value,
) -> Result<(Array3<f32>, Vec<Value>)> {
    // use_cache_branch = 0.0 for first step
    let use_cache_branch = ndarray::arr0(0.0f32);

    let inputs = ort::inputs![
        "input_ids"              => Value::from_array(session.allocator(), &input_ids.into_dyn())?,
        "encoder_hidden_states"  => encoder_hidden,
        "encoder_attention_mask" => encoder_attention_mask,
        "use_cache_branch"       => Value::from_array(session.allocator(), &use_cache_branch)?,
    ];

    let outputs = session.run(inputs)?;

    // 1) Extract logits as [batch, seq, vocab]
    let logits_tensor: OrtOwnedTensor<f32, IxDyn> =
        outputs["logits"].try_extract()?;
    let logits = logits_tensor
        .view()
        .to_owned()
        .into_dimensionality::<Ix3>()?;

    // 2) Collect KV cache from present.* (black-box Values)
    let mut kv_cache = Vec::new();

    // adjust layer count if needed
    let num_layers = 6usize;
    for layer in 0..num_layers {
        for kind in &["decoder.key", "decoder.value", "encoder.key", "encoder.value"] {
            let name = format!("present.{layer}.{kind}");
            let val = outputs[&name].unwrap_value();
            kv_cache.push(val);
        }
    }

    Ok((logits, kv_cache))
}
```

### Notes

- `encoder_hidden` 和 `encoder_attention_mask` 建议在 encoder 阶段就转换为 `Value`，这里直接复用。
- `num_layers = 6` 来自你模型的结构（根据打印的 IO 名字和维度推断）。
- `kv_cache` 是一个 `Vec<Value>`，你不关心里面的实际张量类型，只需要按相同顺序重新绑定回去即可。

---

## 4. Step N Decoder Implementation (With KV Inputs)

下面是 **后续步骤（N ≥ 1）** 的解码函数模板：

```rust
use anyhow::Result;
use ndarray::{Array2, Array3, Ix3, IxDyn};
use ort::tensor::OrtOwnedTensor;
use ort::value::Value;
use ort::session::{Session, SessionInputValue};

pub fn decoder_next_step(
    session: &mut Session,
    input_ids: Array2<i64>,
    encoder_hidden: Value,
    encoder_attention_mask: Value,
    prev_kv: Vec<Value>,
) -> Result<(Array3<f32>, Vec<Value>)> {
    let use_cache_branch = ndarray::arr0(1.0f32);

    let mut inputs = ort::inputs![
        "input_ids"              => Value::from_array(session.allocator(), &input_ids.into_dyn())?,
        "encoder_hidden_states"  => encoder_hidden,
        "encoder_attention_mask" => encoder_attention_mask,
        "use_cache_branch"       => Value::from_array(session.allocator(), &use_cache_branch)?,
    ];

    // Attach KV cache as past_key_values.* inputs
    let mut idx = 0usize;
    let num_layers = 6usize;

    for layer in 0..num_layers {
        let dec_k = prev_kv[idx].clone();
        let dec_v = prev_kv[idx + 1].clone();
        let enc_k = prev_kv[idx + 2].clone();
        let enc_v = prev_kv[idx + 3].clone();
        idx += 4;

        inputs.push((
            format!("past_key_values.{layer}.decoder.key").into(),
            SessionInputValue::Owned(dec_k),
        ));
        inputs.push((
            format!("past_key_values.{layer}.decoder.value").into(),
            SessionInputValue::Owned(dec_v),
        ));
        inputs.push((
            format!("past_key_values.{layer}.encoder.key").into(),
            SessionInputValue::Owned(enc_k),
        ));
        inputs.push((
            format!("past_key_values.{layer}.encoder.value").into(),
            SessionInputValue::Owned(enc_v),
        ));
    }

    let outputs = session.run(inputs)?;

    // logits
    let logits_tensor: OrtOwnedTensor<f32, IxDyn> =
        outputs["logits"].try_extract()?;
    let logits = logits_tensor
        .view()
        .to_owned()
        .into_dimensionality::<Ix3>()?;

    // Collect next KV cache
    let mut new_kv = Vec::new();
    for layer in 0..num_layers {
        for kind in &["decoder.key", "decoder.value", "encoder.key", "encoder.value"] {
            let name = format!("present.{layer}.{kind}");
            new_kv.push(outputs[&name].unwrap_value());
        }
    }

    Ok((logits, new_kv))
}
```

### Notes

- `prev_kv` 的顺序必须与 step0 收集 KV 的顺序一致。
- 你无需关心 `prev_kv` 内部的真实形状，模型会自己保证维度正确。
- 你只对 `logits` 使用 `OrtOwnedTensor` + `into_dimensionality::<Ix3>()`，  
  这部分你已经在 `nmt_onnx_decoder_step` 测试中验证过是安全的。

---

## 5. Why This Solves KV Rank Mismatch

### Root Cause（旧实现）

- `empty_kv` 是通过 `Vec<f32>` 或 1D `Array1<f32>` 构造的；
- 然后送入 `Value::from_array(allocator, &arr1)`；
- 结果是 **1维 tensor**，而不是 `[batch, heads, seq_len, dim]` 这种 4维。

### New Approach（推荐方案）

- 不再手工构造任何 KV tensor；
- **所有 KV** 都由 ONNX decoder 模型的 `present.*` 输出生成；
- **所有 KV 只以 `Value` 的形式在 Rust 层流转**：
  - 作为 `SessionInputValue::Owned(...)` 传入；
  - 作为 `outputs["present.*"].unwrap_value()` 收集；
- 模型内部会确保这些 KV 的 shape 永远是合法的 4D：
  ```text
  [batch=1, num_heads=8, seq_len=t, head_dim=64]
  ```
- 你也不需要再对 KV 使用 `try_extract_tensor` 或自己 reshape 数据。

因此：

- ❌ 不再有 1D rank mismatch 问题；
- ❌ 不再有因 KV 缓冲区对齐导致的 unsafe memory 问题；
- ✅ 解码逻辑与 HuggingFace / Marian 官方实现的模式保持一致。

---

## 6. How to Integrate into Your Project

### 6.1 在 `MarianNmtOnnx` 中维护状态

可以在 `MarianNmtOnnx` 里加一个简单的状态结构：

```rust
pub struct MarianDecoderState {
    pub kv_cache: Option<Vec<Value>>,
}

pub struct MarianNmtOnnx {
    encoder_session: Mutex<Session>,
    decoder_session: Mutex<Session>,
    decoder_state: Mutex<MarianDecoderState>,
    // tokenizer, config, etc...
}
```

### 6.2 在 `translate_incremental` 中使用 Step0 + StepN

伪代码示例：

```rust
impl MarianNmtOnnx {
    pub fn decode_step(
        &self,
        input_ids: Array2<i64>,
        encoder_hidden: Value,
        encoder_attention_mask: Value,
    ) -> EngineResult<(Array3<f32>,)> {
        let mut dec_state = self.decoder_state.lock().unwrap();
        let mut decoder_session = self.decoder_session.lock().unwrap();

        let (logits, new_kv) = if let Some(prev_kv) = dec_state.kv_cache.take() {
            // Step N
            decoder_next_step(
                &mut decoder_session,
                input_ids,
                encoder_hidden,
                encoder_attention_mask,
                prev_kv,
            )?
        } else {
            // Step 0
            decoder_first_step(
                &mut decoder_session,
                input_ids,
                encoder_hidden,
                encoder_attention_mask,
            )?
        };

        dec_state.kv_cache = Some(new_kv);

        Ok((logits,))
    }
}
```

---

## 7. Next Possible Enhancements

Once this decoder path is stable, you can:

- Implement **full greedy decoding** loop:
  - 每次取 argmax(logits[:, -1, :])
  - 追加到 `input_ids`
  - 调用 `decode_step`，直到遇到 `</s>` 或长度上限
- 集成到你的 `NmtIncremental` trait 实现中：
  - `on_partial_transcript` → 调用 encoder + decoder
  - 返回增量翻译字符串
- 支持多语言：
  - `LanguagePair` → 选择不同的 Marian 模型与 tokenizer
- 性能优化：
  - 用更高的图优化等级 / GPU backend
  - 复用 `encoder_hidden_states`，只在输入语句改变时重新跑 encoder

---

## 8. Conclusion

- 你现在的基础设施（文件模式 Session、ORT 1.16.3、logits 提取方式、KV 生命周期）都已经非常好；
- 唯一残留问题是 **“人为构造 empty KV 导致 rank=1”**；
- 采用 **Step0 无 KV + StepN 传回 present.* 作为 past_key_values.*** 的方案，可以：
  - 完全避免手动构造 KV 的维度与布局；
  - 对齐主流 transformer 实现的实践经验；
  - 让代码更安全、更简单、更易维护。

你可以把这份文档直接放入仓库，例如：  
`core/engine/docs/recommended_kv_solution.md`，作为 NMT 架构设计说明的一部分。
