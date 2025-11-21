# 完整 S2S 流测试说明

## 测试程序

`test_s2s_full_real.rs` - 使用真实的 ASR 和 NMT 进行完整的语音转语音翻译测试

## 前提条件

### 1. Piper HTTP 服务

确保 WSL2 中的 Piper HTTP 服务正在运行：

```bash
# 在 WSL2 中
cd /mnt/d/Programs/github/lingua
bash scripts/wsl2_piper/start_piper_service.sh
```

### 2. 模型文件

确保以下模型文件已下载：

- **Whisper ASR**: `core/engine/models/asr/whisper-base/`
- **Marian NMT**: `core/engine/models/nmt/marian-zh-en/`（中文到英文）

### 3. 输入音频文件

准备一个中文语音的 WAV 文件：
- 格式：WAV
- 采样率：建议 16kHz（会自动处理）
- 声道：单声道或立体声（会自动转换为单声道）

## 使用方法

```bash
cd core/engine
cargo run --example test_s2s_full_real -- <input_wav_file>
```

### 示例

```bash
# 使用项目根目录的测试音频文件
cargo run --example test_s2s_full_real -- ../test_input/chinese_audio.wav

# 或使用绝对路径
cargo run --example test_s2s_full_real -- D:/Programs/github/lingua/test_input/chinese_audio.wav
```

## 测试流程

1. **检查服务**: 验证 Piper HTTP 服务是否运行
2. **加载音频**: 读取并解析输入 WAV 文件
3. **构建引擎**: 初始化 CoreEngine（ASR + NMT + TTS）
4. **初始化**: 启动所有组件
5. **ASR 识别**: 使用 Whisper 识别中文语音
6. **NMT 翻译**: 使用 Marian 将中文翻译为英文
7. **TTS 合成**: 使用 Piper 将中文文本合成为语音
8. **保存结果**: 将生成的音频保存到 `test_output/s2s_full_real_test.wav`

## 输出

测试完成后会输出：

- **源文本（中文）**: ASR 识别的结果
- **目标文本（英文）**: NMT 翻译的结果
- **音频文件**: `test_output/s2s_full_real_test.wav`

## 故障排除

### 错误: "Service not available"

**原因**: Piper HTTP 服务未运行

**解决**: 在 WSL2 中启动服务（见前提条件 1）

### 错误: "Whisper ASR model directory not found"

**原因**: Whisper 模型未下载

**解决**: 下载 Whisper 模型到 `core/engine/models/asr/whisper-base/`

### 错误: "Marian NMT model directory not found"

**原因**: Marian NMT 模型未下载

**解决**: 下载或导出 Marian NMT 模型到 `core/engine/models/nmt/marian-zh-en/`

### 错误: "ASR 未返回任何转录结果"

**原因**: 
- 音频文件格式不正确
- 音频内容无法识别
- ASR 模型加载失败

**解决**: 
- 检查音频文件格式
- 确保音频包含清晰的中文语音
- 检查 ASR 模型是否正确加载

## 预期结果

如果一切正常，你应该看到：

1. ✅ 所有步骤成功完成
2. ✅ 源文本（中文）被正确识别
3. ✅ 目标文本（英文）被正确翻译
4. ✅ 生成的音频文件大小合理（> 10KB）
5. ✅ 音频质量清晰可识别

## 下一步

测试成功后，可以：

1. 测试不同的音频文件
2. 测试不同的语言对（需要相应的 NMT 模型）
3. 集成到实际应用中
4. 进行性能优化

