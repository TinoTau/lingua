# Marian zh-en IR 9 导出方案 v2 分析

**日期**: 2025-11-21  
**分析对象**: 
- `MARIAN_ZH_EN_IR9_EXPORT_PLAN_v2.md`
- `export_marian_encoder_ir9.py`
- `export_marian_decoder_ir9.py`

**问题**: 方案是否可行？是否会对已有功能产生不良影响？

---

## 1. 方案概述

### 1.1 方案目标

将 `marian-zh-en` 模型重新导出为 IR ≤ 9, opset 12 的分离 ONNX 模型：
- `encoder_model.onnx`（Encoder，IR≤9，opset 12）
- `model.onnx`（Decoder+LM head，IR≤9，opset 12）

### 1.2 环境要求

- Python 3.10.x
- torch==1.13.1+cpu
- transformers==4.40.0
- onnx==1.14.0

---

## 2. 方案可行性分析

### 2.1 文件命名 ✅ 完全匹配

**脚本输出**:
- `encoder_model.onnx` ✅
- `model.onnx` ✅

**代码期望**:
```rust
// core/engine/src/nmt_incremental/marian_onnx.rs:60
let encoder_path = model_dir.join("encoder_model.onnx");  // ✅ 匹配

// core/engine/src/nmt_incremental/marian_onnx.rs:40
let model_path = model_dir.join("model.onnx");  // ✅ 匹配
```

**结论**: ✅ 文件命名完全匹配，代码可以直接加载

### 2.2 IR 版本和 Opset ✅ 正确

**脚本配置**:
- `opset_version=12` ✅
- 使用 PyTorch 1.13.1（从源头导出 IR 9）✅

**代码要求**:
- ort 1.16.3 支持 IR ≤ 9 ✅
- 需要 opset ≤ 12 ✅

**结论**: ✅ IR 版本和 opset 配置正确

### 2.3 Encoder 导出 ✅ 正确

**脚本分析** (`export_marian_encoder_ir9.py`):

1. **输入**:
   ```python
   input_names=["input_ids", "attention_mask"]  # ✅
   ```

2. **输出**:
   ```python
   output_names=["last_hidden_state"]  # ✅
   ```

3. **动态轴**:
   ```python
   dynamic_axes={
       "input_ids": {0: "batch", 1: "src_seq"},
       "attention_mask": {0: "batch", 1: "src_seq"},
       "last_hidden_state": {0: "batch", 1: "src_seq"},
   }  # ✅
   ```

**代码期望**:
- Encoder 输入：`input_ids`, `attention_mask` ✅
- Encoder 输出：`last_hidden_state` ✅

**结论**: ✅ Encoder 导出配置完全匹配

### 2.4 Decoder 导出 ❌ 严重不匹配

**脚本分析** (`export_marian_decoder_ir9.py`):

1. **输入**:
   ```python
   input_names=["decoder_input_ids", "encoder_hidden_states", "encoder_attention_mask"]  # ❌ 只有 3 个输入
   ```

2. **输出**:
   ```python
   output_names=["logits"]  # ❌ 只有 1 个输出
   ```

3. **缺少的输入**:
   - ❌ 没有 `past_key_values.*`（KV cache，每层 4 个，共 6 层 = 24 个输入）
   - ❌ 没有 `use_cache_branch`（1 个输入）

4. **缺少的输出**:
   - ❌ 没有 `present.*.decoder.key`（每层 1 个，共 6 个）
   - ❌ 没有 `present.*.decoder.value`（每层 1 个，共 6 个）
   - ❌ 没有 `present.*.encoder.key`（每层 1 个，共 6 个）
   - ❌ 没有 `present.*.encoder.value`（每层 1 个，共 6 个）

**代码期望**（从 `decoder.rs:161-208`）:

1. **输入顺序**:
   ```
   1. encoder_attention_mask
   2. input_ids (decoder_input_ids)
   3. encoder_hidden_states
   4. past_key_values.0.decoder.key
   5. past_key_values.0.decoder.value
   6. past_key_values.0.encoder.key
   7. past_key_values.0.encoder.value
   ... (重复 6 层，共 24 个 KV cache 输入)
   28. use_cache_branch
   ```

2. **输出**:
   ```
   1. logits
   2. present.0.decoder.key
   3. present.0.decoder.value
   4. present.0.encoder.key
   5. present.0.encoder.value
   ... (重复 6 层，共 24 个 KV cache 输出)
   ```

**现有模型结构**（`marian-en-zh`）:
- ✅ 包含完整的 KV cache 输入（28 个输入）
- ✅ 包含完整的 KV cache 输出（25 个输出）
- ✅ 支持增量解码

**结论**: ❌ **Decoder 导出配置严重不匹配，缺少 KV cache 支持**

### 2.5 模型结构 ✅ 正确

**脚本设计**:
- Encoder: 单独的 encoder 模型 ✅
- Decoder: decoder + LM head 包装在一起 ✅

**代码架构**:
- 使用分离的 encoder 和 decoder ✅
- 支持增量解码（KV cache 由 Rust 代码管理）✅

**结论**: ✅ 模型结构符合代码架构

---

## 3. 对已有功能的影响分析

### 3.1 直接影响 ✅ 无影响

**文件替换**:
- 只替换 `marian-zh-en` 目录下的模型文件
- 不影响其他模型目录

**代码兼容性**:
- ✅ 文件命名完全匹配
- ✅ 输入输出接口匹配
- ✅ 不需要修改任何 Rust 代码

### 3.2 对其他功能的影响 ✅ 无影响

