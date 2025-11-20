# KV Cache 优化推荐方案

**鏈€鍚庢洿鏂?*: 2024-12-19

---


**������**: 2024-12-19

---

# KV Cache 优化推荐方案

## 📊 当前状�?

### 问题描述
- **当前模式**: Workaround 模式（完全禁�?KV cache�?
- **性能**: 平均每次翻译耗时 ~650ms（短序列）到 ~2000ms（长序列�?
- **错误**: 启用 KV cache 时，第三步（step 2）会出现 Reshape 错误
- **错误信息**: 
  ```
  Non-zero status code returned while running Reshape node.
  The dimension with value zero exceeds the dimension size of the input tensor.
  ```

### 根本原因分析

1. **模型导出问题**（最可能�?
   - `past_key_values.*` 的形状定义可能不正确
   - 动态轴（dynamic axes）配置可能有问题
   - ONNX IR 版本兼容性问�?

2. **代码实现问题**（次要）:
   - `build_initial_kv_values()` �?`dec_len` 可能不正�?
   - `input_ids` 的形状在第一步和后续步骤不一�?

3. **ort crate 版本问题**（可能性较低）:
   - 当前使用 `ort = 1.16.3`
   - 可能存在已知�?Reshape 操作 bug

---

## 🎯 推荐方案（按优先级）

### 方案 1：修复代码实�?+ 调试（推荐，先尝试）⭐⭐�?

**优先�?*: 🔴 **最�?*  
**预计时间**: 1-2 �? 
**成功�?*: 60-70%

#### 实施步骤

##### 步骤 1.1: 添加详细的调试输�?

�?`decoder_step()` 中添加详细的形状和值输出：

```rust
// �?decoder_step() 开始时
println!("[DEBUG] decoder_step - Step {}", step_number);
println!("  input_ids shape: {:?}", input_ids.shape());
println!("  use_cache_branch: {}", state.use_cache_branch);
println!("  kv_cache present: {}", state.kv_cache.is_some());

if let Some(ref kv) = state.kv_cache {
    println!("  KV cache layers: {}", kv.len());
    for (i, layer_kv) in kv.iter().enumerate() {
        // 尝试获取形状（可能需�?unsafe 或特殊处理）
        println!("  Layer {} KV cache present", i);
    }
}
```

##### 步骤 1.2: 修复 `build_initial_kv_values()` �?`dec_len`

**问题**: 第一步的 `dec_len` 应该�?1（只�?BOS token），但可能被设置�?0

**修复**:
```rust
fn build_initial_kv_values(
    &self,
    encoder_seq_len: usize,
) -> anyhow::Result<Vec<[Value<'static>; 4]>> {
    // 关键修复：dec_len 应该�?1（第一步有 BOS token�?
    let dec_len = 1usize;  // 不是 0�?
    
    // ... 其余代码保持不变
}
```

##### 步骤 1.3: 确保第一步提�?KV cache

**当前问题**: 第一步（`use_cache_branch=false`）时跳过�?KV cache 提取

**修复**:
```rust
} else {
    // 第一步：提取 KV cache，为下一步启用正常模�?
    let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
    for _layer in 0..Self::NUM_LAYERS {
        let dec_k = iter.next().expect("missing present.*.decoder.key");
        let dec_v = iter.next().expect("missing present.*.decoder.value");
        let enc_k = iter.next().expect("missing present.*.encoder.key");
        let enc_v = iter.next().expect("missing present.*.encoder.value");
        
        // 关键：直接使�?clone() �?to_owned() 确保生命周期正确
        next_kv.push([
            dec_k.clone(),  // 或使�?to_owned() 如果可用
            dec_v.clone(),
            enc_k.clone(),
            enc_v.clone(),
        ]);
    }
    state.kv_cache = Some(next_kv);
    state.use_cache_branch = true;  // 下一步启�?KV cache
}
```

##### 步骤 1.4: 验证 `input_ids` 形状一致�?

**确保**:
- 第一步：`input_ids = [BOS]` (长度 1)
- 第二步及以后：`input_ids = [last_token]` (长度 1)

**检查点**:
```rust
// �?translate() �?
let current_state = if state.use_cache_branch && state.kv_cache.is_some() {
    // 正常模式：只输入�?token
    let last_token = state.generated_ids.last().copied()
        .unwrap_or(self.decoder_start_token_id);
    DecoderState {
        input_ids: vec![last_token],  // 关键：长度必须是 1
        // ...
    }
} else {
    // Workaround 模式：使用完整序�?
    DecoderState {
        input_ids: current_generated_ids.clone(),  // 长度 > 1
        // ...
    }
};
```

