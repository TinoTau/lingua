# Marian Decoder 输入构造代码

**日期**: 2025-11-21  
**文件**: `core/engine/src/nmt_incremental/decoder.rs`

---

## 错误日志

```
[6/7] 执行 NMT 翻译...
Source text: 'Hello welcome. Hello welcome to the video. Welcome to the video. Hello welcome to the video of the video. Hello, welcome to the \"Lenlo Yu\" movie series.'
Encoded source IDs: [0, 3833, 0, 3833, 2904, 8, 3, 0, 12904, 8, 3, 0, 3833, 2904, 8, 3, 9011, 4, 3, 0, 0, 2904, 8, 3, 0, 0, 15807, 0, 0] (length: 29)
Encoder output shape: [1, 29, 512]
[DEBUG] Step 0: decoder_input_ids=[65000] (length: 1), use_cache_branch=false, has_decoder_kv=false
[decoder_step] step input_ids_len=1, use_cache_branch=false, has_decoder_kv=false
[decoder_step] input_ids shape: [1, 1]
error: process didn't exit successfully: `target\debug\examples\test_s2s_full_simple.exe ..\..\test_output\s2s_flow_test.wav` (exit code: 0xc0000005, STATUS_ACCESS_VIOLATION)
```

**错误位置**: `decoder_step` 函数中调用 `decoder_session.run(input_values)` 时

---

## Decoder 输入构造代码

### 完整函数签名

```rust
pub(crate) fn decoder_step(
    &self,
    encoder_hidden_states: &Array3<f32>,      // [1, encoder_seq_len, hidden_dim]
    encoder_attention_mask: &Array2<i64>,     // [1, encoder_seq_len]
    mut state: DecoderState,
    static_encoder_kv: &Vec<(Value<'static>, Value<'static>)>,  // 静态 encoder KV 占位符
) -> anyhow::Result<(Array1<f32>, DecoderState)>
```

### 输入构造过程

```rust
// 1. 准备 decoder input_ids: [1, cur_len]
let batch_size = 1usize;
let cur_len = state.input_ids.len();
let decoder_input_ids = Array2::<i64>::from_shape_vec(
    (batch_size, cur_len),
    state.input_ids.clone(),
)?;

// 2. use_cache_branch: [1]，类型是 Bool
let use_cache_array = Array1::<bool>::from_vec(vec![state.use_cache_branch]);

// 3. 转换为 Value 的宏
macro_rules! array_to_value {
    ($arr:expr, $ty:ty) => {{
        let arr_dyn = $arr.into_dyn();
        let arr_owned = arr_dyn.to_owned();
        let cow_arr = CowArray::from(arr_owned);
        let value = Value::from_array(ptr::null_mut(), &cow_arr)
            .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
        Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
    }};
}

// 4. 转换所有输入为 Value
let input_ids_value = array_to_value!(decoder_input_ids, i64)?;
let encoder_states_value = array_to_value!(encoder_hidden_states.clone(), f32)?;
let encoder_mask_value = array_to_value!(encoder_attention_mask.clone(), i64)?;
let use_cache_value = array_to_value!(use_cache_array, bool)?;

// 5. 组织输入顺序（严格按照模型 I/O 顺序）
// 输入顺序：encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.*, use_cache_branch
let mut input_values: Vec<Value<'static>> = Vec::new();

// 1. encoder_attention_mask
input_values.push(encoder_mask_value);
// 2. input_ids
input_values.push(input_ids_value);
// 3. encoder_hidden_states
input_values.push(encoder_states_value);

// 4. KV cache：准备输入 KV cache
// - Decoder KV: 使用 state.decoder_kv_cache（如果可用）或零占位符
// - Encoder KV: 始终使用静态占位符（每次步骤都相同）
let encoder_seq_len = encoder_hidden_states.shape()[1];

// 准备 decoder KV cache
let decoder_kv = if state.use_cache_branch && state.decoder_kv_cache.is_some() {
    // 正常模式：使用历史 decoder KV cache
    state.decoder_kv_cache.take().unwrap()
} else {
    // 第一步：使用零占位符
    self.build_zero_decoder_kv()?
};

// 构建完整的 KV cache 输入（模型需要 4 个值：dec_k, dec_v, enc_k, enc_v）
// 根据 marian_nmt_interface_spec.md：Encoder KV 始终使用静态占位符
let static_enc_kv = self.build_static_encoder_kv(encoder_seq_len)?;
let mut decoder_kv_iter = decoder_kv.into_iter();
let mut static_enc_kv_iter = static_enc_kv.into_iter();

for layer_idx in 0..Self::NUM_LAYERS {
    // Decoder KV
    let (dec_k, dec_v) = decoder_kv_iter.next()
        .ok_or_else(|| anyhow!("insufficient decoder KV cache for layer {}", layer_idx))?;
    input_values.push(dec_k);
    input_values.push(dec_v);
    
    // Encoder KV: 使用静态占位符（每次步骤都相同）
    let (enc_k, enc_v) = static_enc_kv_iter.next()
        .ok_or_else(|| anyhow!("insufficient static encoder KV for layer {}", layer_idx))?;
    input_values.push(enc_k);
    input_values.push(enc_v);
}

// 5. use_cache_branch
input_values.push(use_cache_value);

// 6. 调用 session.run（错误发生在这里）
let decoder_session = self.decoder_session.lock().unwrap();
let outputs: Vec<Value<'static>> = decoder_session.run(input_values)  // ❌ 访问违规错误
    .map_err(|e| anyhow!("failed to run decoder model: {e}"))?;
```

