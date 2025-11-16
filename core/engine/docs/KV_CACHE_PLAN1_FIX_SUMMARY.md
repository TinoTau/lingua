# KV Cache 方案 1 修复总结

## ❌ 失败原因

### 问题 1：Encoder KV Cache 形状问题

**发现**：当 `use_cache_branch=true` 时，模型输出的 `present.*.encoder.*` 的第一个维度是 0（形状为 `(0, 8, 1, 64)`），不能用作下一步的 `past_key_values.*.encoder.*` 输入。

**解决方案**：在处理 `present.*` 输出时，跳过 `present.*.encoder.*`，保持使用初始的 encoder KV cache（从 Step 0 提取的）。

### 问题 2：Value 不支持 Clone

**发现**：`ort::Value` 不支持 `Clone`，无法在构建输入之前保存 encoder KV cache。

**当前状态**：代码尝试在构建输入之前保存 encoder KV cache，但由于 `Value` 不支持 `Clone`，导致编译错误。

---

## 🔧 修复方案

### 方案 A：在构建输入时分离 encoder KV cache

**思路**：
1. 在构建输入之前，从 `kv_to_use` 中提取 encoder KV cache（enc_k, enc_v）
2. 将 decoder KV cache 和 encoder KV cache 分别移动到 `input_values`
3. 在处理 `present.*` 输出时，使用保存的 encoder KV cache

**问题**：`Value` 不支持 `Clone`，无法提取 encoder KV cache。

### 方案 B：使用引用计数（Arc）

**思路**：
1. 将 `Value` 包装在 `Arc` 中，以便共享
2. 在构建输入时，使用 `Arc::clone()` 来共享 encoder KV cache
3. 在处理 `present.*` 输出时，使用共享的 encoder KV cache

**问题**：`Value` 可能不支持 `Arc` 包装，需要验证。

### 方案 C：重新设计 KV cache 结构

**思路**：
1. 将 encoder KV cache 和 decoder KV cache 分开存储
2. encoder KV cache 只在 Step 0 时提取一次，之后保持不变
3. decoder KV cache 在每次步骤中更新

**优点**：逻辑清晰，易于维护。

**实施**：
- 修改 `DecoderState` 结构，将 `kv_cache` 分为 `decoder_kv_cache` 和 `encoder_kv_cache`
- 在 Step 0 时提取 encoder KV cache 并保存
- 在后续步骤中，只更新 decoder KV cache，保持 encoder KV cache 不变

---

## 📋 推荐方案

**推荐使用方案 C（重新设计 KV cache 结构）**，原因：
1. 逻辑清晰，易于理解和维护
2. 避免了 `Value` 不支持 `Clone` 的问题
3. 符合模型的实际行为（encoder KV cache 不需要更新）

---

## ⏸️ 当前状态

- ✅ 问题已定位（encoder KV cache 形状问题）
- ⚠️ 修复方案已确定（方案 C）
- ⏳ 等待实施修复

---

**最后更新**: 2024-12-19

