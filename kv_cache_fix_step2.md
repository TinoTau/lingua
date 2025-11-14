# kv_cache_fix_step2.md  
## Fixing KV Cache for Marian ONNX Decoder (ORT 1.16.3)  
### Ensuring first-step stability by supplying placeholder KV tensors

---

## 背景

在当前 Marian NMT ONNX 模型（你 repo 中的 `marian-en-zh`）里：  

- **所有 `past_key_values.*` 都是“必填输入”**  
- ONNX Runtime 即使在 `use_cache_branch = false` 的情况下，也 **仍然要求你提供所有 KV 输入**  

因此，第一步无法做到「完全不传 KV」。  
**必须传入“合法形状的占位 KV 张量（全 0）”，否则 `session.run()` 会报错。**

本文件给出与你当前 GitHub 项目高度兼容的改法。

---

## 目标

1. 确保第一步 `decoder_step` **不 crash**。  
2. 让 KV cache 的生命周期变成：  
   - 第一步：占位 KV（zeros）  
   - 后续：使用上一轮输出的 `present.*`  
3. 避免 `try_extract_tensor()` 在 KV 上触发 unsafe null pointer 问题。  
4. 与 **ONNX Runtime 1.16.3** 的 API 完全兼容。

---

## 修改内容概览

需要修改两部分：

1. **新增一个 KV placeholder 构造器**：  
   `build_initial_kv_values()`

2. **修改 `decoder_step()`**：让第一步也传 KV，但用占位版本，而不是“完全不传”。

原本逻辑（有问题的大致结构）：

```rust
if let Some(prev_kv) = state.kv_cache.take() {
    // push prev_kv 到 inputs
} else {
    // 第一轮：什么 KV 都不 push → 模型报缺失输入
}
```

应修改成：

```rust
let kv_to_use: Vec<[Value<'static>; 4]> =
    if let Some(prev_kv) = state.kv_cache.take() {
        prev_kv
    } else {
        self.build_initial_kv_values(encoder_seq_len)?
    };
```

---

## 1. 新增占位 KV 生成器

**文件位置：**  
`core/engine/src/nmt_incremental/mod.rs`  

在 `impl MarianNmtOnnx` 里增加常量和 helper 函数：

```rust
impl MarianNmtOnnx {
    const NUM_LAYERS: usize = 6;
    const NUM_HEADS: usize = 8;
    const HEAD_DIM: usize = 64;

    /// 构造第一步用的零张量 KV 值
    pub fn build_initial_kv_values(
        &self,
        encoder_seq_len: usize,
    ) -> anyhow::Result<Vec<[Value<'static>; 4]>> {
        use ndarray::Array4;
        use ort::value::Value;

        let batch = 1usize;
        let dec_len = 1usize;           // decoder “历史长度”占位为 1
        let enc_len = encoder_seq_len;  // encoder 长度与真实输入一致

        let zeros_dec =
            Array4::<f32>::zeros((batch, Self::NUM_HEADS, dec_len, Self::HEAD_DIM));
        let zeros_enc =
            Array4::<f32>::zeros((batch, Self::NUM_HEADS, enc_len, Self::HEAD_DIM));

        let mut result = Vec::with_capacity(Self::NUM_LAYERS);

        for _ in 0..Self::NUM_LAYERS {
            // 使用你当前 mod.rs 中已经实现的 array_to_value! 宏
            let dec_k = array_to_value!(zeros_dec.clone());
            let dec_v = array_to_value!(zeros_dec.clone());
            let enc_k = array_to_value!(zeros_enc.clone());
            let enc_v = array_to_value!(zeros_enc.clone());
            result.push([dec_k, dec_v, enc_k, enc_v]);
        }

        Ok(result)
    }
}
```

> ✅ 注意：  
> - 这里对 `zeros_dec` / `zeros_enc` 只做克隆 + `array_to_value!`，**不会再对这些 KV 做 `try_extract_tensor`**。  
> - 形状严格符合你打印出的 `present.*` 形状模式：`[batch, num_heads, seq_len, head_dim]`。

---

## 2. 修改 `decoder_step`（核心修复）

### 原逻辑（有问题的版本）

逻辑大意：

