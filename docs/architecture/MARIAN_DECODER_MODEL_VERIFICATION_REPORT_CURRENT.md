# Marian Decoder 模型验证报告（当前版本）

**日期**: 2025-11-21  
**模型路径**: `core/engine/models/nmt/marian-zh-en/model.onnx`  
**模型版本**: IR 7, Opset 12  
**用途**: 提交给决策部门确认模型是否正确

---

## 执行摘要

### ✅ 成功点
- **输入数量**: 28 个输入，完全匹配代码期望
- **输入节点名**: 所有节点名与代码期望完全匹配
- **输入顺序**: 与代码期望的顺序一致
- **模型加载**: 成功加载，无错误
- **运行时**: 模型可以正常运行，无崩溃

### ⚠️ 关键问题
- **翻译质量**: 翻译结果重复且无意义（见测试日志）
- **Encoder KV Shape**: 模型期望 `src_seq`，代码传递 `encoder_seq_len`（29），**实际上匹配** ✅

---

## 1. Decoder 模型输入签名（Input Signature）

### 1.1 完整输入节点列表

**总输入数**: 28 个 ✅

| 索引 | 输入节点名 | 类型 | Shape |
|------|-----------|------|-------|
| 0 | `encoder_attention_mask` | tensor(int64) | `['batch', 'src_seq']` |
| 1 | `input_ids` | tensor(int64) | `['batch', 'tgt_seq']` |
| 2 | `encoder_hidden_states` | tensor(float) | `['batch', 'src_seq', 512]` |
| 3 | `past_key_values.0.decoder.key` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 4 | `past_key_values.0.decoder.value` | tensor(float) | `['batch', 8, 'past_seq', 64]` |
| 5 | `past_key_values.0.encoder.key` | tensor(float) | `['batch', 8, 'src_seq', 64]` ⚠️ |
| 6 | `past_key_values.0.encoder.value` | tensor(float) | `['batch', 8, 'src_seq', 64]` ⚠️ |
| 7-26 | `past_key_values.{1-5}.{decoder,encoder}.{key,value}` | tensor(float) | 类似结构 |
| 27 | `use_cache_branch` | tensor(bool) | `[1]` |

**关键发现**:
- **Encoder KV Shape**: 模型期望 `src_seq`（动态），不是 `past_seq`
- **代码传递**: `encoder_seq_len` (29)，与 `src_seq` 匹配 ✅

### 1.2 输入节点分类统计

| 类型 | 数量 | 输入索引 | 节点名模式 | Shape 模式 |
|------|------|---------|-----------|-----------|
| **基础输入** | 3 | Input[0-2] | `encoder_attention_mask`, `input_ids`, `encoder_hidden_states` | - |
| **Decoder KV Cache** | 12 | Input[3,4,7,8,11,12,15,16,19,20,23,24] | `past_key_values.{0-5}.decoder.{key,value}` | `['batch', 8, 'past_seq', 64]` |
| **Encoder KV Cache** | 12 | Input[5,6,9,10,13,14,17,18,21,22,25,26] | `past_key_values.{0-5}.encoder.{key,value}` | `['batch', 8, 'src_seq', 64]` ⚠️ |
| **控制标志** | 1 | Input[27] | `use_cache_branch` | `[1]` |
| **总计** | **28** | - | - | - |

### 1.3 输入节点名列表（按顺序）

```
encoder_attention_mask
input_ids
encoder_hidden_states
past_key_values.0.decoder.key
past_key_values.0.decoder.value
past_key_values.0.encoder.key
past_key_values.0.encoder.value
past_key_values.1.decoder.key
past_key_values.1.decoder.value
past_key_values.1.encoder.key
past_key_values.1.encoder.value
past_key_values.2.decoder.key
past_key_values.2.decoder.value
past_key_values.2.encoder.key
past_key_values.2.encoder.value
past_key_values.3.decoder.key
past_key_values.3.decoder.value
past_key_values.3.encoder.key
past_key_values.3.encoder.value
past_key_values.4.decoder.key
past_key_values.4.decoder.value
past_key_values.4.encoder.key
past_key_values.4.encoder.value
past_key_values.5.decoder.key
past_key_values.5.decoder.value
past_key_values.5.encoder.key
past_key_values.5.encoder.value
use_cache_branch
```

---

## 2. Decoder 模型输出签名（Output Signature）

### 2.1 完整输出节点列表

**总输出数**: 25 个

| 索引 | 输出节点名 | 类型 | Shape |
|------|-----------|------|-------|
| 0 | `logits` | tensor(float) | `['batch', 'tgt_seq', 65001]` |
| 1-2 | `present.0.decoder.{key,value}` | tensor(float) | `['batch', 8, 'present_seq', 64]` |
| 3-4 | `present.0.encoder.{key,value}` | tensor(float) | `['batch', 'Transposepresent.0.encoder.key_dim_1', 'present_seq', 'Transposepresent.0.encoder.key_dim_3']` |
| 5-24 | `present.{1-5}.{decoder,encoder}.{key,value}` | tensor(float) | 类似结构 |

