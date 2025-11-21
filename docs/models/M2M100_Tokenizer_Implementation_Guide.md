# M2M100 Tokenizer Implementation Guide

适用对象：开发部门（Rust / TypeScript / Python 集成）  
目标：在现有 NMT + KV Cache 架构中，正确接入 **M2M100Tokenizer**，替换原 Marian tokenizer。

---

## 1. 基本概念

M2M100 使用 HuggingFace 的 **M2M100Tokenizer**，主要特点：

- 基于 SentencePiece 的子词模型（`sentencepiece.model`）
- 使用 **语言代码 token** 控制源语言 / 目标语言
- 支持多语言，但我们当前仅关注：
  - 英文：`"en"`
  - 简体中文：`"zh"`

核心调用方式（Python）：

```python
from transformers import M2M100Tokenizer

tokenizer = M2M100Tokenizer.from_pretrained("facebook/m2m100_418M")

# 设置源语言
tokenizer.src_lang = "en"   # 或 "zh"

# 编码（给 encoder 用）
enc = tokenizer("Hello world", return_tensors="pt")
input_ids = enc["input_ids"]           # [batch, src_seq]
attention_mask = enc["attention_mask"] # [batch, src_seq]

# decoder 起始 token（目标语言）
decoder_start_token_id = tokenizer.get_lang_id("zh")
```

在 ONNX 推理阶段：

- **Encoder ONNX** 接收：`input_ids` + `attention_mask`
- **Decoder ONNX** 接收：
  - `encoder_hidden_states`（来自 encoder 输出）
  - `encoder_attention_mask`（同 attention_mask）
  - `decoder_input_ids`：以 `decoder_start_token_id` 为首 token，后续增量生成

---

## 2. 所需文件

从 HuggingFace 下载或导出后，M2M100 相关文件至少包括：

- `tokenizer.json`（或 `tokenizer_config.json`）
- `sentencepiece.model`（M2M100 的子词模型）
- `config.json`（模型配置，包含 vocab_size、lang_id 等）

建议存放结构：

```text
core/engine/models/nmt/m2m100-en-zh/
  ├─ encoder.onnx
  ├─ decoder.onnx
  ├─ tokenizer.json
  ├─ sentencepiece.model
  └─ config.json
```

Rust / TS 侧只需要知道：
- tokenizer 的目录路径
- 支持的语言代码：`"en"`、`"zh"`

---

## 3. 编码流程（Encoder 侧）

以 **英文 → 中文**（en-zh）为例：

### 3.1 Python 参考实现

```python
tokenizer.src_lang = "en"
enc = tokenizer(source_text, return_tensors="pt", padding=True, truncation=True)

input_ids = enc["input_ids"]           # LongTensor [batch, src_seq]
attention_mask = enc["attention_mask"] # LongTensor [batch, src_seq]
```

需要注意：

- `src_lang` 必须在编码前设置正确，否则语言 token 会不对。
- `padding=True` 时，会自动补 pad token 并生成对应的 attention_mask。
- `truncation=True` 保证不会超过模型最大长度（一般 512 或 1024）。

### 3.2 Rust / TypeScript 侧建议实现方式

有两种方案：

#### 方案 A：通过 Python 服务/模块调用 tokenizer

优点：实现成本低，100% 与 HuggingFace 行为一致。

步骤：

1. 在后端增加一个 **Tokenizer Service**（Python 实现）：
   - 暴露简单 HTTP / gRPC / FFI 接口：
     - 输入：`{ text: string, src_lang: "en" | "zh" }`
     - 输出：`{ input_ids: List[int], attention_mask: List[int] }`
2. Rust / TS 调用该服务获得 `input_ids` 和 `attention_mask`，再喂给 ONNX encoder。

#### 方案 B：在 Rust 使用 HF tokenizers（推荐长期方案）

利用 HuggingFace **tokenizers** 库（Rust 实现），加载 `tokenizer.json`：

1. 在构建时将 `tokenizer.json` 打包到资源目录。
2. 使用 `tokenizers` crate：

   ```rust
   use tokenizers::Tokenizer;

   let mut tokenizer = Tokenizer::from_file("tokenizer.json")?;
   // 设置 src_lang 对 M2M100 来说是通过特殊 token / pretokenizer 完成，
   // 可参考 tokenizer.json 中的特殊 token 配置。
   ```

3. 根据 M2M100 的特殊 token 实现：
   - 在输入文本前添加语言 token，例如：`"<en> " + text`。
   - 或使用 tokenizer 提供的 special token API。

> 由于 M2M100Tokenizer 在 Python 中封装了语言 token 插入逻辑，Rust 侧若完全要复刻，需要解析 `tokenizer.json` 中的 `special_tokens_map` 与 `lang_token_id` 映射。短期仍建议采用 **方案 A**。

---

## 4. 解码流程（Decoder 侧）

Decoder 生成过程使用“自回归 + KV Cache”模式，与当前 Marian 实现类似。

