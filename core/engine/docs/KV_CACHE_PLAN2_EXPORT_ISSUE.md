# KV Cache 方案 2：模型导出修复遇到的问题

## 📊 问题总结

### 1. PyTorch 版本兼容性问题

**问题**：
- 新版本的 PyTorch（可能 >= 2.0）不再支持 `dynamic_axes` 参数
- 需要使用新的 `dynamic_shapes` API
- 导出脚本需要重写以适配新 API

**错误信息**：
```
RuntimeError: Failed to convert 'dynamic_axes' to 'dynamic_shapes'. 
Please provide 'dynamic_shapes' directly.
```

### 2. 重要发现：模型导出实际上是正确的

**根据之前的 Python 测试**：
- ✅ Python 测试中，模型可以正常工作
- ✅ Step 0 和 Step 1 都能成功执行
- ✅ 没有 Reshape 错误

**结论**：
- **模型导出是正确的**
- **问题在于 Rust 代码的实现**

---

## 🎯 建议

### 方案 A：修复导出脚本（不推荐）

**原因**：
- 需要适配新版本的 PyTorch API
- 可能需要降级 PyTorch 版本
- 但模型导出实际上是正确的，修复导出脚本不会解决问题

**预计时间**：2-3 小时

### 方案 B：实施方案 C（推荐）⭐⭐⭐⭐⭐

**原因**：
- 模型导出是正确的，不需要修复
- 问题在于 Rust 代码的实现
- 方案 C（重新设计 KV cache 结构）可以直接解决问题

**预计时间**：1-2 小时

---

## 📋 下一步行动

### 推荐：实施方案 C

1. **修改 `DecoderState` 结构**：
   - 将 `kv_cache` 分为 `decoder_kv_cache` 和 `encoder_kv_cache`
   - encoder KV cache 只在 Step 0 时提取一次，之后保持不变
   - decoder KV cache 在每次步骤中更新

2. **修改 `decoder_step` 方法**：
   - 在 Step 0 时提取并保存 encoder KV cache
   - 在后续步骤中，只更新 decoder KV cache，保持 encoder KV cache 不变

3. **测试验证**：
   - 运行 `cargo test --test nmt_quick_test`
   - 确认没有 Reshape 错误
   - 确认 KV cache 正常工作

---

## 📝 备份信息

**备份位置**：`core/engine/models/nmt/marian-en-zh-backup-20251117-091910`

**备份内容**：
- `encoder_model.onnx` (200.5 MB)
- `model.onnx` (224.91 MB)

**恢复方法**：
```powershell
Copy-Item -Path "core/engine/models/nmt/marian-en-zh-backup-20251117-091910/*.onnx" -Destination "core/engine/models/nmt/marian-en-zh/" -Force
```

---

## 🔍 技术细节

### PyTorch 版本兼容性问题

**新版本 PyTorch 的变化**：
- 移除了 `dynamic_axes` 参数
- 引入了 `dynamic_shapes` API
- 需要使用 `torch.export.export` 而不是 `torch.onnx.export`

**解决方案**：
1. 降级 PyTorch 到支持 `dynamic_axes` 的版本（如 1.x）
2. 重写导出脚本以使用新的 API
3. 使用 `optimum` 库导出（推荐）

### 为什么模型导出是正确的？

**证据**：
1. Python 测试通过：`scripts/test_marian_decoder_kv_cache.py` 成功运行
2. 模型可以正常执行 Step 0 和 Step 1
3. 没有 Reshape 错误

**结论**：
- 模型导出是正确的
- 问题在于 Rust 代码如何处理 KV cache
- 需要修复 Rust 代码，而不是模型导出

---

**最后更新**: 2024-12-19

