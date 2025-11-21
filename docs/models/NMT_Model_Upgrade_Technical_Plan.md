# NMT Model Upgrade Technical Plan — From Marian to M2M100/NLLB

## 1. Background

The current NMT system uses **Helsinki-NLP/opus-mt-en-zh** and **opus-mt-zh-en**, which are:

- Trained on older OPUS corpora  
- Not optimized for oral English / casual video speech  
- Unstable for repetitive or short utterances  
- Likely to produce unnatural or “hallucinated” translations

Recent S2S tests show that even with a correct pipeline:

```text
EN Speech → Whisper → EN text → Marian → ZH text → Piper CN TTS
```

the translation result can be far away from the original meaning (e.g., *“欢迎来到纽约 来个大餐”* for a simple “Hello, welcome to the video” type sentence).

Root cause: **Marian model quality is insufficient for real-time spoken-language translation.**

---

## 2. Upgrade Target

Introduce a **modern, high-quality multilingual NMT backend** that:

- Handles oral / conversational text robustly
- Supports incremental decoding with KV cache
- Fits into the existing **Encoder + Decoder + KV (28-input) architecture**
- Requires minimal changes to Rust / Node integration
- Scales to more languages (e.g., JP/KR/ES/FR) in the future

---

## 3. Recommended Models

### 3.1 Primary: `facebook/m2m100_418M`

Benefits:

- Strong translation quality for EN ↔ ZH
- Good behavior on conversational / noisy inputs
- Full HuggingFace support (tokenizer, language codes)
- Encoder–decoder architecture, friendly to KV-cache ONNX export
- Multilingual (future languages come “for free”)

### 3.2 Secondary: NLLB-200 (e.g., `facebook/nllb-200-distilled-600M`)

Benefits:

- Even higher translation quality
- Broad language support

Trade-offs:

- Heavier model, more memory
- ONNX export and optimization more involved

**Phase 1**: adopt **M2M100-418M** as the new default NMT backend.  
**Phase 2**: optionally add NLLB using the same interface.

---

## 4. Directory Layout

Keep the current NMT model layout, add M2M100 models as follows:

```text
core/engine/models/nmt/
  ├─ m2m100-en-zh/
  │    ├─ encoder.onnx
  │    ├─ decoder.onnx
  │    ├─ tokenizer.json
  │    ├─ sentencepiece.model
  │    └─ config.json (optional)
  └─ m2m100-zh-en/
       ├─ encoder.onnx
       ├─ decoder.onnx
       └─ ...
```

In practice, M2M100 is a single multilingual model; we can still keep two folders for clarity (direction-specific configs / wrappers).

---

## 5. Migration Strategy

### 5.1 Encoder

M2M100 encoder output:

```text
last_hidden_state: float32[batch, src_seq, 1024]
```

This matches the existing pipeline (only the hidden size changes).  
Rust side does **not** need structural changes; only the model path and tensor dimensions in logs need to be updated.

### 5.2 Decoder + KV Cache

To stay compatible with the existing decoder interface used for Marian, we keep the same **28-input** ONNX signature:

Inputs:

1. `encoder_attention_mask` — int64[batch, src_seq]  
2. `input_ids` — int64[batch, tgt_seq] (decoder input ids)  
3. `encoder_hidden_states` — float32[batch, src_seq, 1024]  
4. `past_key_values.{layer}.decoder.key` — float32[batch, heads, past_seq, head_dim]  
5. `past_key_values.{layer}.decoder.value` — same shape  
6. `past_key_values.{layer}.encoder.key` — float32[batch, heads, src_seq, head_dim]  
7. `past_key_values.{layer}.encoder.value` — same shape  
8. `use_cache_branch` — bool[1]

Outputs:

- `logits` — float32[batch, tgt_seq, vocab_size]  
- `present.{layer}.decoder.key/value`  
- `present.{layer}.encoder.key/value`  

M2M100 config (418M):