#### 验收标准

- �?能够成功运行到第三步（step 2�?
- �?没有 Reshape 错误
- �?翻译结果正确
- �?性能提升（短序列 ~100ms，长序列 ~500ms�?

#### 如果失败

如果仍然出现 Reshape 错误，继续尝试方�?2�?

---

### 方案 2：检查并修复模型导出（如果方�?1 失败）⭐⭐⭐�?

**优先�?*: 🟡 **�?*  
**预计时间**: 2-3 �? 
**成功�?*: 80-90%

#### 实施步骤

##### 步骤 2.1: 检查模型导出脚�?

**文件**: `scripts/export_marian_encoder.py`

**检查点**:

1. **动态轴定义**:
```python
dynamic_axes = {
    "input_ids": {0: "batch_size", 1: "sequence_length"},
    "encoder_hidden_states": {0: "batch_size", 1: "encoder_sequence_length"},
    # 关键：past_key_values 的动态轴
    "past_key_values.0.decoder.key": {0: "batch_size", 2: "past_decoder_length"},
    "past_key_values.0.decoder.value": {0: "batch_size", 2: "past_decoder_length"},
    # ...
}
```

2. **opset_version**:
```python
torch.onnx.export(
    # ...
    opset_version=13,  # 确保�?ort 1.16.3 兼容
    # ...
)
```

3. **ONNX IR 版本**:
```python
# 确保导出�?IR version 9（ort 1.16.3 要求�?
# 可能需要显式设�?
```

##### 步骤 2.2: �?Python 中验证模�?

创建测试脚本验证 KV cache �?Python 中是否正常工作：

```python
# scripts/test_marian_decoder_kv.py
import torch
import onnxruntime as ort
import numpy as np

# 加载模型
session = ort.InferenceSession("decoder_model.onnx")

# 第一步：use_cache_branch=False
inputs_step0 = {
    "input_ids": np.array([[65000]], dtype=np.int64),  # BOS
    "encoder_hidden_states": ...,
    "encoder_attention_mask": ...,
    "use_cache_branch": np.array([False], dtype=bool),
    # past_key_values.* 使用初始�?
}
outputs_step0 = session.run(None, inputs_step0)

# 第二步：use_cache_branch=True
inputs_step1 = {
    "input_ids": np.array([[8]], dtype=np.int64),  # �?token
    "encoder_hidden_states": ...,
    "encoder_attention_mask": ...,
    "use_cache_branch": np.array([True], dtype=bool),
    # past_key_values.* 使用 outputs_step0 �?present.*
    "past_key_values.0.decoder.key": outputs_step0["present.0.decoder.key"],
    # ...
}
outputs_step1 = session.run(None, inputs_step1)

# 检查是否有 Reshape 错误
```

##### 步骤 2.3: 修复导出脚本

如果 Python 测试也失败，修复导出脚本�?

1. **调整动态轴定义**
2. **修改 opset_version**
3. **显式设置 ONNX IR 版本**

##### 步骤 2.4: 重新导出模型

```bash
python scripts/export_marian_encoder.py \
    --model_name Helsinki-NLP/opus-mt-en-zh \
    --output_dir core/engine/models/nmt/marian-en-zh \
    --verify
```

#### 验收标准

- �?Python 测试�?KV cache 正常工作
- �?Rust 代码�?KV cache 正常工作
- �?没有 Reshape 错误
- �?性能提升

---

### 方案 3：使用黑�?Value 方式（如果方�?1 �?2 都失败）⭐⭐

**优先�?*: 🟢 **�?*  
**预计时间**: 1 �? 
**成功�?*: 50-60%

#### 实施步骤

**核心思想**: 完全避免提取 KV cache 的内部数据，只传�?`Value` 对象

**当前实现**: 已经在使�?`Value<'static>`，但可能在某些地方仍然尝试提取数�?

**检查点**:
1. 确保 `decoder_step()` 中直接使�?`Value`，不调用 `try_extract_tensor()`
2. 确保 `build_initial_kv_values()` 返回的是 `Value`，不�?`ndarray`
3. 确保所�?KV cache 操作都是 `Value` 级别�?

**参�?*: 之前�?`kv_cache_fix_plan_v2.md` 文档

---

### 方案 4：混合模式（渐进式优化）⭐⭐

**优先�?*: 🟢 **�?*  
**预计时间**: 0.5 �? 
**成功�?*: 100%（但性能提升有限�?

#### 实施步骤

根据序列长度动态选择模式�?

