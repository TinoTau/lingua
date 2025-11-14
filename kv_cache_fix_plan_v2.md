# Marian NMT ONNX KV Cache 修复方案（基于 `ort = 1.16.3`）

> 目标：在 **不破坏现有产品架构** 的前提下，修复 KV cache 相关的内存问题，让 `MarianNmtOnnx` 可以稳定地做多步解码，并最终用于真实翻译。

本说明只做两件事：

1. 说明 **当前问题的根源**。
2. 给出一套 **具体可实现的改动方案（方案 1）**，包含需要改的结构体、函数以及推荐的代码规范。

---

## 1. 当前问题回顾

你在当前分支上已经完成了：

- 升级到 `ort = 1.16.3`
- Encoder / Decoder 模型都可以成功加载
- Encoder 推理成功，输出形状为：`[1, 3, 512]`
- Decoder 单步推理在「不带 KV cache」或「只看 logits」的情况下是能跑的

但在 **第二步开始使用 KV cache 时** 出现：

- `try_extract_tensor` 在从 KV `Value` 中提取数据时，数据指针为 `null`
- 之前的版本里还出现过 rank 不匹配（预期 4 维，实际 1 维）

总结一下：

> **问题并不在模型本身，而在「怎么管理 KV cache」这层 Rust 封装，以及 `Value` 的生命周期 / 所有权上。**

---

## 2. 设计目标（方案 1）

在不破坏你现在整体架构的前提下，我们采用：

- **Encoder**：保持现在的实现（`run_encoder(&self, input_ids: &[i64]) -> Result<(Array3<f32>, Array2<i64>)>`）
- **Decoder**：
  - 只把 **logits** 解成 `Array3<f32>`
  - **不再把 KV cache 从 `Value` 中解出来做手工 `Array4` 管理**
  - **每一步都「消耗旧 KV」并「接收新 KV」**，通过 `Value` 所有权移动解决 `Clone` / 生命周期问题

也就是说：

> 把 KV cache 当成 **完全黑盒的 `Value<'static>`** 来回传递，Rust 只负责「搬运」，不负责「解析」。

这样可以：

- 避免 `try_extract_tensor` 在 KV 上触发 `unsafe precondition violated`
- 避免手动构造「空 KV」导致维度不对、指针无效等问题
- 让 Decoder 迭代逻辑尽量简单清晰

---

## 3. 结构体与状态约定

### 3.1 `DecoderState` 结构体推荐设计

文件路径：`core/engine/src/nmt_incremental/mod.rs`

在 `MarianNmtOnnx` 附近新增一个内部结构体，用于解码状态：

```rust
use ort::value::Value;

/// 单句翻译时 Decoder 的状态
struct DecoderState {
    /// 当前 decoder 的 input_ids（最后一个 token 是本步要解码的）
    pub input_ids: Vec<i64>,

    /// 已经生成的 token IDs（不包括起始的 decoder_start_token_id）
    pub generated_ids: Vec<i64>,

    /// 上一步返回的 KV cache（present.*）
    /// - 每一层有 4 个 Value：decoder.key, decoder.value, encoder.key, encoder.value
    /// - `None` 代表第一步（没有历史 KV）
    pub kv_cache: Option<Vec<[Value<'static>; 4]>>,

    /// 控制 `use_cache_branch` 输入
    pub use_cache_branch: bool,
}
```

> **注意**：这里 KV cache 使用 `Vec<[Value<'static>; 4]>`，而不是 `Vec<Value>` 或 `Vec<Array4<f32>>`，目的是：  
> - 保证每一层的 4 个 KV 总是绑在一起  
> - 所有权由 `DecoderState` 完整掌握，不依赖 `Clone` 或临时引用

### 3.2 在 `MarianNmtOnnx` 中约定常量

在你的 `MarianNmtOnnx` 结构体实现里，建议加上一些简单的常量：

```rust
impl MarianNmtOnnx {
    const NUM_LAYERS: usize = 6;
    const NUM_HEADS: usize = 8;
    const HEAD_DIM: usize = 64;
}
```

虽然我们对 KV cache 不再自己建 `Array4`，但这些常量在 Debug 输出 / 校验时很有用。

---

## 4. Decoder 单步推理 API 约定

### 4.1 函数签名建议

同样在 `mod.rs` 中，为 `MarianNmtOnnx` 新增一个内部方法：

```rust
impl MarianNmtOnnx {
    /// 执行 decoder 的单次步进
    ///
    /// - 输入：
    ///   - encoder_hidden_states: [1, encoder_seq_len, hidden_dim]
    ///   - encoder_attention_mask: [1, encoder_seq_len]
    ///   - state: 包含当前 decoder_input_ids / 上一步 KV cache
    /// - 输出：
    ///   - (logits_last_step, next_state)
    fn decoder_step(
        &self,
        encoder_hidden_states: &Array3<f32>,
        encoder_attention_mask: &Array2<i64>,
        mut state: DecoderState,
    ) -> anyhow::Result<(Array1<f32>, DecoderState)> {
        // 具体实现见下节
    }
}
```

这里的设计要点：

