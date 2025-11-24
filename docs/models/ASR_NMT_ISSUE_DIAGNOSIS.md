# ASR 和 NMT 问题诊断报告

**日期**: 2025-11-21  
**状态**: 🔍 **诊断中**

---

## 📋 测试环境

- **中文音频文件**: `test_output/chinese.wav` (137.54 KB)
- **英文音频文件**: `test_output/english.wav` (238.06 KB)
- **ASR 模型**: Whisper Base
- **NMT 模型**: M2M100

---

## 🔍 问题 1: ASR 识别问题

### 现象

从测试结果看，中文音频被识别为英文文本：
- **识别结果**: "Hello welcome. Hello welcome to the video..."
- **检测到的语言**: "unknown"（Whisper 无法返回）
- **文本推断**: 英文（文本主要为英文字符）

### 可能原因

1. **Whisper 语言检测不准确**
   - Whisper 可能无法正确检测中文音频
   - 需要验证音频文件的实际内容

2. **音频格式问题**
   - 音频格式可能不符合 Whisper 的期望
   - 需要检查采样率、声道数、位深等

3. **Whisper 模型问题**
   - 使用的 Whisper Base 模型可能对中文支持不够好
   - 可能需要使用更大的模型或专门的中文模型

### 诊断步骤

1. ✅ 创建 ASR 单元测试（`tests/asr_whisper_test.rs`）
2. ⏳ 运行 ASR 单元测试，验证中文和英文音频的识别结果
3. ⏳ 检查音频文件的实际内容
4. ⏳ 验证 Whisper 的语言检测功能

---

## 🔍 问题 2: NMT 翻译重复 Token 问题

### 现象

从测试结果看，NMT 翻译出现严重的重复 token 问题：

**英文→中文翻译**:
- **生成的 token 序列**: `[2, 128102, 22, 3, 22, 3, 22, 3, ...]`
- **解码结果**: `'鈻?<unk> 鈻?<unk> 鈻?<unk> ...'`
- **问题**: token 22 和 3 重复出现

**中文→英文翻译**:
- **生成的 token 序列**: `[2, 128022, 1658, 58, 1705, 247, 117, 1197, 58, 1705, 247, 117, 1197, ...]`
- **解码结果**: `'nen ed 鈻乭an 鈻乷n 鈻丩 賲丕 ed 鈻乭an 鈻乷n 鈻丩 賲丕 ed ...'`
- **问题**: token 序列 `[58, 1705, 247, 117, 1197]` 重复出现

### 分析

从日志看，解码过程如下：

**Step 0**:
- 输入: `[2]` (decoder_start_token_id)
- 输出: logits
- 强制选择: `128102` (目标语言 token)

**Step 1**:
- 输入: `[128102]`
- 输出: logits
- 选择: `22` (最高 logits)

**Step 2**:
- 输入: `[22]`
- 输出: logits
- 选择: `3` (最高 logits)

**Step 3**:
- 输入: `[3]`
- 输出: logits
- 选择: `22` (最高 logits) ← **重复！**

**Step 4**:
- 输入: `[22]`
- 输出: logits
- 选择: `3` (最高 logits) ← **重复！**

### 可能原因

1. **KV Cache 更新问题**
   - KV cache 可能没有正确更新
   - 每次步骤的 KV cache 可能被错误地重置或覆盖

2. **解码逻辑问题**
   - 增量解码模式可能有问题
   - `use_cache_branch` 标志可能设置不正确

3. **模型输入格式问题**
   - 输入 token 序列的格式可能不正确
   - 可能需要包含完整的历史序列，而不仅仅是最后一个 token

4. **EOS Token 检测问题**
   - EOS token 可能没有被正确检测
   - 导致解码循环无法正常终止

### 诊断步骤

1. ✅ 创建 NMT 单元测试（`tests/nmt_m2m100_test.rs`）
2. ✅ 运行 NMT 单元测试，验证翻译结果
3. ⏳ 添加详细的调试日志，跟踪 KV cache 的更新过程
4. ⏳ 检查 decoder_step 的实现，验证 KV cache 更新逻辑
5. ⏳ 对比 Python 参考实现，验证解码逻辑

---

## 🛠️ 下一步行动

### 优先级 1: 修复 NMT 重复 Token 问题

1. **添加详细的调试日志**
   - 在 `decoder_step` 中添加 KV cache 形状和内容的日志
   - 在 `translate` 中添加每个步骤的输入和输出日志

2. **检查 KV Cache 更新逻辑**
   - 验证 KV cache 是否正确传递到下一步
   - 检查 KV cache 的形状是否正确

3. **验证解码逻辑**
   - 对比 Python 参考实现
   - 检查增量解码模式的实现

### 优先级 2: 修复 ASR 识别问题

1. **运行 ASR 单元测试**
   - 验证中文和英文音频的识别结果
   - 检查音频文件的实际内容

2. **检查 Whisper 配置**
   - 验证语言检测参数
   - 检查是否需要指定语言提示

3. **验证音频格式**
   - 检查采样率、声道数、位深等
   - 确保音频格式符合 Whisper 的期望

---

## 📊 测试结果

### ASR 单元测试

- ⏳ 待运行

### NMT 单元测试

- ✅ 已运行
- ❌ 发现重复 token 问题
- ❌ 解码结果包含 `<unk>` 和乱码

---

## 📚 参考

- ASR Whisper 实现: `core/engine/src/asr_whisper/`
- NMT M2M100 实现: `core/engine/src/nmt_incremental/m2m100_*.rs`
- 单元测试: `core/engine/tests/asr_whisper_test.rs`, `core/engine/tests/nmt_m2m100_test.rs`

