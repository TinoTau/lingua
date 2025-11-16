# KV Cache 方案 2：修复模型导出指南

## 📊 问题描述

### 当前问题

当 `use_cache_branch=true` 时，模型输出的 `present.*.encoder.*` 的第一个维度是 0（形状为 `(0, 8, 1, 64)`），不能用作下一步的 `past_key_values.*.encoder.*` 输入。

### 根本原因

模型导出时，`use_cache_branch` 分支可能没有正确处理 encoder KV cache 的输出。当 `use_cache_branch=true` 时，模型可能认为 encoder KV cache 不需要更新，因此输出空的 tensor。

---

## 🔧 修复方案

### 方案 2.1：修改导出脚本，确保 encoder KV cache 始终输出

**思路**：
1. 修改 decoder wrapper，确保无论 `use_cache_branch` 的值如何，encoder KV cache 都正确输出
2. 在导出时，明确指定 encoder KV cache 的输出形状

**实施步骤**：

1. **修改 `DecoderWrapper`**：
   - 确保 `use_cache_branch=True` 时，encoder KV cache 仍然正确输出
   - 可能需要修改 forward 方法，强制输出 encoder KV cache

2. **修改动态轴定义**：
   - 明确指定 `present.*.encoder.*` 的形状
   - 确保 encoder KV cache 的形状与 `past_key_values.*.encoder.*` 一致

3. **使用 `optimum` 导出**：
   - 尝试使用 `optimum` 库的导出工具
   - `optimum` 可能对 KV cache 有更好的支持

---

### 方案 2.2：使用 `optimum` 导出（推荐）

**思路**：
使用 HuggingFace 的 `optimum` 库，它专门为 ONNX 导出优化，对 KV cache 有更好的支持。

**优点**：
- 专门为 ONNX 导出优化
- 对 KV cache 有更好的支持
- 自动处理复杂的动态形状

**实施步骤**：

1. **安装 `optimum`**：
   ```bash
   pip install optimum[onnxruntime]
   ```

2. **使用 `optimum` 导出**：
   ```python
   from optimum.onnxruntime import ORTModelForSeq2SeqLM
   from transformers import MarianMTModel
   
   model = MarianMTModel.from_pretrained("Helsinki-NLP/opus-mt-en-zh")
   ort_model = ORTModelForSeq2SeqLM.from_pretrained(
       "Helsinki-NLP/opus-mt-en-zh",
       export=True,
       use_cache=True,  # 启用 KV cache
   )
   ```

---

### 方案 2.3：修改模型结构（高级）

**思路**：
如果导出工具无法解决问题，可能需要修改模型结构，确保 encoder KV cache 始终正确输出。

**实施步骤**：

1. **创建自定义 Decoder**：
   - 继承 `MarianDecoder`
   - 重写 forward 方法，确保 encoder KV cache 始终输出

2. **修改导出脚本**：
   - 使用自定义 Decoder 导出
   - 确保 KV cache 输出正确

---

## 📋 推荐执行顺序

### 第一步：尝试方案 2.2（使用 `optimum`）

**原因**：
- 最简单，成功率最高
- `optimum` 专门为 ONNX 导出优化
- 对 KV cache 有更好的支持

**预计时间**：1-2 小时

### 第二步：如果方案 2.2 失败，尝试方案 2.1

**原因**：
- 需要修改导出脚本
- 可能需要深入理解模型结构

**预计时间**：2-3 小时

### 第三步：如果方案 2.1 也失败，考虑方案 2.3

**原因**：
- 最复杂，需要修改模型结构
- 可能影响模型准确性

**预计时间**：1-2 天

---

## 🔍 验证方法

### 1. 导出后验证

```python
import onnxruntime as ort
import numpy as np

# 加载模型
session = ort.InferenceSession("decoder_model.onnx")

# 检查输出形状
for output in session.get_outputs():
    if "present" in output.name and "encoder" in output.name:
        print(f"{output.name}: shape={output.shape}")
        # 应该不是 (0, ...) 的形状
```

### 2. Python 测试

运行 `scripts/test_marian_decoder_kv_cache.py`，检查：
- Step 1 的 `present.*.encoder.*` 形状是否正常
- 是否仍然是 `(0, 8, 1, 64)`

### 3. Rust 测试

运行 `cargo test --test nmt_quick_test`，检查：
- 是否还有 Reshape 错误
- KV cache 是否正常工作

---

## 📝 注意事项

1. **备份原始模型**：
   - 在修改导出脚本之前，备份原始模型
   - 如果修复失败，可以回退

2. **测试所有语言对**：
   - 修复后，需要测试所有语言对的模型
   - 确保修复对所有模型都有效

3. **性能影响**：
   - 修复后，检查模型性能是否受影响
   - 确保翻译质量没有下降

---

## 🎯 成功标准

修复成功的标准：
1. ✅ Python 测试中，`present.*.encoder.*` 的形状不是 `(0, ...)`
2. ✅ Rust 测试中，没有 Reshape 错误
3. ✅ KV cache 正常工作，翻译功能正常
4. ✅ 所有语言对的模型都能正常工作

---

**最后更新**: 2024-12-19