### 4.1 初始 Step（Step 0）

- `decoder_input_ids` = `[decoder_start_token_id]`
  - 对 EN→ZH：`decoder_start_token_id = tokenizer.get_lang_id("zh")`
  - 对 ZH→EN：`decoder_start_token_id = tokenizer.get_lang_id("en")`

- 初始 KV cache：全 0 tensor（由 Rust / NMT 层构造）
- `use_cache_branch = False`（或 0）

### 4.2 后续 Step（Step 1+）

- `decoder_input_ids` = `[上一步预测出的 token_id]`（仅 1 个 token）
- KV cache：使用上一步 decoder ONNX 输出的 `present.*` 作为 `past_key_values.*`
- `use_cache_branch = True`（或 1）

### 4.3 终止条件

常见终止策略：

- 生成到 `<eos>`（EOS token）时停止；或者
- 超过最大长度（例如 128 或 256）；
- 检测到连续多次重复 token 或重复片段时提前终止（可选）。

### 4.4 解码文本

ONNX decoder 只输出 `logits` 和 `present.*`（KV cache），不包含文本。

为了得到最终文本：

1. 对每一步 `logits[:, -1, :]` 做 `argmax`（或 `top-k / nucleus sampling`），得到 `next_token_id`；
2. 将所有生成 token id（去掉 BOS / 语言 token / EOS 部分）组成一个 token id 序列；
3. 使用 tokenizer 的 `decode` 方法还原文本：

   ```python
   generated_ids = [lang_token_id, ..., eos_id]
   text = tokenizer.decode(generated_ids, skip_special_tokens=True)
   ```

Rust / TS 侧若不实现完整解码逻辑，也可以：

- 仅在 Rust/TS 做 `argmax` 计算下一步 token；
- 将完整的 token id 序列传回 Python，由 Python 的 tokenizer 负责最终 decode。

---

## 5. EN↔ZH 方向支持

### 5.1 EN → ZH

- `tokenizer.src_lang = "en"`
- Encoder：编码英文
- Decoder：
  - `decoder_start_token_id = tokenizer.get_lang_id("zh")`
  - 最终解码时，语言 token `"zh"` 会自动处理 / 被 `skip_special_tokens` 过滤

### 5.2 ZH → EN

- `tokenizer.src_lang = "zh"`
- Encoder：编码中文
- Decoder：
  - `decoder_start_token_id = tokenizer.get_lang_id("en")`

### 5.3 配置建议

在项目配置中增加：

```toml
[nmt.m2m100.en_zh]
src_lang = "en"
tgt_lang = "zh"

[nmt.m2m100.zh_en]
src_lang = "zh"
tgt_lang = "en"
```

由高层根据 direction 选择对应配置，保证：

- ASR 输出语言 → 对应 `src_lang`  
- NMT 输出语言 → 对应 `tgt_lang`  
- TTS 声库语言与 `tgt_lang` 对应

---

## 6. 与现有 Marian Tokenizer 的差异点

| 点位           | Marian                      | M2M100                          |
|----------------|-----------------------------|----------------------------------|
| 模型类型       | BPE (vocab.json + merges)   | SentencePiece + language tokens |
| 多语言支持     | 无（单向/双向）             | 多语言（多 src_lang / tgt_lang） |
| 语言控制方式   | 模型权重决定                | 显式设置 `src_lang` / `lang_id`  |
| Rust 侧复刻难度 | 低                          | 中（建议先通过 Python 调用）     |

短期建议：**tokenizer 与 decode 逻辑尽量在 Python 中完成**，Rust 只负责：

- 调用 ONNX encoder/decoder  
- 处理 KV cache  
- 进行 `argmax` / 采样

---

## 7. 实现 checklist

开发部门在接入 M2M100 时可以按以下清单自检：

- [ ] 能在 Python 中使用 `M2M100Tokenizer` 正确编码 EN / ZH 文本  
- [ ] 能正确设置 `src_lang`，并得到合理的 `input_ids` / `attention_mask`  
- [ ] 能正确获取 `decoder_start_token_id = tokenizer.get_lang_id(tgt_lang)`  
- [ ] Rust / TS 能拿到：
  - [ ] `input_ids`（List[int]）
  - [ ] `attention_mask`（List[int]）
  - [ ] `decoder_start_token_id`  
- [ ] DecoderStep 中正确传入：
  - [ ] encoder_hidden_states  
  - [ ] encoder_attention_mask  
  - [ ] decoder_input_ids（一步一个 token）  
  - [ ] past_key_values（KV cache）  
- [ ] 最终能通过 tokenizer.decode 还原生成文本，并在 S2S 测试中验证：
  - [ ] EN→ZH：听起来是自然的中文；  
  - [ ] ZH→EN：听起来是自然的英文。

---

如需将本指南扩展为 **NLLB Tokenizer 实现指南**，可以复用上述结构，仅替换模型类与语言代码映射。
