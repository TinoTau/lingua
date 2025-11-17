# TTS 模块测试清单

**版本**: 1.0  
**创建日期**: 2024-12-19

---

## 📋 测试概览

本文档列出了 TTS 模块实现后需要执行的所有测试，确保功能正确性和质量。

---

## ✅ 单元测试

### 1. 文本预处理测试

**测试文件**: `core/engine/tests/tts_text_processor_test.rs`

#### 1.1 文本规范化
- [ ] **数字转文字（中文）**
  - 输入: "123"
  - 预期: "一百二十三"
  - 测试: `test_normalize_numbers_chinese()`

- [ ] **数字转文字（英文）**
  - 输入: "123"
  - 预期: "one hundred twenty three"
  - 测试: `test_normalize_numbers_english()`

- [ ] **日期处理**
  - 输入: "2024年12月19日"
  - 预期: "二零二四年十二月十九日"
  - 测试: `test_normalize_date()`

- [ ] **标点符号处理**
  - 输入: "你好，世界！"
  - 预期: 正确处理标点符号
  - 测试: `test_normalize_punctuation()`

#### 1.2 音素转换
- [ ] **中文拼音转换**
  - 输入: "你好"
  - 预期: 音素序列（如 "ni3 hao3"）
  - 测试: `test_phoneme_conversion_chinese()`

- [ ] **英文音素转换**
  - 输入: "hello"
  - 预期: 音素序列（如 "HH EH L OW"）
  - 测试: `test_phoneme_conversion_english()`

#### 1.3 音素 ID 映射
- [ ] **音素 → ID 映射**
  - 输入: 音素序列
  - 预期: 对应的 ID 序列
  - 测试: `test_phoneme_to_id_mapping()`

- [ ] **ID → 音素映射（反向）**
  - 输入: ID 序列
  - 预期: 对应的音素序列
  - 测试: `test_id_to_phoneme_mapping()`

---

### 2. FastSpeech2 推理测试

**测试文件**: `core/engine/tests/tts_fastspeech2_test.rs`

#### 2.1 模型加载
- [ ] **中文模型加载**
  - 测试: `test_load_fastspeech2_chinese()`
  - 验证: 模型文件存在且可以加载

- [ ] **英文模型加载**
  - 测试: `test_load_fastspeech2_english()`
  - 验证: 模型文件存在且可以加载

#### 2.2 输入准备
- [ ] **音素 ID → 张量转换**
  - 输入: 音素 ID 序列 `[1, 2, 3, 4, 5]`
  - 预期: 正确的张量形状 `[1, 5, 384]`（batch=1, seq_len=5, dim=384）
  - 测试: `test_prepare_input_tensor()`

- [ ] **动态长度处理**
  - 输入: 不同长度的音素 ID 序列
  - 预期: 正确处理 padding
  - 测试: `test_dynamic_length_handling()`

#### 2.3 推理执行
- [ ] **单次推理**
  - 输入: 音素 ID 张量
  - 预期: mel-spectrogram 输出
  - 验证: 输出形状 `[1, 80, time_steps]`（batch=1, mel_dim=80）
  - 测试: `test_fastspeech2_inference()`

- [ ] **批量推理（可选）**
  - 输入: 多个音素 ID 序列
  - 预期: 批量 mel-spectrogram 输出
  - 测试: `test_fastspeech2_batch_inference()`

#### 2.4 Mel-spectrogram 处理
- [ ] **归一化/反归一化**
  - 输入: mel-spectrogram（原始）
  - 预期: 使用 `speech_stats.npy` 正确归一化/反归一化
  - 测试: `test_mel_normalization()`

---

### 3. HiFiGAN 推理测试

**测试文件**: `core/engine/tests/tts_hifigan_test.rs`

#### 3.1 模型加载
- [ ] **中文模型加载**
  - 测试: `test_load_hifigan_chinese()`
  - 验证: 模型文件存在且可以加载

- [ ] **英文模型加载**
  - 测试: `test_load_hifigan_english()`
  - 验证: 模型文件存在且可以加载

