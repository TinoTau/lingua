# KV Cache 优化方案

## 当前问题分析

### 现状
- **当前模式**: Workaround 模式（完全禁用 KV cache）
- **问题**: 
  - 每一步都使用完整历史序列作为 `input_ids`
  - 性能较慢，序列越长越慢
  - 平均每次翻译耗时 ~650ms

### 根本原因
启用 KV cache 时，第三步（step 2）会出现 Reshape 错误：
```
Non-zero status code returned while running Reshape node.
The dimension with value zero exceeds the dimension size of the input tensor.
```

---

## 方案对比

### 方案 1：保持 Workaround 模式（当前方案）

**描述**: 继续使用完整序列解码，不使用 KV cache

**优点**:
- ✅ 稳定，不会出现 Reshape 错误
- ✅ 代码简单，易于维护
- ✅ 翻译结果正确（虽然可能有重复 token）

**缺点**:
- ❌ 性能较慢（O(n²) 复杂度）
- ❌ 序列越长，性能越差
- ❌ 不适合生产环境

**适用场景**:
- 短期方案
- 测试和验证阶段
- 序列较短的情况（< 20 tokens）

**实现难度**: ⭐（已完成）

---

### 方案 2：修复 KV Cache 实现（推荐）

**描述**: 修复第三步的 Reshape 错误，正确启用 KV cache

**关键修复点**:

#### 2.1 修复 `decoder_step()` 的第一步处理
```rust
} else {
    // 第一步：提取 KV cache，为下一步启用正常模式
    let mut next_kv: Vec<[Value<'static>; 4]> = Vec::with_capacity(Self::NUM_LAYERS);
    for _layer in 0..Self::NUM_LAYERS {
        let dec_k = iter.next().expect("missing present.*.decoder.key");
        let dec_v = iter.next().expect("missing present.*.decoder.value");
        let enc_k = iter.next().expect("missing present.*.encoder.key");
        let enc_v = iter.next().expect("missing present.*.encoder.value");
        next_kv.push([dec_k, dec_v, enc_k, enc_v]);
    }
    state.kv_cache = Some(next_kv);
    state.use_cache_branch = true;  // 下一步启用 KV cache
}
```

#### 2.2 确保 `input_ids` 形状正确
- 第一步：`input_ids = [BOS]` (长度 1)
- 第二步及以后：`input_ids = [last_token]` (长度 1，只包含新 token)

#### 2.3 检查 `build_initial_kv_values` 的 `dec_len`
```rust
let dec_len = 1usize;  // 第一步的 decoder 历史长度为 1（只有 BOS）
```
可能需要根据实际 decoder 历史长度动态调整。

**优点**:
- ✅ 性能最优（O(n) 复杂度）
- ✅ 适合生产环境
- ✅ 序列长度不影响性能

**缺点**:
- ⚠️ 需要修复 Reshape 错误
- ⚠️ 可能需要调试和测试

**适用场景**:
- 生产环境
- 长序列翻译
- 性能要求高的场景

**实现难度**: ⭐⭐⭐（中等）

**风险**:
- 如果模型导出有问题，可能无法修复
- 可能需要重新导出模型

---

### 方案 3：混合模式（渐进式优化）

**描述**: 根据序列长度动态选择模式

**策略**:
- 短序列（< 10 tokens）：使用 workaround 模式（稳定）
- 长序列（≥ 10 tokens）：尝试使用 KV cache（性能优先）

**实现**:
```rust
let use_kv_cache = state.generated_ids.len() >= 10;
```

**优点**:
- ✅ 平衡稳定性和性能
- ✅ 短序列稳定，长序列快速
- ✅ 可以逐步迁移

**缺点**:
- ⚠️ 代码复杂度增加
- ⚠️ 仍然需要修复 KV cache 实现

**适用场景**:
- 过渡阶段
- 不确定 KV cache 是否完全修复时

**实现难度**: ⭐⭐（简单）

---

### 方案 4：重新导出模型（根本解决）

**描述**: 重新导出 ONNX 模型，确保 KV cache 分支正确

