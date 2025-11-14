# full_translation_step3_fix.md  
## Fixing third-step shape error in full translation pipeline  
### Root cause: decoder input length vs KV cache semantics (Marian ONNX + ORT 1.16.3)

---

## 1. 当前现象总结

你当前的实现（基于 `kv_cache_fix_step2.md`）已经做到：

1. **KV cache 按 Value 黑盒传递**：
   - 使用 `DecoderState.kv_cache: Option<Vec<[Value<'static>; 4]>>` 保存所有层的 KV。
   - `decoder_step()` 每一轮都把 24 个 KV 输入（6 层 × 4 个）按顺序传入。

2. **第一步和第二步推理正常**：
   - 第一轮使用 `build_initial_kv_values()` 生成占位 KV（zeros）。
   - 第二轮使用第一轮输出的 `present.*` 作为新的 KV cache。
   - logit 提取逻辑（`try_extract_tensor` → `view()` → `into_dimensionality()` → `to_owned()`）已经适配 ORT 1.16.3。

3. **第三步报错**：
   - ORT 抛出错误：  
     > `The dimension with value zero exceeds the dimension size of the input tensor`  
   - 报错发生在模型内部 `Reshape` 节点，说明：
     - 模型在根据 `past_decoder_sequence_length`、`decoder_sequence_length` 等变量重建形状时，发现实际 tensor 的某个维度与期望的不一致。
     - 这通常是 **输入的 sequence length / KV cache sequence length / mask 长度 之间不一致** 导致的。

---

## 2. 根本原因推断（结合 Marian Decoder 的结构）

根据你打印的 ONNX I/O 签名：

- 输入中有：  
  - `input_ids`（decoder 端 token 序列）  
  - `encoder_hidden_states`  
  - `encoder_attention_mask`  
  - 一整套 `past_key_values.*`  
  - `use_cache_branch`（切换是否使用缓存分支）
- 输出中有：  
  - `logits`  
  - 一整套 `present.*`（下一轮的 KV）

对 Marian / Transformer-decoder 的典型约定是：

1. **第一步（没有历史 KV）**  
   - `input_ids` = 整个 decoder 序列（一般是 BOS/start token，长度 = 1）。  
   - `use_cache_branch = false`。  
   - `past_decoder_sequence_length` = 0，`decoder_sequence_length` = 1。  
   - 输出 `present.*` 的形状：  
     `[..., past_decoder_sequence_length + decoder_sequence_length, head_dim]`  
     ⇒ 第一轮后，KV 的 decoder 维度长度 = 1。

2. **后续步骤（使用 KV cache）**  
   - KV cache 已经保存了之前所有步骤的历史 states。  
   - `input_ids` **不再是完整历史序列**，而是 **“本轮新增的最后一个 token”**（长度 = 1）。  
   - `use_cache_branch = true`。  
   - `past_decoder_sequence_length` 来自 KV 的第三维长度。  
   - 本轮输出 `present.*` 的 decoder 维度长度 = `past_decoder_sequence_length + 1`。

> **关键点：**  
> 当 `use_cache_branch = true` 时：  
> - 模型假设 **你已经把旧的 decoder 历史全部放进 KV cache**；  
> - 新传进来的 `input_ids` **只表示“新增的那一步”**。  
> - 如果你仍然把“完整历史序列”放进 `input_ids`，内部的 `past_decoder_sequence_length + decoder_sequence_length` 会和真实维度不符，导致 Reshape 报错。  

**结合你现在的现象：**  

- 第 1 步：序列长度很短 → 再怎么错都能勉强过。  
- 第 2 步：运气好、形状还能兼容。  
- 到第 3 步：累积错误导致 `past_len + new_len` 与真实 tensor 的 size 不符 → 出现「The dimension with value zero exceeds the dimension size of the input tensor」这样的 Reshape 报错。  

结论：

> 你的 **KV cache 黑盒逻辑是对的**，问题在于：  
> **后续步骤仍然使用“完整 decoder 历史序列”作为 `input_ids` 和 mask，而不是“每轮只传最后一个 token”。**

---

## 3. 修复思路（核心规则）

### 3.1 DecoderState 的语义约束

保持你现有的 `DecoderState` 定义，在语义上约定：

- `generated_ids: Vec<i64>`：保存 **到当前为止的完整翻译结果**。  
- `input_ids: Vec<i64>`：只用于 **本次调用 `decoder_step` 的 decoder 输入**：
  - 第一轮：`input_ids = [decoder_start_token_id]`  
  - 第二轮起：`input_ids = [本轮新增 token（上轮 argmax 出来的 token）]`

### 3.2 translate() 解码循环的规则

在 `MarianNmtOnnx::translate()` 中的伪流程应该是：

1. **初始化**

```rust
let mut state = DecoderState {
    input_ids: vec![self.decoder_start_token_id],
    generated_ids: Vec::new(),
    kv_cache: None,
    use_cache_branch: false,
};
```

2. **循环解码**（伪代码逻辑）

```rust
for step in 0..max_steps {
    let (logits, new_state) = self.decoder_step(
        &encoder_hidden_states,
        &encoder_attention_mask,
        state,
    )?;

    // 1. 从 logits 取出当前步骤的下一个 token（如 argmax）
    let next_id = self.select_next_token(&logits);

    // 2. 更新 generated_ids：完整历史
    let mut generated = new_state.generated_ids;
    generated.push(next_id);

    // 3. 准备下一轮的 state：
    state = DecoderState {
        input_ids: vec![next_id],       // ← 下一轮只喂“当前 token”
        generated_ids: generated,
        kv_cache: new_state.kv_cache,   // 使用当前轮的 present.*
        use_cache_branch: true,         // 从第二轮开始一直为 true
    };

    if next_id == self.eos_token_id {
        break;
    }
}
```

