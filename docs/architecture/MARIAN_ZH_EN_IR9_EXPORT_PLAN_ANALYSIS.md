# Marian zh-en IR 9 导出方案分析

**日期**: 2025-11-21  
**分析对象**: `MARIAN_ZH_EN_IR9_EXPORT_PLAN.md` 和 `export_marian_ir9.py`  
**问题**: 方案是否可行？是否会对已有功能产生不良影响？

---

## 1. 方案概述

### 1.1 方案目标

将 `marian-zh-en` 模型重新导出为 IR 9, opset 12 的 ONNX 模型，使其能够在 `ort` 1.16.3 下正常加载。

### 1.2 方案步骤

1. 使用 Python 3.10 + PyTorch 1.13.1 + transformers 4.40.0
2. 使用 `opset_version=12` 导出模型
3. 输出文件：`model_ir9.onnx`

---

## 2. 方案可行性分析

### 2.1 技术可行性 ✅

**优点**:
- ✅ 使用旧版本 PyTorch（1.13.1）可以导出 IR 9 模型
- ✅ 使用 `opset_version=12` 确保 IR 版本 ≤ 9
- ✅ 环境要求明确（Python 3.10, PyTorch 1.13.1）

**技术路径正确**:
- 从源头导出 IR 9 模型，而不是手动降级
- 避免了手动降级导致的操作定义不兼容问题

### 2.2 脚本问题 ⚠️

**关键问题**:

1. **文件结构不匹配**:
   - 脚本导出：`model_ir9.onnx`（完整模型，encoder+decoder 一起）
   - 代码期望：
     - `encoder_model.onnx`（encoder 模型，必需）
     - `model.onnx`（decoder 模型，必需）

2. **代码检查**:
   ```rust
   // core/engine/src/nmt_incremental/marian_onnx.rs:60
   let encoder_path = model_dir.join("encoder_model.onnx");
   if !encoder_path.exists() {
       return Err(anyhow!(
           "encoder_model.onnx not found at {}. Please export it first using scripts/export_marian_encoder.py",
           encoder_path.display()
       ));
   }
   ```

3. **脚本输出**:
   ```python
   # export_marian_ir9.py:39
   out_path = output_dir / "model_ir9.onnx"  # ← 只导出一个文件
   ```

### 2.3 模型结构问题 ⚠️

**当前代码架构**:
- 使用分离的 encoder 和 decoder 模型
- Encoder 和 decoder 分别加载和推理
- 支持增量解码（KV cache）

**脚本导出**:
- 导出完整模型（encoder+decoder 一起）
- 无法分离使用

---

## 3. 对已有功能的影响分析

### 3.1 直接影响 ❌

**如果直接使用脚本导出的模型**:

1. **代码无法加载**:
   - 缺少 `encoder_model.onnx`
   - 代码会报错：`encoder_model.onnx not found`

2. **文件命名不匹配**:
   - 脚本输出：`model_ir9.onnx`
   - 代码期望：`model.onnx`（decoder）

3. **模型结构不匹配**:
   - 脚本导出完整模型
   - 代码期望分离的 encoder 和 decoder

### 3.2 对现有功能的影响

#### 3.2.1 如果修改代码支持完整模型 ❌ 不推荐

**影响**:
- ⚠️ 需要大幅修改 `MarianNmtOnnx` 实现
- ⚠️ 可能影响增量解码（KV cache）功能
- ⚠️ 可能影响其他使用 `marian-en-zh` 的功能
- ⚠️ 需要全面测试

**风险**: 🟡 中高

#### 3.2.2 如果修改脚本导出分离模型 ✅ 推荐

**影响**:
- ✅ 不影响现有代码
- ✅ 保持现有架构
- ✅ 只替换模型文件

**风险**: 🟢 低

### 3.3 对其他模型的影响

**不受影响**:
- ✅ `marian-en-zh`（英文→中文）：使用不同的模型目录
- ✅ 其他 NMT 模型：使用不同的模型目录
- ✅ ASR、Emotion、TTS：不依赖 NMT 模型文件

**影响范围**:
- 只影响使用 `marian-zh-en` 的功能
- 主要是新的 S2S 测试（`test_s2s_full_simple.rs`）

---

## 4. 方案修正建议

### 4.1 方案 1: 修改脚本导出分离模型 ⭐ 推荐

**修改脚本**，分别导出 encoder 和 decoder：

```python
def export_marian_ir9_separate(output_dir: Path, model_id: str):
    # 1. 导出 encoder
    encoder_path = output_dir / "encoder_model.onnx"
    # ... 导出 encoder 模型
    
    # 2. 导出 decoder
    decoder_path = output_dir / "model.onnx"
    # ... 导出 decoder 模型
```

**优点**:
- ✅ 符合现有代码架构
- ✅ 不影响现有功能
- ✅ 可以保持增量解码功能

**缺点**:
- ⚠️ 需要修改脚本
- ⚠️ 需要了解如何分离导出 encoder 和 decoder

### 4.2 方案 2: 使用现有导出脚本

**检查是否有现有的导出脚本**:
- `scripts/export_marian_encoder.py`（代码中提到的）
- 其他导出脚本

**如果存在**:
- 使用现有脚本导出 encoder
- 修改 `export_marian_ir9.py` 只导出 decoder
- 确保都使用 IR 9, opset 12

### 4.3 方案 3: 重命名文件（临时方案）⚠️