1. **`DecoderState` 通过值传入、再返回**  
   - 利用 Rust 所有权，让「旧 KV cache」可以在本函数内被 `take()` 掉并移动到 `inputs`
   - 再把新的 KV cache 打包进 `DecoderState` 返回

2. **只返回当前步最后一个 token 的 logits**  
   - `Array1<f32>` 对应 `[vocab_size]`  
   - 简化采样逻辑（greedy / top-k 等你可以后面慢慢加）

---

## 5. decoder_step 实现要点（核心方案）

> 下面是 **关键逻辑说明**，你在本地写代码时可以直接照着改，注意对齐你当前的模块路径和类型别名。

### 5.1 构造输入张量 → `Value`

伪代码风格描述（但 API 和类型都是真实的 `ort 1.16.3`）：

```rust
use ndarray::{Array1, Array2, Array3};
use ort::value::Value;

// 1. 准备 decoder input_ids: [1, cur_len]
let batch_size = 1usize;
let cur_len = state.input_ids.len();
let decoder_input_ids = Array2::<i64>::from_shape_vec(
    (batch_size, cur_len),
    state.input_ids.clone(),
)?;

// 2. use_cache_branch: [1]，注意类型是 Bool / i64 / f32 要与你当前模型匹配
// 你提到已经修成 Bool，这里假设是 Bool：
let use_cache_array = Array1::<bool>::from_vec(vec![state.use_cache_branch]);

// 3. encoder_hidden_states 已经是 Array3<f32>
// 4. encoder_attention_mask 已经是 Array2<i64>
// 统一转换为 Value：
macro_rules! array_to_value {
    ($arr:expr) => {{
        let arr_dyn = $arr.into_dyn();
        let shape: Vec<i64> = arr_dyn.shape().iter().map(|&d| d as i64).collect();
        let data: Vec<_> = arr_dyn.iter().cloned().collect();
        Value::from_array((shape, data))?
    }};
}

let input_ids_value = array_to_value!(decoder_input_ids);
let encoder_states_value = array_to_value!(encoder_hidden_states.clone());
let encoder_mask_value = array_to_value!(encoder_attention_mask.clone());
let use_cache_value = array_to_value!(use_cache_array);
```

### 5.2 组织输入顺序（严格按照模型 I/O 顺序）

根据你之前打印出的模型 inputs：

1. `encoder_attention_mask`
2. `input_ids`
3. `encoder_hidden_states`
4. `past_key_values.0.decoder.key`
5. `past_key_values.0.decoder.value`
6. `past_key_values.0.encoder.key`
7. `past_key_values.0.encoder.value`
8. ...
27. `use_cache_branch`

> **关键点**：  
> - 第一轮（没有历史 KV）时：**不要构造任何假的 KV tensor**，按模型导出时的约定，要么模型内自己处理（根据 `use_cache_branch=false` 忽略分支），要么你在 `export` 时就把这些输入设置为可选输入。  
> - 第二轮及以后：**完全用上一轮 `present.*` 的 Value 来填充 `past_key_values.*`**。

推荐做法：

```rust
let mut input_values: Vec<Value<'static>> = Vec::new();

// 1. encoder_attention_mask
input_values.push(encoder_mask_value);
// 2. input_ids
input_values.push(input_ids_value);
// 3. encoder_hidden_states
input_values.push(encoder_states_value);

// 4. KV cache：只有当有历史 KV 时才追加
if let Some(prev_kv) = state.kv_cache.take() {
    // prev_kv: Vec<[Value<'static>; 4]>
    for layer in 0..MarianNmtOnnx::NUM_LAYERS {
        let [dec_k, dec_v, enc_k, enc_v] = prev_kv[layer];
        input_values.push(dec_k);
        input_values.push(dec_v);
        input_values.push(enc_k);
        input_values.push(enc_v);
    }
} else {
    // 第一轮：模型如果要求这些输入是必填，就需要你在导出 ONNX 时
    // 把这些输入改成可选并在图里根据 `use_cache_branch` 决定是否访问。
    // 否则就不要在这里自己造「空 KV」。
}

// 5. use_cache_branch
input_values.push(use_cache_value);
```

> **注意**：这里使用了 `state.kv_cache.take()`：  
> - 这样上一轮的 KV 会被「消费掉」，所有权完全转移到 `input_values`。  
> - 运行结束后，我们再从输出中构造新的 KV 赋给 `state.kv_cache = Some(next_kv)`。  
> - **全程没有 `Clone`，也没有对 KV 做 `try_extract_tensor`。**

### 5.3 调用 `session.run`

```rust
let decoder_session = self.decoder_session.lock().unwrap();
let outputs: Vec<Value<'static>> = decoder_session.run(input_values)?;
```

输出顺序通常是：

1. `logits`
2. `present.0.decoder.key`
3. `present.0.decoder.value`
4. `present.0.encoder.key`
5. `present.0.encoder.value`
6. `present.1.decoder.key`
7. ...

你可以用一次性 `println!` / 断言来校验顺序是否与你模型的实际 `outputs` 顺序一致。

### 5.4 从输出中提取 logits + 新 KV

**logits** 是唯一需要转回 `ndarray` 的：