#### 3.2 Vocoder 推理
- [ ] **Mel-spectrogram → 音频转换**
  - 输入: mel-spectrogram `[1, 80, time_steps]`
  - 预期: 音频波形 `[1, audio_samples]`
  - 验证: 输出形状正确
  - 测试: `test_hifigan_inference()`

- [ ] **音频格式验证**
  - 验证: 采样率 = 16kHz
  - 验证: 位深 = 16-bit
  - 验证: 声道 = mono
  - 测试: `test_audio_format()`

---

### 4. 完整 TTS 流程测试

**测试文件**: `core/engine/tests/tts_integration_test.rs`

#### 4.1 中文文本合成
- [ ] **简单文本**
  - 输入: "你好"
  - 预期: 生成音频（PCM 格式）
  - 验证: 音频长度 > 0
  - 测试: `test_synthesize_chinese_simple()`

- [ ] **长文本**
  - 输入: "你好，世界。这是一个测试。"
  - 预期: 生成完整音频
  - 验证: 音频长度合理
  - 测试: `test_synthesize_chinese_long()`

#### 4.2 英文文本合成
- [ ] **简单文本**
  - 输入: "Hello"
  - 预期: 生成音频（PCM 格式）
  - 验证: 音频长度 > 0
  - 测试: `test_synthesize_english_simple()`

- [ ] **长文本**
  - 输入: "Hello, world. This is a test."
  - 预期: 生成完整音频
  - 验证: 音频长度合理
  - 测试: `test_synthesize_english_long()`

#### 4.3 流式输出测试
- [ ] **Chunk 分割**
  - 输入: 完整音频
  - 预期: 多个 chunk，最后一个 `is_last=true`
  - 验证: 所有 chunk 的 `audio` 长度 > 0
  - 测试: `test_streaming_chunks()`

- [ ] **时间戳验证**
  - 验证: 每个 chunk 的 `timestamp_ms` 递增
  - 验证: 时间戳间隔合理（基于采样率和 chunk 大小）
  - 测试: `test_chunk_timestamps()`

---

## 🔗 集成测试

### 5. 端到端业务流程测试

**测试文件**: `core/engine/tests/tts_e2e_test.rs`

#### 5.1 完整流程
- [ ] **ASR → NMT → TTS**
  - 输入: 音频帧（模拟 ASR 输入）
  - 流程: VAD → ASR → Emotion → Persona → NMT → TTS
  - 预期: 生成目标语言的音频
  - 验证: TTS 事件发布
  - 测试: `test_full_pipeline_asr_nmt_tts()`

#### 5.2 事件发布验证
- [ ] **TTS 事件格式**
  - 验证: 事件 topic = "TtsChunk"
  - 验证: payload 包含 `audio`, `timestamp_ms`, `is_last`
  - 测试: `test_tts_event_publishing()`

- [ ] **事件顺序**
  - 验证: 事件按时间顺序发布
  - 验证: 最后一个事件的 `is_last=true`
  - 测试: `test_tts_event_ordering()`

---

## 🎵 音频验证测试

### 6. WAV 文件输出测试

**测试文件**: `core/engine/tests/tts_audio_validation_test.rs`

#### 6.1 WAV 文件生成
- [ ] **保存为 WAV 文件**
  - 输入: TTS 生成的 PCM 音频
  - 操作: 转换为 WAV 格式并保存
  - 验证: WAV 文件存在且可读取
  - 测试: `test_save_wav_file()`

#### 6.2 WAV 文件格式验证
- [ ] **格式检查**
  - 验证: 采样率 = 16000 Hz
  - 验证: 位深 = 16-bit
  - 验证: 声道 = mono
  - 验证: 格式 = PCM
  - 测试: `test_wav_format()`

#### 6.3 音频质量验证（手动）
- [ ] **播放测试**
  - 操作: 使用音频播放器播放生成的 WAV 文件
  - 验证: 音频清晰、无杂音
  - 验证: 语音自然、可理解
  - **注意**: 这是手动测试，需要人工验证

