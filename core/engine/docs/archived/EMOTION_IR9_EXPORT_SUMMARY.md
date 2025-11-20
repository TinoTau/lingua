# Emotion 模型 IR 9 导出总结

## ✅ 导出完成

### 1. 模型导出

**执行命令**:
```bash
python scripts/export_emotion_model_ir9.py \
    --model_name cardiffnlp/twitter-xlm-roberta-base-sentiment \
    --output_dir core/engine/models/emotion/xlm-r \
    --opset_version 12
```

**结果**:
- ✅ 模型导出成功
- ⚠️ 初始导出为 IR version 10（PyTorch 默认使用 opset 18）

---

### 2. IR 版本降级

**问题**: 
- PyTorch 新版本默认导出 IR version 10
- `ort` 1.16.3 只支持 IR version 9
- 自动版本转换失败（LayerNormalization 操作不支持降级）

**解决方案**:
创建了手动降级脚本 `scripts/convert_onnx_ir9.py`

**执行命令**:
```bash
python scripts/convert_onnx_ir9.py \
    --input_model core/engine/models/emotion/xlm-r/model.onnx \
    --output_model core/engine/models/emotion/xlm-r/model_ir9.onnx
```

**结果**:
- ✅ 手动降级成功
- ⚠️ IR version: 9
- ⚠️ Opset version: 12（手动设置）
- ⚠️ **警告**: 手动降级可能导致运行时错误，需要测试

---

## 📁 生成的文件

### 模型文件
- `core/engine/models/emotion/xlm-r/model.onnx` - IR 10 版本（原始导出）
- `core/engine/models/emotion/xlm-r/model_ir9.onnx` - IR 9 版本（手动降级）✅

### 配置文件
- `core/engine/models/emotion/xlm-r/tokenizer.json` - Tokenizer 配置
- `core/engine/models/emotion/xlm-r/config.json` - 模型配置

---

## 🔧 代码更新

### 自动选择 IR 9 模型

更新了 `XlmREmotionEngine::new_from_dir()` 方法，优先使用 IR 9 模型：

```rust
// 优先使用 IR 9 版本的模型（如果存在）
let model_path = if model_dir.join("model_ir9.onnx").exists() {
    model_dir.join("model_ir9.onnx")
} else {
    model_dir.join("model.onnx")
};
```

**文件**: `core/engine/src/emotion_adapter/xlmr_emotion.rs`

---

## ⚠️ 注意事项

### 1. 手动降级的风险

手动降级 IR 版本和 opset 版本可能导致：
- 运行时错误
- 操作不兼容
- 性能问题

**建议**: 需要充分测试以确保模型正常工作

### 2. 测试计划

1. **模型加载测试**
   ```bash
   cargo test --test emotion_test test_xlmr_emotion_engine_load -- --nocapture
   ```

2. **推理测试**
   ```bash
   cargo test --test emotion_test test_xlmr_emotion_inference -- --nocapture
   ```

3. **集成测试**
   - 测试 Emotion 在完整业务流程中的使用

---

## 🎯 下一步

1. **测试 IR 9 模型** 🔴
   - 验证模型是否能正常加载
   - 验证推理是否正常工作

2. **如果测试失败** 🟡
   - 考虑升级 `ort` 到支持 IR 10 的版本
   - 或使用其他模型导出方法

3. **集成测试** 🟡
   - 测试 Emotion 在完整业务流程中的使用

---

## 📊 状态

| 任务 | 状态 | 说明 |
|------|------|------|
| 模型导出 | ✅ 完成 | IR 10 版本 |
| IR 降级 | ✅ 完成 | 手动降级到 IR 9 |
| 代码更新 | ✅ 完成 | 自动选择 IR 9 模型 |
| 功能测试 | ⚠️ 待测试 | 需要验证模型是否正常工作 |

---

**最后更新**: 2024-12-19  
**状态**: 模型导出和降级完成，待测试验证