```rust
// �?translate() �?
let use_kv_cache = if state.generated_ids.len() < 10 {
    false  // 短序列：使用 workaround（稳定）
} else {
    true   // 长序列：尝试使用 KV cache（性能优先�?
};
```

**优点**: 
- �?短序列稳�?
- �?长序列快�?
- �?可以逐步迁移

**缺点**:
- ⚠️ 仍然需要修�?KV cache 实现
- ⚠️ 性能提升有限

---

## 📋 推荐执行顺序

### 第一阶段（立即执行）

1. **方案 1：修复代码实�?+ 调试** (1-2 �?
   - 添加详细调试输出
   - 修复 `build_initial_kv_values()` �?`dec_len`
   - 确保第一步提�?KV cache
   - 验证 `input_ids` 形状一致�?

### 第二阶段（如果方�?1 失败�?

2. **方案 2：检查并修复模型导出** (2-3 �?
   - 检查模型导出脚�?
   - �?Python 中验证模�?
   - 修复导出脚本
   - 重新导出模型

### 第三阶段（如果方�?2 也失败）

3. **方案 3：使用黑�?Value 方式** (1 �?
   - 完全避免提取 KV cache 内部数据

### 备选方�?

4. **方案 4：混合模�?* (0.5 �?
   - 如果所有方案都失败，使用混合模式作为过�?

---

## 🔍 调试技�?

### 1. 添加详细的日�?

```rust
// 在关键位置添加日�?
println!("[KV_CACHE_DEBUG] Step {}", step);
println!("  input_ids: {:?}", input_ids);
println!("  use_cache_branch: {}", use_cache_branch);
println!("  kv_cache present: {}", kv_cache.is_some());
```

### 2. 使用 ONNX Runtime �?Python API 验证

```python
# �?Python 中逐步验证每一�?
# 这样可以确定是模型问题还�?Rust 代码问题
```

### 3. 检查模型输�?输出形状

```rust
// 打印所有输入输出的形状
for (name, value) in inputs.iter() {
    println!("Input {}: shape = {:?}", name, get_shape(value));
}
```

### 4. 使用 ONNX 模型检查工�?

```bash
# 使用 onnxruntime �?Python API 检查模�?
python -c "import onnxruntime as ort; sess = ort.InferenceSession('model.onnx'); print(sess.get_inputs())"
```

---

## 📊 预期性能提升

| 方案 | 短序列（5 tokens�?| 长序列（50 tokens�?| 稳定�?|
|------|-------------------|-------------------|--------|
| 当前（Workaround�?| ~200ms | ~2000ms | ⭐⭐⭐⭐�?|
| 方案 1（修复代码） | ~100ms | ~500ms | ⭐⭐⭐⭐ |
| 方案 2（修复模型） | ~100ms | ~500ms | ⭐⭐⭐⭐�?|
| 方案 3（黑�?Value�?| ~150ms | ~800ms | ⭐⭐�?|
| 方案 4（混合模式） | ~200ms | ~800ms | ⭐⭐⭐⭐ |

---

## 🎯 最终建�?

### 立即行动

1. **先尝试方�?1**（修复代码实现）
   - 这是最快的方案
   - 如果成功，可以立即获得性能提升
   - 如果失败，至少可以获得详细的调试信息

2. **如果方案 1 失败，尝试方�?2**（修复模型导出）
   - 这是最根本的解决方�?
   - 虽然耗时较长，但成功率最�?

3. **如果都失败，使用方案 4**（混合模式）
   - 作为过渡方案
   - 至少可以在长序列时获得性能提升

### 长期规划

- 如果方案 1 �?2 成功，可以完全启�?KV cache
- 如果都失败，考虑�?
  - 升级 `ort` crate 到更新版�?
  - 或者使用其�?ONNX Runtime 绑定（如 `onnxruntime-rs`�?

---

## 📝 实施检查清�?

### 方案 1 检查清�?

- [ ] 添加详细的调试输�?
- [ ] 修复 `build_initial_kv_values()` �?`dec_len`
- [ ] 确保第一步提�?KV cache
- [ ] 验证 `input_ids` 形状一致�?
- [ ] 运行测试，检查是否还�?Reshape 错误
- [ ] 性能测试（短序列和长序列�?

### 方案 2 检查清�?

- [ ] 检查模型导出脚本的动态轴定义
- [ ] 检�?`opset_version` �?ONNX IR 版本
- [ ] �?Python 中验证模�?KV cache
- [ ] 修复导出脚本（如果需要）
- [ ] 重新导出模型
- [ ] �?Rust 中测�?

---

**最后更�?*: 2024-12-19