---

## ⚡ 性能测试

### 7. 性能基准测试

**测试文件**: `core/engine/tests/tts_performance_test.rs`

#### 7.1 推理延迟
- [ ] **单次推理延迟**
  - 输入: 短文本（"你好"）
  - 测量: 从文本输入到音频输出的时间
  - 目标: < 500ms（短文本）
  - 测试: `test_inference_latency_short()`

- [ ] **长文本推理延迟**
  - 输入: 长文本（50+ 字符）
  - 测量: 从文本输入到音频输出的时间
  - 目标: < 2s（长文本）
  - 测试: `test_inference_latency_long()`

#### 7.2 内存使用
- [ ] **内存峰值测量**
  - 操作: 执行多次 TTS 推理
  - 测量: 内存使用峰值
  - 目标: < 500MB（单次推理）
  - 测试: `test_memory_usage()`

#### 7.3 并发性能
- [ ] **并发推理**
  - 操作: 同时执行多个 TTS 推理
  - 测量: 总耗时和成功率
  - 目标: 支持至少 2-3 个并发请求
  - 测试: `test_concurrent_inference()`

---

## 🐛 错误处理测试

### 8. 错误场景测试

**测试文件**: `core/engine/tests/tts_error_handling_test.rs`

#### 8.1 输入验证
- [ ] **空文本**
  - 输入: ""
  - 预期: 返回错误或空音频
  - 测试: `test_empty_text()`

- [ ] **超长文本**
  - 输入: 1000+ 字符
  - 预期: 返回错误或截断
  - 测试: `test_too_long_text()`

- [ ] **不支持的语言**
  - 输入: `locale = "fr"`（法语，未支持）
  - 预期: 返回错误
  - 测试: `test_unsupported_language()`

#### 8.2 模型加载错误
- [ ] **模型文件不存在**
  - 操作: 使用不存在的模型路径
  - 预期: 返回清晰的错误信息
  - 测试: `test_model_file_not_found()`

#### 8.3 推理错误
- [ ] **ONNX 推理失败**
  - 操作: 模拟 ONNX 推理错误
  - 预期: 返回错误，不 panic
  - 测试: `test_onnx_inference_error()`

---

## 📊 测试执行计划

### 阶段 1: 单元测试（开发阶段）
- **时间**: 与实现同步进行
- **目标**: 每个功能模块实现后立即编写测试
- **覆盖**: 文本预处理、FastSpeech2、HiFiGAN

### 阶段 2: 集成测试（功能完成后）
- **时间**: 所有功能实现后
- **目标**: 验证完整流程
- **覆盖**: 端到端流程、事件发布

### 阶段 3: 音频验证（功能稳定后）
- **时间**: 集成测试通过后
- **目标**: 验证音频质量
- **覆盖**: WAV 文件生成、格式验证、手动播放测试

### 阶段 4: 性能测试（优化阶段）
- **时间**: 功能稳定后
- **目标**: 性能基准和优化
- **覆盖**: 延迟、内存、并发

---

## ✅ 测试通过标准

### 必须通过
- ✅ 所有单元测试通过
- ✅ 所有集成测试通过
- ✅ 错误处理测试通过
- ✅ WAV 文件格式正确

### 推荐通过
- ⚠️ 性能测试达到目标（延迟、内存）
- ⚠️ 音频质量人工验证通过

---

## 📝 测试报告模板

测试完成后，应生成测试报告，包含：

1. **测试执行摘要**
   - 总测试数
   - 通过数
   - 失败数
   - 跳过数

2. **功能验证结果**
   - 文本预处理 ✅/❌
   - FastSpeech2 推理 ✅/❌
   - HiFiGAN 推理 ✅/❌
   - 完整流程 ✅/❌

3. **性能指标**
   - 平均推理延迟
   - 内存使用峰值
   - 并发性能

4. **已知问题**
   - 列出已知的限制或问题

---

**最后更新**: 2024-12-19  
**状态**: 📋 测试清单，等待实现完成后执行