```rust
let mut input_values = Vec::<Value<'static>>::new();

// 1) encoder_attention_mask / input_ids / encoder_hidden_states
input_values.push(encoder_mask_value);
input_values.push(input_ids_value);
input_values.push(encoder_states_value);

// 2) KV cache
if let Some(prev_kv) = state.kv_cache.take() {
    for layer in 0..Self::NUM_LAYERS {
        let [dec_k, dec_v, enc_k, enc_v] = prev_kv[layer];
        input_values.push(dec_k);
        input_values.push(dec_v);
        input_values.push(enc_k);
        input_values.push(enc_v);
    }
}

// 3) use_cache_branch
input_values.push(use_cache_value);

// 然后调用 decoder_session.run(input_values)
```

**问题：**  
- 第一轮 `state.kv_cache == None`，导致完全不传 `past_key_values.*` 给模型。  
- 但模型的 ONNX 签名要求这些输入是“必填”，ORT 会报错：**缺失输入 / shape 不匹配**。

### 修复后的逻辑

**核心思路**：

- 第一轮：`kv_cache == None` → 调用 `build_initial_kv_values` 构造占位 KV。  
- 第二轮以后：使用上一步返回的 `present.*`。  
- 所以 `decoder_step` 里 **任何时候都会传完整的 24 个 KV 输入**。

示例结构：

```rust
fn decoder_step(
    &self,
    encoder_hidden_states: &Array3<f32>,
    encoder_attention_mask: &Array2<i64>,
    mut state: DecoderState,
) -> anyhow::Result<(Array1<f32>, DecoderState)> {
    use ort::value::Value;

    let batch_size = 1usize;
    let encoder_seq_len = encoder_hidden_states.shape()[1];

    let mut input_values: Vec<Value<'static>> = Vec::new();

    // 1) encoder_attention_mask / input_ids / encoder_hidden_states → Value
    input_values.push(encoder_mask_value);      // encoder_attention_mask
    input_values.push(input_ids_value);         // decoder input_ids
    input_values.push(encoder_states_value);    // encoder_hidden_states

    // 2) KV cache：第一步用占位，后续用真实 present.*
    let kv_to_use: Vec<[Value<'static>; 4]> =
        if let Some(prev_kv) = state.kv_cache.take() {
            prev_kv
        } else {
            // 第一步：构造零张量占位 KV
            self.build_initial_kv_values(encoder_seq_len)?
        };

    for layer in 0..Self::NUM_LAYERS {
        let [dec_k, dec_v, enc_k, enc_v] = kv_to_use[layer];
        input_values.push(dec_k);
        input_values.push(dec_v);
        input_values.push(enc_k);
        input_values.push(enc_v);
    }

    // 3) use_cache_branch：第一步为 false，之后 decoder_step 内部改为 true
    input_values.push(use_cache_value);

    // 4) 调用 decoder session
    let mut decoder_session = self.decoder_session.lock().unwrap();
    let outputs: Vec<Value<'static>> = decoder_session.run(input_values)?;

    // 5) 从 outputs 中取 logits + present.*
    //    - outputs[0] = logits
    //    - outputs[1..] = present.*（按层 * 4）
    //    - logits 用 try_extract_tensor
    //    - present.* 只“搬运”成新的 kv_cache，不做 try_extract_tensor

    // …… logits 提取略（你当前实现已经正确） ……

    let mut iter = outputs.into_iter();
    let logits_value = iter.next().expect("missing logits");
    // 从 logits_value 提取 Array3 / Array1<f32> 的逻辑沿用你现有的

    let mut next_kv: Vec<[Value<'static>; 4]> =
        Vec::with_capacity(Self::NUM_LAYERS);

    for _ in 0..Self::NUM_LAYERS {
        let dec_k = iter.next().expect("missing present.*.decoder.key");
        let dec_v = iter.next().expect("missing present.*.decoder.value");
        let enc_k = iter.next().expect("missing present.*.encoder.key");
        let enc_v = iter.next().expect("missing present.*.encoder.value");
        next_kv.push([dec_k, dec_v, enc_k, enc_v]);
    }

    state.kv_cache = Some(next_kv);
    state.use_cache_branch = true; // 后续步骤开始走缓存分支

    Ok((last_step_logits, state))
}
```

> ✅ 关键点：  
> - 无论第几步，都向模型提供完整的 KV 输入（第一步用占位 zeros）。  
> - 只对 `logits` 调用 `try_extract_tensor`，**绝不对 KV 做 `try_extract`**。  
> - `state.kv_cache` 通过 `take()` + `Some(next_kv)` 完整搬运，避免 `Clone` / 生命周期问题。

