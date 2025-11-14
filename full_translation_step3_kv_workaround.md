# full_translation_step3_kv_workaround.md
## 暂时关掉 KV cache，让完整翻译 pipeline 先稳定跑通

> 适用场景：  
> - 你已经按 `kv_cache_fix_step2.md` 和 `full_translation_step3_fix.md` 改好了代码；  
> - `decoder_step()` 能跑，前两步也 OK；  
> - **第 3 步仍然在 ONNX 模型内部的 Reshape 节点报错**（`The dimension with value zero exceeds the dimension size of the input tensor`）；  
> - 初步判断问题来自 **模型导出时 KV cache 分支的形状约束**，不是 Rust 侧简单就能完全修掉的。  

本文件给的是一个 **务实的“先能翻译、再优化性能”方案**：  
暂时 **完全不用 KV cache 分支**，退回到「每一步都用完整历史序列解码」的安全模式，让 `test_full_translation_pipeline` 能稳定跑完。

---

## 1. 当前状态回顾

你现在的实现大致是：

- `MarianNmtOnnx`：
  - 内部持有 `encoder_session`、`decoder_session`（ORT 1.16.3）。
  - 暴露 `translate()` 方法，做完整句子解码。

- `DecoderState`：
  - `input_ids: Vec<i64>`：当前这一步要喂进 decoder 的 token。  
  - `generated_ids: Vec<i64>`：到目前为止生成的完整翻译序列。  
  - `kv_cache: Option<Vec<[Value<'static>; 4]>>`：每层 4 份 KV（decoder K/V + encoder K/V）。  
  - `use_cache_branch: bool`：控制是否走 KV 分支。

- `decoder_step()`：
  - 使用 `state.input_ids` 构建形状 `[1, 1]` 的 `input_ids`。  
  - 正确构建 `encoder_attention_mask`、`encoder_hidden_states`。  
  - 如果 `state.kv_cache` 为 `Some`：
    - 把 KV 依次按输入顺序 `past_key_values.*` 推入 `input_values`。  
    - `use_cache_branch = true`。  
  - 如果 `state.kv_cache` 为空：
    - 调用 `build_initial_kv_values()` 构造占位 KV。  
    - `use_cache_branch = false`。  
  - 最终调用 `session.run(inputs)` 得到 `logits` 和新的 `present.*`。

- `translate()`：
  - 第 1 步：`input_ids = [decoder_start_token_id]`，`use_cache_branch = false`。  
  - 之后每步：
    - 从 `logits` 选出 `next_id`。  
    - 更新 `generated_ids`。  
    - **只把新 token 放到下一步的 `input_ids = vec![next_id]`**。  
    - `use_cache_branch = true`。

从调试输出来看：

- Step 0: `input_ids shape [1, 1]`  
- Step 1: `input_ids shape [1, 1]`  
- Step 2: `input_ids shape [1, 1]`  

说明 **decoder 输入这部分是正确的**，问题更可能出在：

- 模型在 KV cache 分支里对 `past_decoder_sequence_length` / `decoder_sequence_length` 做了比较严格的 Assumption；  
- 或导出时某个中间 Reshape 对 KV 分支的维度有“写死”的行为，第三步开始累积就炸了。

这种错误 **已经超出纯 Rust 侧能够“瞎猜”修好的范围**，一般要么：

1. 回到 Python / PyTorch 那一层，重新导出 `decoder.onnx`（KV 输入设为可选/nullable，或者关闭 KV 分支）；  
2. 或者在工程上暂时 **完全不用 KV cache 分支**，改走无 cache 的稳定路径。

---

## 2. Workaround 总体思路：禁用 KV cache，改用“全序列无 cache 解码”

### 2.1 思路概述

- 不再尝试走 `use_cache_branch = true` 的 KV 分支。  
- **每一步都设置 `use_cache_branch = false`**，让模型走“不使用缓存”的路径。  
- 每一步 **把完整的 decoder 序列** 作为 `input_ids` 喂入（而不是只有最新的那个 token）。  
- 不再从输出里保存/回传 `present.*` 作为下一轮的 `kv_cache`。  
- 换句话说：**每一步都“从头解码”一次完整序列，只取最后一个位置的 logits 作为本轮输出**。

