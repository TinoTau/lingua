# ASR + VAD 简单测试说明

## 测试文件

`asr_vad_simple_test.rs` - 包含 3 个测试，用于验证 ASR 服务器的停顿识别和文本识别功能。

## 测试列表

### 1. `test_asr_vad_simple` - 完整集成测试
- **功能**：验证 VAD 停顿识别 + ASR 文本识别
- **测试流程**：
  1. 加载 WAV 音频文件（`test_output/chinese.wav`）
  2. 将音频分割成 32ms 帧
  3. 逐帧处理，通过 VAD 检测停顿
  4. 在检测到停顿时，触发 ASR 识别文本
  5. 输出统计信息（停顿次数、识别次数、识别率等）

### 2. `test_vad_boundary_detection` - VAD 停顿识别测试
- **功能**：验证 VAD 能够检测到停顿
- **测试流程**：
  1. 创建 SileroVad
  2. 发送静音帧
  3. 验证是否检测到停顿

### 3. `test_asr_text_recognition` - ASR 文本识别测试
- **功能**：验证 ASR 能够识别文本
- **测试流程**：
  1. 创建 Whisper ASR
  2. 加载音频文件并累积帧
  3. 在边界时进行推理
  4. 验证识别结果

## 运行测试

### 前提条件

1. **模型文件**：
   - Whisper 模型：`core/engine/models/asr/whisper-base/`
   - Silero VAD 模型：`core/engine/models/vad/silero/silero_vad.onnx`

2. **测试音频文件**：
   - `test_output/chinese.wav`（用于完整集成测试和 ASR 文本识别测试）

### 运行命令

```bash
# 运行所有测试（不包括被忽略的）
cargo test --test asr_vad_simple_test

# 运行被忽略的测试（需要模型文件）
cargo test --test asr_vad_simple_test -- --ignored

# 运行特定测试（注意：测试名称在 --test 之后）
cargo test --test asr_vad_simple_test test_asr_vad_simple -- --ignored

# 运行 VAD 停顿识别测试
cargo test --test asr_vad_simple_test test_vad_boundary_detection -- --ignored

# 运行 ASR 文本识别测试
cargo test --test asr_vad_simple_test test_asr_text_recognition -- --ignored

# 运行测试并显示输出（--nocapture）
cargo test --test asr_vad_simple_test -- --ignored --nocapture
```

## 测试输出示例

测试会输出：
- 模型和音频文件检查结果
- 处理进度（每 100 帧输出一次）
- 检测到的停顿信息（时间戳、置信度）
- 识别到的文本（内容、语言、说话者）
- 统计信息（总帧数、停顿次数、识别次数、识别率）

## 注意事项

- 所有测试都标记为 `#[ignore]`，因为需要模型文件
- 如果模型文件不存在，测试会自动跳过（不会失败）
- 如果测试音频文件不存在，测试会提示并跳过