---

## 3. `translate()` 初始化要求

在 `MarianNmtOnnx::translate(&self, source_text: &str)` 里，确保：

```rust
let mut state = DecoderState {
    input_ids: vec![self.decoder_start_token_id],
    generated_ids: Vec::new(),
    kv_cache: None,          // 让第一步自动走占位 KV 分支
    use_cache_branch: false, // 第一轮不走 cache 分支
};
```

解码循环中，每一轮调用 `decoder_step` 更新这个 `state`，直到：

- 生成 EOS token，或者  
- 达到 `max_length` 限制。

最后：

```rust
let translated = self.tokenizer.decode(&state.generated_ids);
Ok(translated)
```

---

## 4. 为什么第一步必须传占位 KV？

你的 ONNX 模型输入签名大致如下：

- `encoder_attention_mask`
- `input_ids`
- `encoder_hidden_states`
- `past_key_values.0.decoder.key`
- `past_key_values.0.decoder.value`
- `past_key_values.0.encoder.key`
- `past_key_values.0.encoder.value`
- …
- `past_key_values.5.encoder.value`
- `use_cache_branch`

ORT 在执行 `session.run()` 时会：

- 检查 **每个必填输入是否存在**  
- 检查 **每个输入的 shape 是否符合模型要求**  
- 不会因为 `use_cache_branch = false` 就跳过上述检查

因此：

> ❌ 不传 KV = 100% 报错（缺失输入 / 维度错误）  
> ✅ 传合法 shape 的占位 zero tensor = 稳定运行（第一步即使用不到，也必须交卷）

---

## 5. 这个方案避免了哪些错误？

通过本方案，可以避免：

- ✅ `try_extract_tensor` 在 KV 上触发 `unsafe precondition violated`（null pointer）  
- ✅ KV 的 rank 不匹配（预期 4 维，实际 1 维）  
- ✅ 第一轮缺少 `past_key_values.*` 导致的 ORT 输入检查错误  
- ✅ 手动构造不一致形状的 `Array4` KV 导致的潜在 UB  

同时：

- 保持了你现在的 **架构设计**（`DecoderState`, `decoder_step`, `translate`）。  
- KV 始终当黑盒 `Value<'static>` 搬运，生命周期简单清晰。

---

## 6. 建议的测试步骤

### 6.1 单步 decoder 测试

继续使用你已有的：

```bash
cargo test test_marian_decoder_single_step -- --nocapture
```

确保：

- 日志中有 `Decoder logits shape: [1, 1, vocab_size]`  
- 不再出现内存错误 / null pointer。

### 6.2 两轮 decoder_step 测试（建议新增）

新增测试：

1. 手工初始化 `DecoderState`（`kv_cache = None`）。  
2. 调用 `decoder_step` 两次：
   - 第一次使用占位 KV → 得到 `present.*`。  
   - 第二次使用第一次的 `state.kv_cache`。  

观察：

- 两次都能拿到 logits；  
- 第二次不报错，说明 KV 生命周期正确。

### 6.3 端到端翻译测试

写一个简单的集成测试或 bin，调用：

```rust
let result = marian.translate("Hello world")?;
println!("NMT result = {}", result);
```

现在翻译质量不一定完美，但：

- 不崩溃，  
- 不出现「内存非法访问」之类错误，  
- 解码可以正常停止（遇到 EOS 或长度上限）。

---

## 7. TL;DR（一页总览）

- **原因**：ONNX 模型把 `past_key_values.*` 设为了必填输入 → 第一轮也必须给。  
- **方案**：  
  - 新增 `build_initial_kv_values()`：构造合法形状的零张量 KV。  
  - 在 `decoder_step` 中：
    - `kv_cache.is_none()` → 调用 `build_initial_kv_values`。  
    - 否则使用上一轮的 `present.*`。  
  - KV 全程作为 `Value<'static>` 黑盒搬运，只在 logits 上做 `try_extract_tensor`。  
- **效果**：  
  - 第一轮不再缺失 KV，模型输入检查通过。  
  - 不触碰 KV 内部数据，避免 unsafe 问题。  
  - 整体解码流程可以用于真实翻译。

---

如果你之后改完 `mod.rs`，可以把新的 `decoder_step` 和相关改动贴出来，我可以再帮你做一轮行级 review，看看有没有还可以收紧 / 优化的地方。  
