# VITS 中文 AISHELL3 音频质量修复

**日期**: 2024-12-19  
**问题**: 音频质量差，语速过快（至少2倍速）

---

## 已修复的问题

### 1. 语速过快
**原因**: `length_scale` 参数设置为 1.0（默认值），导致语速过快

**修复**: 将 `length_scale` 从 1.0 调整为 2.0
- 位置: `core/engine/src/tts_streaming/vits_tts.rs:478`
- `length_scale > 1.0` 会减慢语速，`< 1.0` 会加快语速

### 2. 采样率不正确
**原因**: 使用 16000 Hz 采样率保存音频，但 vits-zh-aishell3 模型输出是 22050 Hz

**修复**: 
- 将模型采样率设置为 22050 Hz（`core/engine/src/tts_streaming/vits_tts.rs:246`）
- 更新测试文件使用 22050 Hz 保存音频（`core/engine/tests/vits_tts_test.rs:171`）

---

## 修改详情

### 代码修改

1. **`length_scale` 参数调整**
   ```rust
   // 之前: 1.0
   let length_scale_array: Array1<f32> = Array1::from_vec(vec![1.0f32]);
   
   // 现在: 2.0（减慢语速）
   let length_scale_array: Array1<f32> = Array1::from_vec(vec![2.0f32]);
   ```

2. **采样率设置**
   ```rust
   // 根据模型类型设置采样率
   let sample_rate = if is_zh_aishell3 {
       22050u32  // vits-zh-aishell3 通常使用 22050 Hz
   } else {
       16000u32  // MMS TTS 使用 16000 Hz
   };
   ```

3. **测试文件更新**
   ```rust
   // 使用 22050 Hz 保存音频
   save_pcm_to_wav(&chunk.audio, &output_path, 22050, 1)
   ```

---

## 测试命令

```powershell
cd D:\Programs\github\lingua\core\engine
cargo test test_vits_tts_synthesize_chinese -- --nocapture
```

---

## 参数说明

### `length_scale` 参数
- **默认值**: 1.0
- **作用**: 控制语速
  - `> 1.0`: 减慢语速（例如 2.0 = 一半速度）
  - `< 1.0`: 加快语速（例如 0.5 = 两倍速度）
- **当前设置**: 2.0（如果还是太快，可以继续增大到 2.5 或 3.0）

### 其他可调参数

1. **`noise_scale`** (默认 0.667)
   - 控制音调变化
   - 增大值会增加音调变化

2. **`noise_scale_w`** (默认 0.8)
   - 控制音调变化（另一个维度）
   - 增大值会增加音调变化

3. **`sid`** (默认 0)
   - 说话人 ID
   - 可以尝试不同的说话人（0-N）

---

## 如果语速还是太快

可以继续增大 `length_scale`：

```rust
// 尝试 2.5 或 3.0
let length_scale_array: Array1<f32> = Array1::from_vec(vec![2.5f32]);
// 或
let length_scale_array: Array1<f32> = Array1::from_vec(vec![3.0f32]);
```

---

## 下一步

1. 运行测试验证修复效果
2. 如果语速还是太快，继续增大 `length_scale`
3. 如果音质有问题，可以调整 `noise_scale` 和 `noise_scale_w`