- `decoder_layers = 12`  
- `decoder_attention_heads = 16`  
- `d_model = 1024` → `head_dim = 64`

KV cache structure is the same as Marian’s, only the number of layers and heads differ.

### 5.3 Tokenizer and Language Codes

Switch from Marian tokenizer to **`M2M100Tokenizer`**.

Before encoding, we must set the source language, and the decoder must start with the target language id:

Example (EN → ZH):

```python
tokenizer.src_lang = "en"
enc = tokenizer(source_text, return_tensors="pt")
input_ids = enc["input_ids"]
attention_mask = enc["attention_mask"]

decoder_start_token_id = tokenizer.get_lang_id("zh")
```

The ONNX export scripts embed this logic for dummy inputs.  
At runtime, the high-level integration (Python/Rust/Node) should ensure that the correct `src_lang` / `tgt_lang` are selected based on direction.

---

## 6. Integration Changes

### 6.1 Add NMT backend enum

In Rust (or the central configuration), add:

```rust
enum NmtBackend {
    Marian,
    M2M100,
    NLLB,
}
```

And a config entry to select:

- backend type: `Marian` or `M2M100`
- direction: `en-zh` or `zh-en`

### 6.2 Direction Handling

For **EN → ZH**:

- ASR: Whisper EN  
- NMT: M2M100 with `src_lang = "en"`, `tgt_lang = "zh"`  
- TTS: Chinese voice (Piper CN)

For **ZH → EN**:

- ASR: Whisper ZH (or EN+ZH joint model, but language tag “zh” input)  
- NMT: M2M100 with `src_lang = "zh"`, `tgt_lang = "en"`  
- TTS: English voice

Existing test script `test_s2s_full_simple` can be extended with:

```bash
--nmt-backend m2m100
--direction en-zh   # or zh-en
```

### 6.3 Decoder Loop and KV Cache

No structural changes required:

- Same number of tensors passed to ONNX session (28 inputs)
- Same KV cache communication pattern:
  - Initial step: all-zero KV
  - Subsequent steps: feed previous `present.*` back as `past_key_values.*`
- Only dimensions change:
  - layers: 12 instead of 6
  - heads: 16 instead of 8

Rust-side tensor shapes must be adapted accordingly, but the logic remains identical.

---

## 7. Expected Improvements

| Model         | Oral Quality | Stability | Repetition Handling | Real-time Suitability |
|---------------|-------------|-----------|---------------------|------------------------|
| Marian        | Low         | Poor      | Poor                | Fast but inaccurate    |
| M2M100 (418M) | High        | Good      | Good                | Good                   |
| NLLB          | Very High   | Very Good | Very Good           | Medium                 |

For the project’s use case (real-time EN↔ZH speech translation), **M2M100** is the most balanced choice.

---

## 8. Work Items

| Task                                        | Owner      | Estimate |
|---------------------------------------------|-----------|----------|
| Export M2M100 encoder/decoder to ONNX       | AI module | 1 day    |
| Integrate tokenizer & direction switching   | Dev team  | 1 day    |
| Replace Marian pipeline with M2M100 backend | Dev team  | 1 day    |
| End-to-end testing (EN↔ZH S2S)              | QA        | 1 day    |

---

## 9. Deliverables

1. `encoder.onnx` (M2M100 encoder)  
2. `decoder.onnx` (M2M100 decoder with KV cache)  
3. `tokenizer.json` + `sentencepiece.model`  
4. `export_m2m100_encoder.py`  
5. `export_m2m100_decoder_kv.py`  
6. Updated integration test scripts supporting M2M100 backend

---

## 10. Conclusion

Upgrading from Marian to **M2M100** will:

- Significantly improve translation accuracy for oral and conversational text  
- Fit into the existing ONNX + KV cache architecture with minimal changes  
- Provide a clear path to future NLLB integration if needed

This is a **low-risk, high-impact** upgrade and is recommended as the default NMT backend going forward.
