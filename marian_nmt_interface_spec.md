
# MarianNmtOnnx Interface Redesign (KV-safe Version)

## 1. Goals

- Keep external translator interface unchanged.
- Internally:
  - Only maintain *decoder KV cache*
  - Encoder is run once; encoder KV is never updated.
  - Provide stable input ordering for ONNX models.
  - Avoid unsafe operations (`try_extract_tensor`).
  - Treat `Value` as opaque.

## 2. Data Structures

### MarianNmtOnnxOptions

```rust
pub struct MarianNmtOnnxOptions {
    pub use_decoder_kv: bool,
    pub max_decode_steps: usize,
}
```

### DecoderState

```rust
pub struct DecoderState {
    pub encoder_hidden_states: ort::Value,
    pub encoder_attention_mask: ort::Value,
    pub decoder_kv: Option<Vec<ort::Value>>,
}
```

### StepOutput

```rust
pub struct StepOutput {
    pub logits: ndarray::Array1<f32>,
    pub token_id: i64,
}
```

## 3. Public Interface

### new_from_dir_with_options

```rust
impl MarianNmtOnnx {
    pub fn new_from_dir_with_options(
        model_dir: &Path,
        opts: MarianNmtOnnxOptions,
    ) -> Result<Self> {
        Ok(Self {
            opts,
            // encoder/decoder sessions...
        })
    }
}
```

### encode_source

```rust
pub fn encode_source(
    &self,
    input_ids: &Array2<i64>,
    attention_mask: &Array2<i64>,
) -> Result<DecoderState> {
    // run encoder session
    Ok(DecoderState {
        encoder_hidden_states,
        encoder_attention_mask,
        decoder_kv: None,
    })
}
```

## 4. Decoder Step (Critical Section)

### Input ordering (ONNX model)

```
0   encoder_attention_mask
1   input_ids
2   encoder_hidden_states
3   past_key_values.0.decoder.key
4   past_key_values.0.decoder.value
...
27  use_cache_branch
```

### decoder_step Implementation

```rust
pub fn decoder_step(
    &self,
    state: &mut DecoderState,
    next_token_id: i64,
    is_first_step: bool,
) -> Result<StepOutput> {
    let mut inputs = Vec::with_capacity(28);

    // 0
    inputs.push(state.encoder_attention_mask.clone());

    // 1
    let input_ids_value = array_to_value_i64(&[1,1], &[next_token_id])?;
    inputs.push(input_ids_value);

    // 2
    inputs.push(state.encoder_hidden_states.clone());

    // 3..26 KV processing
    for layer in 0..NUM_LAYERS {
        // Decoder KV
        let (dec_k, dec_v) = match &state.decoder_kv {
            Some(kv) => (
                kv[idx_for(layer, "decoder.key")].clone(),
                kv[idx_for(layer, "decoder.value")].clone(),
            ),
            None => self.build_zero_decoder_kv_for_layer(layer)?,
        };
        inputs.push(dec_k);
        inputs.push(dec_v);

        // Encoder KV = static placeholder
        let (enc_k, enc_v) = self.get_static_encoder_kv_for_layer(layer)?;
        inputs.push(enc_k);
        inputs.push(enc_v);
    }

    // 27 use_cache_branch
    let branch = self.opts.use_decoder_kv && !is_first_step;
    inputs.push(array_to_value_bool_scalar(branch)?);

    // run decoder
    let outputs = self.decoder_session.run(inputs)?;

    // extract logits
    let logits: ArrayD<f32> = outputs[0].try_extract()?;
    let logits = logits.into_dimensionality::<Ix3>()?;
    let token_logits = logits.slice(s![0,0,..]).to_owned();

    let (token_id, _) = argmax(&token_logits);

    // collect decoder KV
    let new_decoder_kv = collect_decoder_kv_from_outputs(&outputs)?;
    state.decoder_kv = Some(new_decoder_kv);

    Ok(StepOutput { logits: token_logits, token_id })
}
```

## 5. Why This Design Is Safe

- Encoder KV never changes → no reshape errors.
- Decoder KV remains incremental → fast inference.
- All ORT `Value` objects are treated opaquely → no unsafe extraction.
- Easy future migration to optimized encoder/decoder ONNX models.

