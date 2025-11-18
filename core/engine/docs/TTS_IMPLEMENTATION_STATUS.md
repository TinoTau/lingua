# TTS 模块实现状态

**更新时间**: 2024-12-19  
**状态**: 代码实现完成，待测试

---

## ✅ 已完成

### 1. 基础结构
- ✅ `FastSpeech2TtsEngine` 结构体定义
- ✅ 模型加载逻辑（FastSpeech2 和 HiFiGAN，中英文）
- ✅ `TtsStreaming` trait 实现
- ✅ `TtsStub` 占位实现

### 2. 文本预处理
- ✅ `TextProcessor` 结构体
- ✅ `phone_id_map.txt` 加载
- ✅ 文本规范化（基本处理）
- ✅ 音素到 ID 映射
- ✅ 文本到音素转换（简化版）

### 3. FastSpeech2 推理
- ✅ 音素 ID 输入处理
- ✅ ONNX 模型推理
- ✅ Mel-spectrogram 输出处理
- ✅ 形状验证和转置处理

### 4. HiFiGAN 推理
- ✅ Mel-spectrogram 输入处理
- ✅ ONNX 模型推理
- ✅ 音频波形输出处理
- ✅ 形状验证（支持 [batch, samples] 和 [samples]）

### 5. 音频处理
- ✅ PCM 16-bit 转换
- ✅ 音频 chunk 分割（用于流式输出）
- ✅ WAV 文件保存工具（`audio_utils.rs`）

### 6. 错误处理
- ✅ 输入验证
- ✅ 形状验证
- ✅ 空数据检查
- ✅ 详细的错误消息

### 7. 测试
- ✅ 模型加载测试
- ✅ 文本预处理器测试
- ✅ 集成测试框架

---

## ⚠️ 待完善

### 1. 文本预处理（高优先级）
- ⚠️ **中文拼音转换**：当前只做字符映射，需要实现文本 → 拼音 → 音素
- ⚠️ **英文音素转换**：当前只做单词映射，需要实现文本 → 音素（使用 CMUdict）
- ⚠️ **数字转文字**：需要实现 "123" → "一百二十三" 或 "one hundred twenty three"
- ⚠️ **日期时间处理**：需要规范化日期时间格式
- ⚠️ **缩写展开**：需要展开常见缩写（如 "Dr." → "Doctor"）

### 2. FastSpeech2 输入形状（中优先级）
- ⚠️ **Embedding 层**：当前假设模型接受整数 ID，可能需要预处理 embedding
- ⚠️ **输入形状验证**：需要根据实际模型验证输入形状是 `[1, seq_len]` 还是 `[1, seq_len, 384]`

### 3. Mel-spectrogram 归一化（中优先级）
- ⚠️ **统计信息加载**：需要加载 `speech_stats.npy` 进行归一化
- ⚠️ **归一化/反归一化**：需要实现 mel-spectrogram 的归一化和反归一化

### 4. 流式输出（低优先级）
- ⚠️ **真正的流式 chunk**：当前返回完整音频，未来可以实现真正的流式分割
- ⚠️ **时间戳计算**：需要根据实际音频长度计算准确的时间戳

### 5. 集成到 CoreEngine（待实现）
- ⚠️ **Step 7**: 将 TTS 集成到 `CoreEngine` 的 `process_audio_frame` 流程中

---

## 📝 代码结构

```
core/engine/src/tts_streaming/
├── mod.rs                 # 模块声明和 trait 定义
├── fastspeech2_tts.rs     # FastSpeech2 + HiFiGAN 引擎
├── text_processor.rs      # 文本预处理
├── audio_utils.rs         # 音频工具（WAV 保存等）
└── stub.rs                # Stub 实现

core/engine/tests/
├── tts_model_load_test.rs      # 模型加载测试
├── tts_text_processor_test.rs  # 文本预处理器测试
└── tts_integration_test.rs     # 集成测试
```

---

## 🔧 已知问题

### 1. 编译环境问题
- ❌ Windows 环境下 `cargo build` 和 Python 脚本可能卡住
- 💡 **解决方案**: 使用 WSL 或 Linux 环境，或将项目目录添加到防病毒软件白名单

### 2. 文本预处理简化
- ⚠️ 当前文本预处理是简化实现，需要完善音素转换逻辑
- 💡 **解决方案**: 集成第三方库（如 `pypinyin` 用于中文，`CMUdict` 用于英文）

### 3. 模型输入形状
- ⚠️ FastSpeech2 输入形状可能需要根据实际模型调整
- 💡 **解决方案**: 通过实际测试验证模型输入输出形状

---

## 📋 下一步计划

### 优先级 1: 完善文本预处理
1. 实现中文拼音转换
2. 实现英文音素转换
3. 实现数字转文字

### 优先级 2: 集成到 CoreEngine
1. 在 `CoreEngine` 中添加 TTS 字段
2. 在 `process_audio_frame` 中调用 TTS
3. 处理 NMT 输出 → TTS 输入的流程

### 优先级 3: 测试和优化
1. 在 Linux 环境测试完整流程
2. 优化性能（批处理、缓存等）
3. 完善错误处理

---

## 📚 参考文档

- `TTS_IMPLEMENTATION_PLAN_FOR_REVIEW.md`: 完整的实现计划
- `TTS_TEST_CHECKLIST.md`: 测试清单
- `TTS_COMPILATION_ISSUE.md`: 编译问题诊断

---

**注意**: 由于 Windows 环境编译问题，建议在 Linux/macOS 环境进行测试。
