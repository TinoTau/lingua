# VITS 中文 AISHELL3 参数调优指南

**日期**: 2024-12-19

---

## 当前参数设置

### 语速控制
- **length_scale**: `2.5`
  - 范围：通常 0.5 - 4.0
  - `> 1.0` 变慢，`< 1.0` 变快
  - 当前：2.5（如果偏慢可降到 2.0，如果偏快可升到 3.0）

### 音调控制
- **noise_scale**: `0.3`
  - 范围：通常 0.1 - 1.0
  - 控制音调变化和声音尖锐度
  - 减小值可以降低尖锐度，使声音更柔和
  - 当前：0.3（如果仍尖锐可降到 0.2 或 0.1）

- **noise_scale_w**: `0.4`
  - 范围：通常 0.1 - 1.0
  - 控制音调变化（另一个维度）
  - 减小值可以降低尖锐度
  - 当前：0.4（如果仍尖锐可降到 0.3 或 0.2）

### 说话人选择
- **sid**: `0`
  - 范围：取决于模型训练的说话人数量
  - vits-zh-aishell3 可能支持多个说话人
  - 可以尝试 0, 1, 2, 3 等不同的说话人 ID

---

## 问题诊断

### 声音尖锐 + 听不清楚发音

可能的原因：
1. **noise_scale 和 noise_scale_w 设置不当**
   - 当前已降低到 0.3 和 0.4
   - 如果仍尖锐，可继续降低

2. **说话人 ID 不合适**
   - 尝试不同的 sid（0, 1, 2, 3...）
   - 不同说话人的音质可能不同

3. **Tokenizer 编码问题**
   - 检查 tokenizer 是否正确编码中文文本
   - 确认 lexicon.txt 中的拼音映射是否正确

4. **模型质量问题**
   - vits-zh-aishell3 模型本身可能存在问题
   - 可能需要使用其他中文 TTS 模型

---

## 调优建议

### 如果声音尖锐
1. 继续降低 `noise_scale`（例如 0.2 或 0.1）
2. 继续降低 `noise_scale_w`（例如 0.3 或 0.2）
3. 尝试不同的说话人 ID

### 如果语速不合适
1. 偏慢：降低 `length_scale`（例如 2.0）
2. 偏快：提高 `length_scale`（例如 3.0）

### 如果听不清楚发音
1. 检查 tokenizer 编码是否正确
2. 尝试不同的说话人 ID
3. 检查模型文件是否完整
4. 考虑使用其他中文 TTS 模型

---

## 参数调整位置

文件：`core/engine/src/tts_streaming/vits_tts.rs`

方法：`run_inference_aishell3`

大约在第 478-494 行：

```rust
// noise_scale: [1] (float)
let noise_scale_array: Array1<f32> = Array1::from_vec(vec![0.3f32]);

// length_scale: [1] (float)
let length_scale_array: Array1<f32> = Array1::from_vec(vec![2.5f32]);

// noise_scale_w: [1] (float)
let noise_scale_w_array: Array1<f32> = Array1::from_vec(vec![0.4f32]);

// sid: [1] (int64) - 说话人 ID
let sid_array: Array1<i64> = Array1::from_vec(vec![0i64]);
```

---

## 下一步

1. **继续调优参数**：根据反馈调整 noise_scale 和 length_scale
2. **尝试不同说话人**：修改 sid 尝试不同的说话人
3. **检查 tokenizer**：验证中文编码是否正确
4. **考虑替代方案**：如果音质仍不理想，考虑使用其他中文 TTS 模型