---

## 输入顺序和类型

### 输入顺序（共 28 个输入）

1. `encoder_attention_mask` - `Array2<i64>` - `[1, encoder_seq_len]`
2. `input_ids` - `Array2<i64>` - `[1, decoder_seq_len]`
3. `encoder_hidden_states` - `Array3<f32>` - `[1, encoder_seq_len, hidden_dim]`
4-27. `past_key_values.*` - 每层 4 个 KV cache（共 6 层 × 4 = 24 个）
   - `past_key_values.{layer}.decoder.key` - `Array4<f32>` - `[1, num_heads, past_seq, head_dim]`
   - `past_key_values.{layer}.decoder.value` - `Array4<f32>` - `[1, num_heads, past_seq, head_dim]`
   - `past_key_values.{layer}.encoder.key` - `Array4<f32>` - `[1, num_heads, encoder_seq_len, head_dim]`
   - `past_key_values.{layer}.encoder.value` - `Array4<f32>` - `[1, num_heads, encoder_seq_len, head_dim]`
28. `use_cache_branch` - `Array1<bool>` - `[1]`

### 第一步（use_cache_branch=false）的 KV Cache

**Decoder KV**:
- 使用 `build_zero_decoder_kv()` 创建零占位符
- 形状: `[1, num_heads, 1, head_dim]`（past_seq=1）

**Encoder KV**:
- 使用 `build_static_encoder_kv(encoder_seq_len)` 创建静态占位符
- 形状: `[1, num_heads, encoder_seq_len, head_dim]`

---

## KV Cache 构建函数

### build_zero_decoder_kv()

```rust
pub(crate) fn build_zero_decoder_kv(
    &self,
) -> anyhow::Result<Vec<(Value<'static>, Value<'static>)>> {
    use ndarray::Array4;
    use std::ptr;
    use ndarray::CowArray;
    use anyhow::anyhow;

    let batch = 1usize;
    let dec_len = 1usize;  // decoder "历史长度"占位为 1

    let zeros_dec = Array4::<f32>::zeros((batch, Self::NUM_HEADS, dec_len, Self::HEAD_DIM));

    macro_rules! array_to_value {
        ($arr:expr) => {{
            let arr_dyn = $arr.into_dyn();
            let arr_owned = arr_dyn.to_owned();
            let cow_arr = CowArray::from(arr_owned);
            let value = Value::from_array(ptr::null_mut(), &cow_arr)
                .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
            Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
        }};
    }

    let mut result = Vec::with_capacity(Self::NUM_LAYERS);

    for _ in 0..Self::NUM_LAYERS {
        // 每层有 2 个 decoder KV：dec_k, dec_v
        let dec_k = array_to_value!(zeros_dec.clone())?;
        let dec_v = array_to_value!(zeros_dec.clone())?;
        result.push((dec_k, dec_v));
    }

    Ok(result)
}
```

### build_static_encoder_kv()

```rust
pub(crate) fn build_static_encoder_kv(
    &self,
    encoder_seq_len: usize,
) -> anyhow::Result<Vec<(Value<'static>, Value<'static>)>> {
    use ndarray::Array4;
    use std::ptr;
    use ndarray::CowArray;
    use anyhow::anyhow;

    let batch = 1usize;
    let dec_len = 1usize;           // decoder "历史长度"占位为 1
    let enc_len = encoder_seq_len;  // encoder 长度与真实输入一致

    let zeros_enc = Array4::<f32>::zeros((batch, Self::NUM_HEADS, enc_len, Self::HEAD_DIM));

    macro_rules! array_to_value {
        ($arr:expr) => {{
            let arr_dyn = $arr.into_dyn();
            let arr_owned = arr_dyn.to_owned();
            let cow_arr = CowArray::from(arr_owned);
            let value = Value::from_array(ptr::null_mut(), &cow_arr)
                .map_err(|e| anyhow!("failed to convert array to Value: {:?}", e))?;
            Ok::<ort::Value<'static>, anyhow::Error>(unsafe { std::mem::transmute::<ort::Value, ort::Value<'static>>(value) })
        }};
    }

    let mut result = Vec::with_capacity(Self::NUM_LAYERS);

    for _ in 0..Self::NUM_LAYERS {
        // 每层有 2 个 encoder KV：enc_k, enc_v
        let enc_k = array_to_value!(zeros_enc.clone())?;
        let enc_v = array_to_value!(zeros_enc.clone())?;
        result.push((enc_k, enc_v));
    }

    Ok(result)
}
```

---

## 关键常量

```rust
impl MarianNmtOnnx {
    const NUM_LAYERS: usize = 6;      // Marian 模型层数
    const NUM_HEADS: usize = 8;       // 注意力头数
    const HEAD_DIM: usize = 64;       // 每个头的维度
}
```

---

## 错误发生时的状态

根据错误日志，在第一步时：

- `decoder_input_ids`: `[65000]` (BOS token)
- `decoder_input_ids` 形状: `[1, 1]`
- `encoder_hidden_states` 形状: `[1, 29, 512]`
- `encoder_attention_mask` 形状: `[1, 29]`
- `use_cache_branch`: `false`
- `has_decoder_kv`: `false`

**KV Cache 状态**:
- Decoder KV: 零占位符，形状 `[1, 8, 1, 64]`（6 层）
- Encoder KV: 静态占位符，形状 `[1, 8, 29, 64]`（6 层）

**总输入数量**: 3（基础输入）+ 24（KV cache）+ 1（use_cache_branch）= 28 ✅

---

**最后更新**: 2025-11-21

