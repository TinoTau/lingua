# Marian NMT 模型版本历史和使用情况

**日期**: 2025-11-21  
**状态**: 📋 历史记录

## 关键发现

### 1. 原始模型版本

**`marian-zh-en`（中文→英文）**:
- **原始 IR 版本**: IR 10, Opset 18
- **当前 IR 版本**: IR 9, Opset 12（已手动降级，有问题）
- **备份文件**: `encoder_model.onnx.ir10.backup`, `model.onnx.ir10.backup`

**`marian-en-zh`（英文→中文）**:
- **IR 版本**: IR 9, Opset 18（原生 IR 9，正常）
- **状态**: ✅ 正常工作

### 2. 之前使用的模型版本

#### 在本次测试之前

**大部分功能和测试使用的是 `marian-en-zh`（英文→中文）**:

1. **`bootstrap.rs` 默认模型**:
   ```rust
   // core/engine/src/bootstrap.rs:115
   pub fn nmt_with_default_marian_onnx(mut self) -> EngineResult<Self> {
       let model_dir = crate_root.join("models/nmt/marian-en-zh");  // ← 使用 marian-en-zh
   }
   ```

2. **测试文件使用的模型**:
   - `tests/nmt_comprehensive_test.rs` → `marian-en-zh`
   - `tests/business_flow_e2e_test.rs` → `marian-en-zh`
   - `tests/business_flow_step_by_step_test.rs` → `marian-en-zh`
   - `tests/nmt_bootstrap_integration.rs` → `marian-en-zh`
   - `tests/nmt_quick_test.rs` → `marian-en-zh`
   - 等等...

3. **`marian-en-zh` 的 IR 版本**: IR 9, Opset 18（原生 IR 9，与 `ort` 1.16.3 兼容）

#### 只有新的 S2S 测试使用 `marian-zh-en`

**只有以下文件使用 `marian-zh-en`（中文→英文）**:

1. `examples/test_s2s_full_simple.rs` → `marian-zh-en`
2. `examples/test_s2s_full_real.rs` → `marian-zh-en`

**原因**: 这些测试是为了验证**中文到英文**的完整 S2S 流程：
- 中文语音 → Whisper ASR → 中文文本
- 中文文本 → Marian NMT → 英文文本
- 英文文本 → Piper TTS → 英文语音

### 3. 为什么选择 `marian-zh-en`？

**没有找到明确的历史文档说明为什么选择 `marian-zh-en`**，但从代码和测试来看：

1. **S2S 测试需求**: 需要测试中文→英文的翻译流程
2. **模型可用性**: `marian-zh-en` 模型已下载到 `models/nmt/marian-zh-en/`
3. **IR 版本问题**: 该模型是 IR 10，但 `ort` 1.16.3 只支持 IR 9

### 4. 历史文档中的 IR 版本说明

**Emotion 模块的类似问题**:

- **文档**: `core/engine/docs/archived/EMOTION_IR9_TEST_RESULT.md`
- **问题**: Emotion XLM-R 模型使用 IR 10，但 `ort` 1.16.3 只支持 IR 9
- **结论**: 手动降级 IR 版本**不能满足功能需求**，因为只修改了元数据，模型内部操作定义未转换

**NMT 模型的 IR 版本要求**:

- **文档**: `core/engine/docs/KV_CACHE_OPTIMIZATION_RECOMMENDED.md`
- **说明**: "确保导出为 IR version 9（ort 1.16.3 要求）"
- **但**: 这是针对**新导出**的模型，不是针对已存在的模型

## 对之前功能的影响分析

### ✅ 不会影响之前的功能

**原因**:

1. **之前的功能使用的是 `marian-en-zh`**:
   - 该模型是原生 IR 9，与 `ort` 1.16.3 兼容
   - 所有测试和功能都基于这个模型

2. **`marian-zh-en` 是新测试才使用的**:
   - 只有 `test_s2s_full_simple.rs` 和 `test_s2s_full_real.rs` 使用
   - 这些是新添加的测试，不是之前的功能

3. **模型降级只影响 `marian-zh-en`**:
   - 降级操作只修改了 `marian-zh-en` 模型
   - `marian-en-zh` 模型未受影响

### ⚠️ 如果恢复原始模型

**如果恢复 `marian-zh-en` 的原始 IR 10 模型**:

1. **不会影响之前的功能**: 因为之前的功能不使用 `marian-zh-en`
2. **会影响新的 S2S 测试**: 因为新测试需要 `marian-zh-en`，但 IR 10 模型无法加载

### ⚠️ 如果保持降级后的模型

**如果保持降级后的 IR 9 模型**:

1. **不会影响之前的功能**: 因为之前的功能不使用 `marian-zh-en`
2. **新测试仍然无法运行**: 因为降级后的模型有运行时错误（`Unrecognized attribute: start for operator Shape`）

## 建议

### 方案 1: 恢复原始模型，升级 ONNX Runtime（推荐）⭐

**优点**:
- 不影响之前的功能（它们使用 `marian-en-zh`）
- 解决 IR 版本兼容性问题
- 使用原始模型，无功能缺失

**步骤**:
1. 恢复 `marian-zh-en` 的原始 IR 10 模型
2. 升级 `ort` 到支持 IR 10 的版本
3. 测试所有功能（包括之前的功能和新测试）

### 方案 2: 使用 `marian-en-zh` 进行 S2S 测试

**优点**:
- 快速验证 S2S 流程
- 不需要修改模型或升级 ONNX Runtime

**缺点**:
- 测试的是英文→中文流程，不是中文→英文流程
- 不符合原始测试目标

**步骤**:
1. 修改 `test_s2s_full_simple.rs` 使用 `marian-en-zh`
2. 调整测试流程为：英文语音 → 英文文本 → 中文文本 → 中文语音

### 方案 3: 保持现状，等待解决方案

**优点**:
- 不影响之前的功能
- 保留原始模型备份

**缺点**:
- 新测试无法运行
- 问题未解决

## 总结

1. **之前使用的模型**: `marian-en-zh`（IR 9, Opset 18）✅
2. **新测试使用的模型**: `marian-zh-en`（原始 IR 10, Opset 18）❌
3. **降级操作的影响**: 只影响 `marian-zh-en`，不影响之前的功能
4. **恢复原始模型的影响**: 不会影响之前的功能，但会影响新测试
5. **推荐方案**: 恢复原始模型，升级 ONNX Runtime

## 相关文件

- `core/engine/src/bootstrap.rs` - 默认使用 `marian-en-zh`
- `core/engine/examples/test_s2s_full_simple.rs` - 使用 `marian-zh-en`
- `core/engine/models/nmt/marian-zh-en/encoder_model.onnx.ir10.backup` - 原始模型备份
- `core/engine/docs/archived/EMOTION_IR9_TEST_RESULT.md` - Emotion 模块的类似问题