代价：

- 性能会差一些（复杂度从 O(T) 变成 O(T²) 量级）。  
- 但好处是：
  - 不需要依赖模型 KV 分支的内部实现和维度约定；  
  - 只要 encoder 和 decoder 本身是正确的，这个方案 **几乎一定能跑通**。

---

## 3. 具体修改建议

下面的修改建议尽量用「语义 + 操作步骤」来描述，避免写死你代码中的具体函数名/字段名。你可以对照自己的实现做调整。

### 3.1 调整 DecoderState 的语义

保留结构体定义不变，只是在使用上做约定：

- `generated_ids`: 始终保存「到当前为止的完整输出序列」。
- `input_ids`: 每一步用于喂给 decoder 的序列：  
  - **Workaround 中：`input_ids = generated_ids.clone()`**（即每步都是全历史）。
- `kv_cache`: 在这个 workaround 里不再真正使用，可以保留字段但置为 `None`。  
- `use_cache_branch`: 在整个 `translate()` 生命周期内 **始终为 false**。

> 为了简单，你也可以在 `DecoderState` 里暂时去掉 `kv_cache` / `use_cache_branch`，但不删字段对后续改回 KV 分支更方便。

### 3.2 修改 translate() 解码循环

在 `MarianNmtOnnx::translate()` 里：

1. **初始化**

```rust
let mut state = DecoderState {
    input_ids: vec![self.decoder_start_token_id],
    generated_ids: vec![self.decoder_start_token_id],
    kv_cache: None,
    use_cache_branch: false,
};
```

> 注意：这里 `generated_ids` 一开始就包含了 BOS / decoder_start_token。

2. **每一步循环解码**（主逻辑）：

伪代码层级的逻辑：

```rust
for step in 0..max_steps {
    // 1. 每一步都用完整历史序列解码
    state.input_ids = state.generated_ids.clone();
    state.use_cache_branch = false; // 关键：每一步都禁用 KV 分支
    state.kv_cache = None;          // 关键：不携带历史 KV

    let (logits, _new_state) = self.decoder_step(
        &encoder_hidden_states,
        &encoder_attention_mask,
        state,
    )?;

    // 2. 从 logits 里取最后一个位置的分布
    let next_id = self.select_next_token_from_last_position(&logits);

    // 3. 更新 generated_ids
    state.generated_ids.push(next_id);

    // 4. 终止条件
    if next_id == self.eos_token_id {
        break;
    }
}
```

落地时请注意：

- `decoder_step()` 的签名可能是 `fn decoder_step(&self, enc_hid, enc_mask, state: DecoderState) -> Result<(Array3<f32>, DecoderState)>` 之类；
- 既然我们不再使用 KV cache，就可以：
  - 让 `decoder_step()` 内部 **忽略传入的 `kv_cache`**；
  - 返回的 `DecoderState` 也不再依赖 `present.*` 去更新 `kv_cache`。

如果你目前的 `decoder_step()` 返回的是 `(logits, DecoderState)`，可以在实现里：

- 直接把传入的 `state.generated_ids` 原样塞回去；  
- 不再对 `kv_cache` / `use_cache_branch` 做任何修改。

### 3.3 修改 decoder_step() 的输入构造

现在我们要遵循下面这套约定：

1. `state.input_ids` 每一步是「完整历史序列」。  
2. `state.use_cache_branch` 永远为 `false`。  
3. `state.kv_cache` 始终是 `None`。

因此在 `decoder_step()` 里：

1. 构造 decoder 输入：

```rust
let seq_len = state.input_ids.len();

let input_ids_arr = Array2::<i64>::from_shape_vec(
    (1, seq_len),
    state.input_ids.clone(),
)?;

// 如果模型需要 decoder attention mask：
let decoder_attention_mask = Array2::<i64>::from_shape_vec(
    (1, seq_len),
    vec![1; seq_len],
)?;
```