**如果脚本导出的完整模型可以分离使用**:
1. 导出完整模型
2. 使用工具分离 encoder 和 decoder
3. 重命名为 `encoder_model.onnx` 和 `model.onnx`

**缺点**:
- ⚠️ 需要额外的分离工具
- ⚠️ 可能无法正确分离
- ⚠️ 不确定是否可行

---

## 5. 风险评估

### 5.1 技术风险

| 风险项 | 风险等级 | 说明 |
|--------|---------|------|
| 脚本导出格式不匹配 | 🟡 中 | 需要修改脚本或代码 |
| 模型结构不匹配 | 🟡 中 | 需要分离导出 |
| IR 版本兼容性 | 🟢 低 | 使用旧版本 PyTorch 应该可以 |

### 5.2 功能风险

| 风险项 | 风险等级 | 说明 |
|--------|---------|------|
| 影响现有功能 | 🟢 低 | 只影响 `marian-zh-en` |
| 影响其他模型 | 🟢 低 | 其他模型不受影响 |
| 需要代码修改 | 🟡 中 | 如果脚本不改，需要改代码 |

---

## 6. 推荐方案

### 6.1 推荐：修改脚本导出分离模型 ⭐

**步骤**:
1. 修改 `export_marian_ir9.py`，分别导出 encoder 和 decoder
2. 确保输出文件名为：
   - `encoder_model.onnx`（encoder）
   - `model.onnx`（decoder）
3. 使用 IR 9, opset 12 导出
4. 验证模型可以加载

**优点**:
- ✅ 符合现有代码架构
- ✅ 不影响现有功能
- ✅ 风险最低

### 6.2 备选：修改现有导出脚本 ⚠️ 需要修改

**发现现有脚本**:
- ⚠️ `scripts/export_marian_encoder.py` - 导出 encoder
  - 使用 `opset_version=13`
  - **问题**: 手动降级 IR 版本（`model_proto.ir_version = 9`），可能有问题
- ⚠️ `scripts/export_marian_onnx.py` - 导出 encoder 和 decoder
  - 使用 `opset_version=14`
  - **问题**: 不支持 IR 9
- ✅ `scripts/export_marian_decoder_fixed.py` - 导出 decoder

**建议**:
1. **修改 `export_marian_encoder.py`**:
   - 将 `opset_version=13` 改为 `opset_version=12`
   - 移除手动降级 IR 版本的代码（使用旧版本 PyTorch 从源头导出 IR 9）

2. **修改 `export_marian_onnx.py`**:
   - 将 `opset_version=14` 改为 `opset_version=12`
   - 或创建新版本专门用于 IR 9 导出

3. **使用修改后的脚本**:
   - 在 Python 3.10 + PyTorch 1.13.1 环境中运行
   - 分别导出 encoder 和 decoder（IR 9 版本）

**优点**:
- ✅ 脚本已经存在，架构正确
- ✅ 支持分离导出 encoder 和 decoder
- ⚠️ 需要修改 `opset_version` 参数

**缺点**:
- ⚠️ 需要修改现有脚本
- ⚠️ 需要确保使用旧版本 PyTorch

---

## 7. 实施建议

### 7.1 立即行动

1. **检查现有导出脚本**:
   ```bash
   ls scripts/export_marian*.py
   ```

2. **如果存在分离导出脚本**:
   - 检查是否支持 IR 9 导出
   - 如果支持，使用现有脚本

3. **如果不存在**:
   - 修改 `export_marian_ir9.py`，分别导出 encoder 和 decoder
   - 参考 `marian-en-zh` 的模型结构

### 7.2 验证步骤

1. **导出模型后验证**:
   ```bash
   python -c "import onnx; m = onnx.load('encoder_model.onnx'); print(f'IR: {m.ir_version}')"
   python -c "import onnx; m = onnx.load('model.onnx'); print(f'IR: {m.ir_version}')"
   ```

2. **测试加载**:
   ```bash
   cargo run --example test_s2s_full_simple -- test_output/s2s_flow_test.wav
   ```

3. **验证功能**:
   - 确认模型可以加载
   - 确认翻译功能正常
   - 确认不影响其他功能

---

## 8. 总结

### 8.1 方案可行性

- ✅ **技术可行**: 使用旧版本 PyTorch 可以导出 IR 9 模型
- ⚠️ **脚本需要修改**: 当前脚本导出完整模型，但代码需要分离的 encoder 和 decoder

### 8.2 对已有功能的影响

- ✅ **影响范围小**: 只影响使用 `marian-zh-en` 的功能
- ✅ **不影响其他模型**: `marian-en-zh` 等其他模型不受影响
- ⚠️ **需要修改脚本**: 如果脚本不改，代码无法加载模型

### 8.3 推荐行动

1. **修改脚本**: 分别导出 encoder 和 decoder 模型
2. **保持文件结构**: 确保输出文件名符合代码期望
3. **验证兼容性**: 导出后验证 IR 版本和功能

---

## 9. 相关文件

- `MARIAN_ZH_EN_IR9_EXPORT_PLAN.md` - 导出计划
- `export_marian_ir9.py` - 导出脚本（需要修改）
- `core/engine/src/nmt_incremental/marian_onnx.rs` - 模型加载代码
- `core/engine/models/nmt/marian-zh-en/` - 模型目录

---

**最后更新**: 2025-11-21  
**状态**: 方案可行，但脚本需要修改以导出分离的 encoder 和 decoder 模型