**务必避免：**

```rust
// ❌ 不要每轮都这样：
state.input_ids = state.generated_ids.clone();
```

这会让 `decoder_sequence_length` 逐轮变成 1, 2, 3, 4, ...  
而模型在 cache 分支下假定——

- 这些历史都已经在 KV 里；  
- `decoder_sequence_length` 应该恒为 1。  

两者矛盾就会在某一轮（通常是第 3 步左右）直接触发 Reshape 报错。

---

## 4. 对 decoder_step 的输入规范

在你现有的 `decoder_step()` 实现基础上，保证以下约束：

### 4.1 input_ids 张量

- 由 `state.input_ids` 构造：

```rust
// batch = 1, seq_len = state.input_ids.len()（规范后永远等于 1）
let input_ids_arr = Array2::<i64>::from_shape_vec(
    (1, state.input_ids.len()),
    state.input_ids.clone(),
)?;
```

只要 **translate() 每轮只把单个 token 放进 `state.input_ids`**，这里的 shape 就会一直是 `[1, 1]`。

### 4.2 decoder attention mask

可简单设置为：

```rust
let decoder_attention_mask =
    Array2::<i64>::from_shape_vec((1, state.input_ids.len()), vec![1])?;
```

同理：确保 **和 `input_ids` 的 seq_len 一致（即 1）**。  
如果模型要求单独的 decoder mask，可以按相同模式构造；如果只用 encoder mask，那就无须再多传。

> 重点：**不要尝试把历史长度编码进 mask 里**——历史已经被缓存到了 KV 中。

### 4.3 KV cache 输入（已在 kv_cache_fix_step2.md 里完成）

保持你现在的逻辑：

```rust
let kv_to_use: Vec<[Value<'static>; 4]> =
    if let Some(prev_kv) = state.kv_cache.take() {
        prev_kv
    } else {
        self.build_initial_kv_values(encoder_seq_len)?
    };

for layer in 0..Self::NUM_LAYERS {
    let [dec_k, dec_v, enc_k, enc_v] = kv_to_use[layer];
    input_values.push(dec_k);
    input_values.push(dec_v);
    input_values.push(enc_k);
    input_values.push(enc_v);
}
```

不再对 KV 做任何 `try_extract_tensor`，直接当黑盒 `Value` 使用即可。

---

## 5. 推荐的调试验证步骤

### 5.1 打印每一轮的形状（强烈建议）

在 `decoder_step()` 开头增加：

```rust
println!(
    "[decoder_step] step input_ids_len={}, use_cache_branch={}, has_kv_cache={}",
    state.input_ids.len(),
    state.use_cache_branch,
    state.kv_cache.is_some(),
);
```

在构造 `input_ids` 的位置打印：

```rust
println!(
    "[decoder_step] input_ids shape: {:?}",
    input_ids_arr.shape()
);
```

运行 `test_full_translation_pipeline`，理想日志应该类似：

```text
[decoder_step] step input_ids_len=1, use_cache_branch=false, has_kv_cache=false
[decoder_step] input_ids shape: [1, 1]
...
[decoder_step] step input_ids_len=1, use_cache_branch=true, has_kv_cache=true
[decoder_step] input_ids shape: [1, 1]
...
[decoder_step] step input_ids_len=1, use_cache_branch=true, has_kv_cache=true
[decoder_step] input_ids shape: [1, 1]
...
```

一旦看到某一轮 `input_ids_len` 变成 2、3……，就说明你在 `translate()` 里不小心把整个历史重新喂进去了。

### 5.2 确认 KV 维度持续增长

你之前已经在 `test_marian_decoder_single_step` 里打印过：

```text
Decoder logits shape: [1, 1, 65001]
present.0.decoder.key shape: [1, 8, 2, 64]
```

可以在 full pipeline 的第三步前后，再打印一次 `present.0.decoder.key` 的 shape，确认第三维度从 1 → 2 → 3 连续增长，而模型不再报错。

---

## 6. 最终效果预期

完成上述修改后：

1. **第一步**  
   - 用占位 KV；  
   - `input_ids = [start_token]`；  
   - `use_cache_branch = false`；  
   - 输出 `present.*`（decoder 长度 = 1）。  

2. **第二步**  
   - 使用第一步的 KV cache；  
   - `input_ids = [上一步的 next_token]`；  
   - `use_cache_branch = true`；  
   - 输出 `present.*`（decoder 长度 = 2）。  

3. **第三步及以后**  
   - 持续使用最新的 KV cache；  
   - `input_ids` 每次都只包含 **1 个 token**；  
   - `present.*` decoder 长度逐步递增；  
   - 模型内部的 `Reshape` 不再出现「dimension with value zero exceeds the dimension size of the input tensor」异常。  

即使翻译质量一开始不完美，只要：

- 流程稳定；  
- 不再有 `Reshape` / 内存错误；  
- 可以完整跑完 `test_full_translation_pipeline`；  

就说明 **整个端到端推理链路已经打通**。

---

## 7. TL;DR（一句话版）

> 你已经把 KV cache 部分修好了；现在第三步报错的根本原因是：  
> **在使用 cache 分支时，`input_ids` 仍然是“完整历史”，而不是“每轮只传最后一个 token”。**  
>  
> 修复方式：  
> 在 `translate()` 循环中，每次调用 `decoder_step()` 前，只把 **最新生成的一个 token** 放进 `state.input_ids`，并保持 `decoder_step()` 中 `input_ids` 的 shape 始终为 `[1, 1]`。  
> 如此一来，第三步的 Reshape 错误就会消失，你的 `test_full_translation_pipeline` 就能正常跑完。