2. 构造 `encoder_attention_mask` / `encoder_hidden_states` ——保持你之前的写法，不需要动。

3. KV 部分的处理：

- 由于这套 workaround **完全不使用 KV 分支**，你可以：

  - 方案 A（最简单）：**不再把任何 KV 相关输入塞进 `input_values`**，只传：  
    - `encoder_attention_mask`  
    - `input_ids`  
    - `encoder_hidden_states`  
    - `use_cache_branch = false`  

  - 如果模型强制要求有 `past_key_values.*` 输入（不传会报缺少输入）：  
    - 方案 B：继续用 `build_initial_kv_values()` 构造一组「零 KV」，但在 ONNX 里只要 `use_cache_branch = false`，这些 KV 应该不会真正影响计算。
    - 关键：**无论第几步调用，都当作“第一步无历史”的情形来跑**。

4. `use_cache_branch`：

- 始终构造一个布尔 tensor：

```rust
let use_cache_input = Array1::<bool>::from_vec(vec![false]);
```

- 注意：你之前已经把它从 `f32` 改成 `bool`，保持即可。

5. 从输出中只关心 `logits`：

- 不再解析 `present.*` 作为下一轮的 KV cache。  
- 可以完全忽略 `outputs[1..]`，只用 `outputs[0]` 做下一个 token 的选择。

---

## 4. 如何从 logits 中取“最后一个位置”的分布

既然每一步都用「完整历史序列」解码，那么：

- ONNX 输出 `logits` 的形状是：`[batch=1, seq_len, vocab_size]`。  
- 我们需要的是 **当前步骤的“最后一个 token”的 logits**：

```rust
// 假设 logits_arr: Array3<f32> [1, seq_len, vocab_size]
let seq_len = logits_arr.shape()[1];
let last_step_logits = logits_arr.index_axis(Axis(1), seq_len - 1); // shape: [1, vocab]
```

然后在 `select_next_token_from_last_position()` 中：

- 对 `last_step_logits` 做 argmax（或 sampling），得到 `next_id`。  
- 这部分逻辑你已经有了，只要确认取的是「最后一维度」即可。

---

## 5. 预期测试结果

应用这个 workaround 后：

- `test_full_translation_pipeline` 的行为变成：  
  - 每一步都重新把「目前所有生成的 token」喂进 decoder；  
  - decoder 不再依赖 KV cache，`use_cache_branch` 永远是 `false`；  
  - 不再出现第三步的 Reshape 错误。

- 性能：  
  - 对于短句（比如 demo / 测试阶段），几乎没什么体感差别；  
  - 对于将来长句或实时场景，后续再单独优化 KV 分支。

- 功能上：  
  - encoder → decoder → argmax → 拼接 tokens → detokenize 的整体「真实翻译」链路是打通的。

---

## 6. 后续可以做的事（不影响当前 workaround）

1. **重新导出 ONNX decoder 模型（长期修复）**  
   - 在 Python / PyTorch 侧确保：  
     - KV 输入设为「可选」；  
     - Reshape / 形状推断逻辑对多步（step ≥ 3）稳健。  

2. **重新启用 KV cache（性能优化）**  
   - 在 ONNX 模型确认无误之后：  
     - 再恢复现在的 `DecoderState.kv_cache` + `use_cache_branch = true` 方案；  
     - 把 `input_ids` 改回「每一步只传最后一个 token」。  

---

## 7. TL;DR

> 你现在的 Reshape 错误非常像「decoder 的 KV 分支导出不干净」导致的内部形状冲突，纯 Rust 这边已经 fix 到极限。  
>  
> **短期务实方案：**  
> - 暂时 **完全禁用 KV cache**，`use_cache_branch = false`；  
> - 每一步都把完整的 `generated_ids` 作为 `input_ids` 喂给 decoder；  
> - 不再从 `present.*` 恢复 KV cache，`kv_cache` 可以一直为 `None`；  
> - 从 `logits[0, last_pos, :]` 取分布做解码。  
>  
> 这样你就能先拿到一个「可以真实翻译的端到端流程」，后面再慢慢迭代模型导出和 KV 性能优化。