**可能的问题**:
1. **模型导出配置不正确**
   - `opset_version` 可能不兼容
   - 动态轴定义可能有问题
   - KV cache 分支的输入/输出形状可能不匹配

2. **ONNX IR 版本问题**
   - 当前使用 IR version 9（ort 1.16.3 要求）
   - 可能需要调整导出脚本

**修复步骤**:
1. 检查 `scripts/export_marian_encoder.py` 中的导出配置
2. 确保 `past_key_values` 和 `present.*` 的形状定义正确
3. 使用正确的 `opset_version`（当前是 13）
4. 验证导出的模型在 Python 中能正常工作

**优点**:
- ✅ 从根本上解决问题
- ✅ 可能解决所有 KV cache 相关问题

**缺点**:
- ⚠️ 需要重新导出所有模型
- ⚠️ 可能需要修改导出脚本
- ⚠️ 时间成本较高

**适用场景**:
- 其他方案都无法解决时
- 有时间和资源重新导出模型时

**实现难度**: ⭐⭐⭐⭐（较难）

---

### 方案 5：使用不同的 ONNX Runtime 版本

**描述**: 尝试升级或降级 `ort` crate 版本

**当前版本**: `ort = 1.16.3`

**可能尝试**:
- 升级到 `ort = 2.0.0-rc.10`（之前使用过，但有其他问题）
- 或者尝试其他稳定版本

**优点**:
- ✅ 可能解决 API 兼容性问题
- ✅ 不需要修改模型

**缺点**:
- ⚠️ 可能引入新的问题
- ⚠️ 需要重新适配代码
- ⚠️ 不确定是否能解决问题

**适用场景**:
- 怀疑是 `ort` 版本问题时
- 其他方案都失败时

**实现难度**: ⭐⭐⭐（中等）

---

## 推荐方案

### 短期（立即执行）
**方案 1 + 方案 3（混合模式）**
- 保持 workaround 模式作为默认
- 添加配置选项，允许用户选择是否启用 KV cache
- 为后续优化做准备

### 中期（1-2 周内）
**方案 2（修复 KV Cache 实现）**
- 尝试修复第三步的 Reshape 错误
- 如果成功，可以获得最佳性能
- 如果失败，回退到方案 1

### 长期（如果方案 2 失败）
**方案 4（重新导出模型）**
- 检查并修复模型导出脚本
- 重新导出所有模型
- 验证 KV cache 正常工作

---

## 实施建议

### 第一步：添加配置选项
```rust
pub struct MarianNmtOnnx {
    // ... 现有字段 ...
    use_kv_cache: bool,  // 新增：是否使用 KV cache
}

impl MarianNmtOnnx {
    pub fn new_from_dir_with_options(
        model_dir: &Path,
        use_kv_cache: bool,  // 新增参数
    ) -> Result<Self> {
        // ...
    }
}
```

### 第二步：实现方案 2 的修复
- 修改 `decoder_step()` 的第一步处理
- 确保 `input_ids` 形状正确
- 添加详细的调试输出

### 第三步：测试和验证
- 运行测试，检查是否还有 Reshape 错误
- 如果成功，性能应该显著提升
- 如果失败，回退到 workaround 模式

### 第四步：如果失败，考虑方案 4
- 检查模型导出脚本
- 重新导出模型
- 再次测试

---

## 性能对比（预期）

| 方案 | 短序列（5 tokens） | 长序列（50 tokens） | 稳定性 |
|------|-------------------|-------------------|--------|
| 方案 1（Workaround） | ~200ms | ~2000ms | ⭐⭐⭐⭐⭐ |
| 方案 2（KV Cache） | ~100ms | ~500ms | ⭐⭐⭐（需修复） |
| 方案 3（混合） | ~200ms | ~500ms | ⭐⭐⭐⭐ |

---

## 决策建议

**如果时间紧迫**:
- 选择方案 1（保持现状）
- 专注于其他模块的实现

**如果有时间优化**:
- 先尝试方案 2（修复 KV Cache）
- 如果失败，考虑方案 4（重新导出模型）

**如果想要平衡**:
- 选择方案 3（混合模式）
- 逐步迁移到方案 2