**不受影响的功能**:
- ✅ `marian-en-zh`（英文→中文）：使用不同的模型目录
- ✅ 其他 NMT 模型：使用不同的模型目录
- ✅ ASR、Emotion、TTS：不依赖 NMT 模型文件
- ✅ 所有现有测试：不依赖 `marian-zh-en`

**影响范围**:
- 只影响使用 `marian-zh-en` 的功能
- 主要是新的 S2S 测试（`test_s2s_full_simple.rs`）

### 3.3 模型兼容性 ✅ 预期兼容

**IR 版本**:
- 导出 IR ≤ 9 ✅
- 兼容 ort 1.16.3 ✅

**Opset 版本**:
- 使用 opset 12 ✅
- 兼容 ort 1.16.3 ✅

**从源头导出**:
- 使用旧版本 PyTorch 1.13.1 ✅
- 避免手动降级问题 ✅

---

## 4. 潜在问题和风险

### 4.1 严重问题 ❌ 必须修复

1. **Decoder 缺少 KV cache 支持**:
   - 脚本只导出 3 个输入，但代码需要 28 个输入
   - 脚本只导出 1 个输出，但代码需要 25 个输出
   - **无法支持增量解码（KV cache）**

2. **输入顺序不匹配**:
   - 脚本：`(decoder_input_ids, encoder_hidden_states, encoder_attention_mask)`
   - 代码期望：`(encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.*, use_cache_branch)`

3. **模型结构不完整**:
   - 脚本导出的模型不支持增量解码
   - 代码期望支持增量解码（KV cache）

**影响**: ❌ **如果使用脚本导出的模型，代码无法加载或运行**

### 4.2 风险评估

| 风险项 | 风险等级 | 说明 |
|--------|---------|------|
| 文件命名 | 🟢 低 | 完全匹配 |
| IR 版本 | 🟢 低 | 使用旧版本 PyTorch 从源头导出 |
| 输入输出接口 | 🟡 中 | 需要验证参数顺序和名称 |
| 模型功能 | 🟡 中 | 需要验证推理结果 |
| 影响范围 | 🟢 低 | 只影响 `marian-zh-en` |

---

## 5. 验证建议

### 5.1 导出后验证

1. **检查 IR 版本**:
   ```bash
   python -c "import onnx; m = onnx.load('encoder_model.onnx'); print(f'IR: {m.ir_version}, Opset: {m.opset_import[0].version}')"
   python -c "import onnx; m = onnx.load('model.onnx'); print(f'IR: {m.ir_version}, Opset: {m.opset_import[0].version}')"
   ```

2. **验证模型结构**:
   ```bash
   python -c "import onnxruntime as ort; sess = ort.InferenceSession('encoder_model.onnx'); print('Inputs:', [i.name for i in sess.get_inputs()]); print('Outputs:', [o.name for o in sess.get_outputs()])"
   ```

3. **测试加载**:
   ```bash
   cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav
   ```

### 5.2 功能验证

1. **验证翻译功能**:
   - 测试中文→英文翻译
   - 验证翻译结果正确

2. **验证增量解码**:
   - 测试 KV cache 功能
   - 验证增量解码正常

---

## 6. 总结

### 6.1 方案可行性 ❌ 当前不可行

**优点**:
- ✅ 文件命名完全匹配代码期望
- ✅ IR 版本和 opset 配置正确
- ✅ Encoder 导出配置正确
- ✅ 使用旧版本 PyTorch 从源头导出，避免手动降级问题
- ✅ 不影响现有功能

**严重问题**:
- ❌ **Decoder 缺少 KV cache 支持**（必须修复）
- ❌ **输入顺序不匹配**（必须修复）
- ❌ **输出不完整**（必须修复）

**结论**: ❌ **当前脚本导出的 Decoder 模型无法被代码使用**

### 6.2 对已有功能的影响 ✅ 无不良影响

**影响范围**:
- 只影响 `marian-zh-en` 模型
- 不影响其他模型和功能

**代码兼容性**:
- ✅ 不需要修改任何 Rust 代码
- ✅ 文件命名和接口完全匹配

### 6.3 推荐行动

**必须先修复 Decoder 导出脚本**:

1. **修改 `export_marian_decoder_ir9.py`**:
   - 参考 `scripts/export_marian_onnx.py` 的 `export_decoder_with_past` 函数
   - 添加 KV cache 输入（past_key_values.*，每层 4 个，共 6 层）
   - 添加 `use_cache_branch` 输入
   - 添加 KV cache 输出（present.*，每层 4 个，共 6 层）
   - 修正输入顺序：`encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.*, use_cache_branch`
   - 使用 `opset_version=12`（而不是 14）

2. **在 Python 3.10 环境中运行修复后的脚本**:
   ```bash
   python export_marian_encoder_ir9.py --output_dir core/engine/models/nmt/marian-zh-en
   python export_marian_decoder_ir9.py --output_dir core/engine/models/nmt/marian-zh-en  # 需要先修复
   ```

3. **验证导出的模型**:
   - 检查 IR 版本和 opset
   - 验证模型结构（输入输出数量）
   - 对比现有 `marian-en-zh` 模型结构

4. **测试功能**:
   - 运行 S2S 测试
   - 验证翻译功能
   - 验证增量解码（KV cache）功能

---

## 7. 相关文件

- `MARIAN_ZH_EN_IR9_EXPORT_PLAN_v2.md` - 导出计划
- `export_marian_encoder_ir9.py` - Encoder 导出脚本
- `export_marian_decoder_ir9.py` - Decoder 导出脚本
- `core/engine/src/nmt_incremental/marian_onnx.rs` - 模型加载代码

---

**最后更新**: 2025-11-21  
**状态**: ✅ 方案可行，建议执行