---

## 3. KV Cache 构建日志（Shape 信息）

### 3.1 第一步（use_cache_branch=false）

#### 基础输入 Shape

```
[decoder_step] step input_ids_len=1, use_cache_branch=false, has_decoder_kv=false
[Input Construction] Basic inputs:
  - encoder_attention_mask: shape [1, 29]
  - decoder_input_ids: shape [1, 1]
  - encoder_hidden_states: shape [1, 29, 512]
  - use_cache_branch: false
```

#### Decoder KV Cache 构建

```
[KV Cache] Building zero decoder KV cache...
[KV Cache] Decoder KV cache built: 6 layers, shape [1, 8, 1, 64]
[KV Cache] Layer 0: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 1: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 2: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 3: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 4: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
[KV Cache] Layer 5: decoder_k shape [1, 8, 1, 64], decoder_v shape [1, 8, 1, 64]
```

**Decoder KV Cache 总结**:
- 层数: 6 层
- 每层 KV 数: 2 个（key, value）
- Shape: `[1, 8, 1, 64]` = `[batch, num_heads, past_seq, head_dim]`
- 总 KV 数: 12 个（6 层 × 2 KV）
- **状态**: ✅ 与模型期望 `['batch', 8, 'past_seq', 64]` 匹配

#### Encoder KV Cache 构建

```
[KV Cache] Building static encoder KV cache for encoder_seq_len=29...
[KV Cache] Encoder KV cache built: 6 layers, shape [1, 8, 29, 64]
[KV Cache] Layer 0: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 1: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 2: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 3: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 4: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
[KV Cache] Layer 5: encoder_k shape [1, 8, 29, 64], encoder_v shape [1, 8, 29, 64]
```

**Encoder KV Cache 总结**:
- 层数: 6 层
- 每层 KV 数: 2 个（key, value）
- Shape: `[1, 8, 29, 64]` = `[batch, num_heads, encoder_seq_len, head_dim]`
- 总 KV 数: 12 个（6 层 × 2 KV）
- **状态**: ✅ 与模型期望 `['batch', 8, 'src_seq', 64]` 匹配（`src_seq` = 29）

#### 输入组装

```
[KV Cache] Assembling KV cache inputs for 6 layers...
[KV Cache] Total KV cache inputs: 24 (6 layers × 4 KV per layer)
[Input Construction] Total inputs prepared: 28
[Input Construction] Input order: encoder_attention_mask, input_ids, encoder_hidden_states, past_key_values.* (24 KV), use_cache_branch
[Decoder] Calling decoder_session.run() with 28 inputs...
[Decoder] decoder_session.run() completed, got 25 outputs
```

### 3.2 后续步骤（use_cache_branch=true）

#### Decoder KV Cache

```
[KV Cache] Using existing decoder KV cache
```

#### Encoder KV Cache

```
[KV Cache] Building static encoder KV cache for encoder_seq_len=29...
[KV Cache] Encoder KV cache built: 6 layers, shape [1, 8, 29, 64]
```

#### 输入组装

```
[KV Cache] Total KV cache inputs: 24 (6 layers × 4 KV per layer)
[Input Construction] Total inputs prepared: 28
[Decoder] Calling decoder_session.run() with 28 inputs...
[Decoder] decoder_session.run() completed, got 25 outputs
```

### 3.3 KV Cache Shape 对比表

| KV Cache 类型 | 层数 | 每层 KV 数 | 代码构建 Shape | 模型期望 Shape | 状态 |
|--------------|------|-----------|---------------|---------------|------|
| **Decoder KV** | 6 | 2 (key, value) | `[1, 8, 1, 64]` | `['batch', 8, 'past_seq', 64]` | ✅ **匹配** |
| **Encoder KV** | 6 | 2 (key, value) | `[1, 8, 29, 64]` | `['batch', 8, 'src_seq', 64]` | ✅ **匹配** |
| **总计** | 6 | 4 | - | - | - |

**关键发现**:
- **Decoder KV**: 代码构建的 `past_seq=1` 与模型期望的动态 `past_seq` 匹配 ✅
- **Encoder KV**: 代码构建的 `encoder_seq_len=29` 与模型期望的动态 `src_seq` 匹配 ✅
- **结论**: Shape 匹配正确，不是导致翻译质量问题的原因

---

## 4. 实际测试结果

### 4.1 测试输入

**ASR 输出（源文本）**:
```
"Hello welcome. Hello welcome to the video. Welcome to the video. Hello welcome to the video of the video. Hello, welcome to the \"Lenlo Yu\" movie series."
```

### 4.2 NMT 翻译输出

**NMT 输出（目标文本）**:
```
"- It's all right, I'm gonna get you out of here, I'm gonna get you out of here, I'm gonna get you out of here, I'm gonna get you out of here, I'm gonna get you out of here, I'm gonna get you out of here, I'm gonna get you, I'm gonna get you, I'm gonna get you, I'm gonna get you, I'm gonna, I'm gonna, I'm gonna, I'm gonna, I'm gonna, I'm gonna, I'm"
```