```rust
use ort::tensor::OrtOwnedTensor;

let logits_value = &outputs[0];
let logits_tensor: OrtOwnedTensor<f32, _> = logits_value.try_extract()?;
let logits_array = logits_tensor.view().to_owned(); // shape: [1, cur_len, vocab_size]

// 取最后一个 step 的 logits: [vocab_size]
let shape = logits_array.shape().to_vec();
let vocab_size = shape[2];
let last_step_logits = logits_array
    .index_axis(ndarray::Axis(1), shape[1] - 1)
    .to_owned(); // Array1<f32>，长度 vocab_size
```

**KV cache** 只搬运，不解析：

```rust
let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(MarianNmtOnnx::NUM_LAYERS);
let mut idx = 1; // 从 outputs[1] 开始是 present.*

// 这里推荐把 outputs 作为 `Vec<Value>` by value 获取，而不是 `&[Value]`，
// 然后使用 into_iter() 完整“吃掉”整个 Vec，避免 Clone。
let mut iter = outputs.into_iter();
let logits_value = iter.next().unwrap(); // 已经在前面处理过 logits

for _layer in 0..MarianNmtOnnx::NUM_LAYERS {
    let dec_k = iter.next().expect("missing present.*.decoder.key");
    let dec_v = iter.next().expect("missing present.*.decoder.value");
    let enc_k = iter.next().expect("missing present.*.encoder.key");
    let enc_v = iter.next().expect("missing present.*.encoder.value");
    next_kv.push([dec_k, dec_v, enc_k, enc_v]);
}
```

最后，更新 `state`：

```rust
state.kv_cache = Some(next_kv);
state.use_cache_branch = true; // 后续步骤都走 cache 分支

Ok((last_step_logits, state))
```

---

## 6. translate 整体流程串联

在 `MarianNmtOnnx::translate(&self, source_text: &str)` 中，大致流程如下：

1. `tokenizer.encode` 得到 `source_ids`
2. `run_encoder(&self, &source_ids)` 得到 `(encoder_hidden_states, encoder_attention_mask)`
3. 初始化 `DecoderState`：

```rust
let mut state = DecoderState {
    input_ids: vec![self.decoder_start_token_id],
    generated_ids: Vec::new(),
    kv_cache: None,
    use_cache_branch: false,
};
```

4. 进入解码循环（伪代码）：

```rust
for _step in 0..self.max_length {
    let (logits, next_state) =
        self.decoder_step(&encoder_hidden_states, &encoder_attention_mask, state)?;

    state = next_state;

    // 选一个 token（先用 greedy：argmax）
    let (next_token_id, _score) = logits
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap();

    let next_token_id = next_token_id as i64;

    if next_token_id == self.eos_token_id {
        break;
    }

    state.generated_ids.push(next_token_id);
    state.input_ids.push(next_token_id);
}

// 最后用 tokenizer.decode 把 generated_ids 解回字符串
let translated = self.tokenizer.decode(&state.generated_ids, false);
Ok(translated)
```

> 这部分你之前已经有 `translate()` stub，可以在此基础上替换掉 stub 逻辑，改成真正的迭代推理。

---

## 7. 代码规范建议

为了让这部分代码将来好维护（你自己也好继续改），推荐遵守以下规范：

1. **所有 Session 交互只在 `MarianNmtOnnx` 内部发生**
   - 外部接口只暴露 `translate`, `new_from_dir`, `new_from_language_pair` 等高层 API。
   - 不要在别的模块里直接调用 `Session::run`。

2. **KV cache 全程当黑盒 `Value<'static>` 管理**
   - 不对 KV 做任何 `try_extract_tensor` / `Array4` 操作。
   - 所有权通过 `Option<Vec<[Value; 4]>>` + `take()` + `into_iter()` 管理。

3. **所有 ONNX I/O 顺序一定要靠注释锁定**
   - 在 `decoder_step` 顶部写清楚：输入顺序 / 输出顺序与模型的对应关系。
   - 如果将来重新导出模型，第一时间更新这里的注释和索引。

4. **日志统一用 `println!` + 前缀**
   - 例如：`[NMT][decoder_step] logits shape: ...`
   - 方便你在多线程、多模块日志里快速过滤 NMT 相关输出。

---

## 8. 下一步建议

1. **先只在测试代码里调用 `MarianNmtOnnx::translate("Hello world")`**
   - 确认整条链路「不会崩」「返回的 token 列表合理」。

2. **再接入你现有的 `NmtIncremental` trait**
   - 在 `MarianNmtOnnx` 里实现真正的 `NmtIncremental::translate`，从 `PartialTranscript` 里取文本。

3. **最后再考虑多语言 / 自动识别**
   - 把 `LanguagePair` + `Tokenizer` 逻辑串起来，根据源语种/目标语种动态选择模型目录。

如果你愿意，我可以在你完成上述方案 1 的基础改动后，再帮你一起 review 一遍 GitHub 上的 `mod.rs`，专门看：

- `decoder_step` 实现是否正确使用了 `Value` 所有权
- ONNX I/O 顺序是否和模型一致
- `translate` 的循环逻辑是否易读、易测
