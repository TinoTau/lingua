# Marian NMT IR 版本兼容性问题

**日期**: 2025-11-21  
**状态**: ✅ 已恢复原始模型，待升级 ONNX Runtime

## 问题描述

在运行完整 S2S 测试时，Marian NMT 模型（`marian-zh-en`）无法加载，错误信息：

```
Unsupported model IR version: 10, max supported IR version: 9
```

## 历史背景

### 之前遇到过类似问题

**是的**，在 Emotion 模块遇到过完全相同的问题：

- **文件**: `core/engine/docs/archived/EMOTION_IR9_TEST_RESULT.md`
- **问题**: Emotion XLM-R 模型使用 IR 10，但 `ort` 1.16.3 只支持 IR 9
- **尝试**: 手动降级 IR 版本到 9
- **结果**: ❌ **失败** - 手动降级只修改了元数据，模型内部操作定义仍使用 opset 18 特性，导致运行时错误

### Emotion 模块的教训

手动降级 IR 版本的问题：

1. **只修改元数据**: IR version 和 opset version 被修改，但模型内部的操作定义未转换
2. **运行时错误**: 出现 `Unrecognized attribute: start for operator Shape` 等错误
3. **根本原因**: `Shape` 操作在 opset 18 中支持 `start` 属性，但在 opset 12 中不支持

## 当前状态

### Marian NMT 模型

- **模型**: `marian-zh-en`
- **原始 IR 版本**: 10 (opset 18)
- **当前 IR 版本**: 10 (opset 18) - ✅ 已恢复原始模型
- **状态**: ✅ 已恢复，待升级 ONNX Runtime 以支持 IR 10

### 错误信息

```
Error Unrecognized attribute: start for operator Shape
```

这与 Emotion 模块遇到的问题**完全相同**。

## 对其他功能的影响

### 1. NMT 功能

- ❌ **当前状态**: `marian-zh-en` 模型无法加载
- ⚠️ **影响范围**: 所有使用 `marian-zh-en` 的测试和功能
- ✅ **其他模型**: `marian-en-zh` 等模型可能仍可正常使用（需验证）

### 2. Emotion 功能

- ⚠️ **已知问题**: Emotion 模块也存在 IR 版本问题
- 📝 **状态**: 已归档，未完全解决

### 3. 其他 ONNX 模型

- ✅ **ASR (Whisper)**: 不使用 ONNX Runtime，不受影响
- ✅ **TTS (Piper)**: 通过 HTTP 服务调用，不受影响
- ⚠️ **其他 NMT 模型**: 需要检查 IR 版本

## 解决方案

### 方案 1: 升级 ONNX Runtime（推荐）⭐

**优点**:
- 使用原始 IR 10 模型，无需降级
- 模型完整，无功能缺失
- 代码修改最小

**缺点**:
- 需要测试是否影响其他功能
- 可能需要处理 API 变化

**实施步骤**:
1. 升级 `ort` 到支持 IR 10 的版本
2. 测试 NMT 功能是否正常
3. 测试 Emotion 功能是否正常
4. 如果都正常，直接使用 IR 10 模型

**风险评估**:
- ⚠️ 需要全面测试所有 ONNX 模型
- ⚠️ 可能影响现有功能

### 方案 2: 使用旧版本 PyTorch 重新导出模型

**优点**:
- 可以导出真正的 IR 9 模型
- 兼容 `ort` 1.16.3

**缺点**:
- 需要安装旧版本 PyTorch
- 需要重新导出所有模型
- 可能无法使用最新模型特性

**实施步骤**:
1. 安装 PyTorch 1.x（支持 opset 12）
2. 使用旧版本重新导出 `marian-zh-en` 模型
3. 验证模型兼容性

### 方案 3: 恢复原始模型，使用其他 NMT 模型

**优点**:
- 快速恢复功能
- 无需修改代码

**缺点**:
- 可能影响翻译质量
- 需要验证其他模型是否也有 IR 版本问题

**实施步骤**:
1. 恢复 `marian-zh-en` 原始模型（从备份）
2. 检查其他 NMT 模型的 IR 版本
3. 如果其他模型可用，暂时使用其他模型

## 推荐方案

**推荐使用方案 1（升级 ONNX Runtime）**，原因：

1. **根本解决**: 解决 IR 版本兼容性问题，而不是绕过它
2. **长期维护**: 使用最新版本的 ONNX Runtime，便于后续维护
3. **功能完整**: 使用原始模型，无功能缺失

## 下一步行动

1. **检查其他 NMT 模型的 IR 版本** 🔴
   - 检查 `marian-en-zh`、`marian-en-ja` 等模型的 IR 版本
   - 确认是否有可用的 IR 9 模型

2. **测试升级 ONNX Runtime** 🟡
   - 升级 `ort` 到支持 IR 10 的版本
   - 运行 NMT 测试
   - 运行 Emotion 测试
   - 如果都正常，使用 IR 10 模型

3. **如果升级失败** 🟡
   - 考虑方案 2（重新导出模型）
   - 或方案 3（使用其他模型）

## 相关文件

- `core/engine/docs/archived/EMOTION_IR9_TEST_RESULT.md` - Emotion 模块的类似问题
- `scripts/convert_onnx_ir9.py` - IR 版本转换脚本
- `scripts/convert_marian_nmt_ir9.ps1` - Marian NMT 转换脚本
- `core/engine/models/nmt/marian-zh-en/encoder_model.onnx.ir10.backup` - 原始模型备份

## 总结

- ✅ **之前遇到过**: 是的，在 Emotion 模块遇到过完全相同的问题
- ❌ **手动降级无效**: 手动降级只修改元数据，无法解决运行时错误
- ⚠️ **影响范围**: 主要影响 NMT 和 Emotion 功能
- 💡 **推荐方案**: 升级 ONNX Runtime 到支持 IR 10 的版本