**生成的 Token IDs**:
```
[65000, 28, 82, 21, 22, 58, 151, 2, 26, 21, 97, 894, 400, 37, 130, 4, 313, 2, 26, 21, 97, 894, 400, 37, 130, 4, 313, 2, 26, 21, 97, 894, 400, 37, 130, 4, 313, 2, 26, 21, 97, 894, 400, 37, 130, 4, 313, 2, 26, 21, 97, 894, 400, 37, 130, 4, 313, 2, 26, 21, 97, 894, 400, 37, 2, 26, 21, 97, 894, 400, 37, 2, 26, 21, 97, 894, 400, 37, 2, 26, 21, 97, 894, 400, 37, 2, 26, 21, 97, 894, 2, 26, 21, 97, 894, 2, 26, 21, 97, 894, 2, 26, 21, 97, 894, 2, 26, 21, 97, 894, 2, 26, 21, 97] (length: 129)
```

### 4.3 问题分析

**翻译质量问题**:
- ❌ **重复模式**: 翻译结果包含大量重复的短语
- ❌ **无意义输出**: 翻译内容与输入文本完全不相关
- ❌ **Token 重复**: Token IDs 显示明显的重复模式（如 `[2, 26, 21, 97, 894, 400, 37, 130, 4, 313]` 重复多次）

**可能原因**:
1. **模型训练问题**: 模型可能未正确训练或导出
2. **Token 解码问题**: Tokenizer 可能存在问题
3. **模型权重问题**: 模型权重可能损坏或不完整
4. **输入预处理问题**: 输入文本的预处理可能不正确
5. **解码策略问题**: 解码策略（如 beam search、sampling）可能存在问题

---

## 5. 关键发现和问题

### 5.1 输入数量 ✅

| 项目 | 当前模型 | 代码期望 | 状态 |
|------|---------|---------|------|
| 基础输入 | 3 | 3 | ✅ 匹配 |
| Decoder KV | 12 | 12 | ✅ 匹配 |
| Encoder KV | 12 | 12 | ✅ 匹配 |
| use_cache_branch | 1 | 1 | ✅ 匹配 |
| **总计** | **28** | **28** | ✅ **完全匹配** |

### 5.2 输入节点名 ✅

所有输入节点名与代码期望完全匹配：
- ✅ 基础输入: `encoder_attention_mask`, `input_ids`, `encoder_hidden_states`
- ✅ Decoder KV: `past_key_values.{0-5}.decoder.{key,value}`
- ✅ Encoder KV: `past_key_values.{0-5}.encoder.{key,value}`
- ✅ 控制标志: `use_cache_branch`

### 5.3 KV Cache Shape ✅

**重要发现**:
- **Encoder KV Shape**: 模型期望 `src_seq`（动态），代码传递 `encoder_seq_len` (29)
- **匹配状态**: ✅ **完全匹配**（`src_seq` = 29）
- **结论**: Shape 匹配正确，不是导致翻译质量问题的原因

### 5.4 翻译质量问题 ❌

**问题描述**:
- 翻译结果重复且无意义
- Token IDs 显示明显的重复模式
- 翻译内容与输入文本完全不相关

**可能原因**:
1. 模型训练/导出问题
2. Tokenizer 问题
3. 模型权重问题
4. 输入预处理问题
5. 解码策略问题

---

## 6. 总结

### 6.1 成功点 ✅

- ✅ **输入数量正确**: 28 个输入，完全匹配代码期望
- ✅ **所有输入都存在**: 包括 12 个 Encoder KV cache 输入
- ✅ **输入顺序正确**: 与代码期望的顺序一致
- ✅ **输入节点名正确**: 所有节点名与代码期望完全匹配
- ✅ **KV Cache 构建正确**: 代码正确构建了所有 24 个 KV cache
- ✅ **KV Cache Shape 匹配**: Decoder KV 和 Encoder KV 的 Shape 都与模型期望匹配
- ✅ **模型加载成功**: 模型可以成功加载和运行
- ✅ **运行时无崩溃**: 模型可以正常运行，无访问违规或崩溃

### 6.2 需要解决的问题 ❌

- ❌ **翻译质量**: 翻译结果重复且无意义，需要进一步调查
- ❌ **Token 重复**: Token IDs 显示明显的重复模式，可能表明解码策略或模型问题

### 6.3 建议

1. **验证模型导出**: 检查模型导出过程是否正确，确认模型权重完整
2. **对比工作模型**: 对比 `marian-en-zh` 模型，确认导出脚本和参数是否正确
3. **检查 Tokenizer**: 验证 Tokenizer 是否正确加载和使用
4. **检查解码策略**: 验证解码策略（beam search、sampling 等）是否正确实现
5. **检查输入预处理**: 验证输入文本的预处理是否正确
6. **使用参考实现**: 使用 PyTorch 原始模型进行对比测试，确认导出模型是否正确

---

**最后更新**: 2025-11-21  
**状态**: ✅ **模型输入/输出签名正确，KV Cache Shape 匹配，但翻译质量存在问题**

