
# NMT ONNX File Mode Migration Guide
## Overview
This guide explains how to migrate your Marian NMT ONNX inference code to **file-based session mode**, required because:
- `InMemorySession` cannot be stored in `Arc<Mutex<...>>`
- `Session` cannot be constructed from raw bytes
- KV cache cannot safely be extracted using `try_extract_tensor()` in older `ort` crate versions

This guide summarizes the full architecture and provides working code templates for your GitHub project.

---

# 1. Why Switch to File‑Mode?
Your previous implementation relied on:

```rust
SessionBuilder::new().with_model_from_memory(bytes).build()
```

But:
- This produces **InMemorySession<'a>**, which cannot be stored in your struct.
- You need `Arc<Mutex<Session>>` for incremental decoding.
- ONNX Runtime prevents dynamically cloning or transferring in‑memory sessions.

**Solution → Use file-mode sessions everywhere**

```rust
SessionBuilder::new().with_model_from_file("path/to/model.onnx").build()
```

Advantages:
- No lifetime issues
- Safe to store in `Arc<Mutex<Session>>`
- Works with decoder KV‑cache outputs
- Compatible with future GPU execution (CUDA/TensorRT)

---

# 2. Required Code Refactor

## 2.1 Modify `NmtIncremental` Implementation

### **Old (in‑memory mode, broken)**
```rust
let decoder_bytes = include_bytes!("marian_decoder.onnx");
let decoder_session = SessionBuilder::new()
    .with_model_from_memory(decoder_bytes)
    .build()?;
```

### **New (file‑mode, correct)**
```rust
let decoder_session = Arc::new(Mutex::new(
    SessionBuilder::new()
        .with_model_from_file(decoder_path)
        .build()?
));
```

---

# 3. Handling KV Cache in File Mode

KV cache consists of **24 outputs**, one for each:
- present.X.decoder.key
- present.X.decoder.value
- present.X.encoder.key
- present.X.encoder.value  
for X = 0..5 layers.

### SAFE RULE:
🚫 **Never use `try_extract_tensor()` with f32 KV cache outputs.**  
Reason: ONNX Runtime sometimes allocates unaligned memory → Rust slice assumptions break → undefined behavior.

### ✔ Correct handling: treat KV cache as **opaque Value**

#### Store as black‑box:
```rust
let mut kv_cache: Vec<Value> = Vec::new();

kv_cache.push(outputs[i].clone()); // when clone is not available:
kv_cache.push(Value::from_array(session.allocator(), array)?);
```

#### Pass directly into next step:
```rust
("past_key_values.0.decoder.key", kv_cache[0].clone()),
```

You do **not** need to understand or transform KV tensors at all.

---

# 4. How to Load Inputs / Outputs Safely

## 4.1 Extract logits (f32 OK)
```rust
let logits: OrtOwnedTensor<f32, _> = outputs[0].try_extract()?;
let logits_arr = logits.view().to_owned();
```

## 4.2 Extract KV Cache (DO NOT convert)
```rust
let kv_val = outputs[i].clone().unwrap_value();
kv_cache.push(kv_val);
```

---

# 5. Directory Layout Required

Place all Marian models under:

```
core/engine/models/nmt/
    marian-en-zh/
        config.json
        vocab.json
        source.spm
        target.spm
        encoder.onnx
        decoder.onnx
```

You may also add:

```
    marian-zh-en/
    marian-en-es/
    marian-es-en/
```

---

# 6. New mod.rs (Summary)

```rust
pub mod tokenizer;
pub mod language_pair;
pub mod marian;

pub use marian::MarianNmtIncremental;
```

---

# 7. Updated language_pair.rs (Summary)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguagePair {
    pub source: String,
    pub target: String,
}

impl LanguagePair {
    pub fn new(src: &str, tgt: &str) -> Self {
        Self { source: src.into(), target: tgt.into() }
    }

    pub fn model_dir(&self) -> String {
        format!("models/nmt/{}-{}", self.source, self.target)
    }
}
```

---

# 8. Updated tokenizer.rs (Summary)

```rust
pub struct Tokenizer {
    source_spm: SentencePieceProcessor,
    target_spm: SentencePieceProcessor,
}

impl Tokenizer {
    pub fn encode_source(&self, text: &str) -> Vec<i64> {
        self.source_spm.encode(text)
    }

    pub fn decode_target(&self, ids: &[i64]) -> String {
        self.target_spm.decode(ids)
    }
}
```

---

# 9. Decoder Step Template (Final Version)

```rust
pub fn decoder_step(
    session: &mut Session,
    input_ids: Vec<i64>,
    encoder_hidden: Value,
    kv_cache: Vec<Value>,
) -> Result<(Array3<f32>, Vec<Value>)> {

    let mut inputs = vec![
        ("input_ids", array_to_value!(input_ids_array)),
        ("encoder_hidden_states", encoder_hidden),
        ("use_cache_branch", Value::from_array(session.allocator(), arr1(&[true]) )?),
    ];

    // Rebind KV cache
    for (i, cache_val) in kv_cache.into_iter().enumerate() {
        inputs.push((KV_INPUT_NAMES[i], cache_val));
    }

    let outputs = session.run(inputs)?;

    let logits: OrtOwnedTensor<f32, IxDyn> = outputs[0].try_extract()?;
    let logits_arr = logits.view().to_owned().into_dimensionality::<Ix3>()?;

    // Collect new KV cache
    let mut new_cache = Vec::new();
    for i in 1..=24 {
        new_cache.push(outputs[i].unwrap_value());
    }

    Ok((logits_arr, new_cache))
}
```

---

# 10. Next Steps
You can now:

✔ Run multi-step decoding  
✔ Support any language pair  
✔ Avoid unsafe memory issues  
✔ Prepare for GPU inference  
✔ Integrate with your real pipeline (ASR → NMT → TTS)

If you'd like, I can generate:
- Fully finished Rust code for `marian.rs`
- A complete incremental decode loop
- A full NMTEngine wrapper
- Automatic language detection system
- Benchmark / profiling utilities

