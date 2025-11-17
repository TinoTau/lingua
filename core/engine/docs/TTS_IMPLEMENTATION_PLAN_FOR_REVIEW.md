# TTS 模块实现计划（评估部门审核版）

**文档版本**: 1.0  
**创建日期**: 2024-12-19  
**审核状态**: 待审核  
**预计实现时间**: 3-4 天（22-32 小时）

---

## 📋 执行摘要

本文档详细说明了 TTS（文本转语音）模块的实现计划，包括：
- 技术方案（FastSpeech2 + HiFiGAN）
- 实现步骤（7 个步骤）
- 测试计划（8 个测试类别）
- 风险评估和缓解措施

**目标**: 实现基于 ONNX 的 TTS 模块，支持中文和英文文本转语音，输出 PCM 格式音频（16-bit, 16kHz）。

---

## 1. 技术方案

### 1.1 架构选择

**方案**: FastSpeech2 (声学模型) + HiFiGAN (声码器)

**理由**:
- ✅ 模型文件已准备（`models/tts/fastspeech2-lite/`, `models/tts/hifigan-lite/`）
- ✅ ONNX 格式，与现有 NMT/Emotion 模块技术栈一致
- ✅ 支持流式生成（FastSpeech2 streaming 版本）
- ✅ 质量优秀（FastSpeech2 是业界标准 TTS 模型）

**替代方案**:
- ❌ Tacotron2 + WaveNet（模型更大，延迟更高）
- ❌ 端到端模型（如 VITS，但模型文件未准备）

### 1.2 数据流

```
文本输入
    ↓
文本预处理（规范化、音素转换）
    ↓
FastSpeech2 推理（音素 ID → Mel-spectrogram）
    ↓
HiFiGAN 推理（Mel-spectrogram → 音频波形）
    ↓
PCM 音频处理（16-bit, 16kHz, mono）
    ↓
Chunk 分割（流式输出）
    ↓
TtsStreamChunk 输出
```

### 1.3 技术栈

**依赖**:
- `ort = "1.16.3"` (ONNX Runtime，与现有模块一致)
- `ndarray = "0.15"` (张量操作)
- `tokenizers = "0.15"` (可选，用于文本预处理)

**模型文件**:
- FastSpeech2: `fastspeech2_csmsc_streaming.onnx` (中文), `fastspeech2_ljspeech.onnx` (英文)
- HiFiGAN: `hifigan_csmsc.onnx` (中文), `hifigan_ljspeech.onnx` (英文)
- 辅助文件: `phone_id_map.txt`, `speech_stats.npy`

---

## 2. 实现步骤

### Step 1: 创建基础结构（2-3 小时）

**任务**:
- 创建 `FastSpeech2TtsEngine` 结构体
- 实现模型加载逻辑
- 实现基础初始化方法

**交付物**:
- `core/engine/src/tts_streaming/fastspeech2_tts.rs`
- 模型加载测试

---

### Step 2: 实现文本预处理（4-6 小时）

**任务**:
- 文本规范化（数字转文字、标点处理）
- 音素转换（文本 → 音素序列）
- 音素 ID 映射（使用 `phone_id_map.txt`）

**交付物**:
- `core/engine/src/tts_streaming/text_processor.rs`
- 文本预处理测试

**技术挑战**:
- ⚠️ 中文拼音/音素转换可能需要外部库（如 `pinyin`）
- ⚠️ 英文音素转换可能需要 CMUdict 或类似词典

**缓解措施**:
- 先实现简单规则引擎，后续可扩展
- 或使用 HuggingFace `transformers` 库的 tokenizer（如果可用）

---

### Step 3: 实现 FastSpeech2 推理（4-6 小时）

**任务**:
- 输入准备（音素 ID → 张量 `[1, seq_len, 384]`）
- ONNX 推理执行
- Mel-spectrogram 提取（输出形状 `[1, 80, time_steps]`）
- 处理 `speech_stats.npy`（归一化/反归一化）

**交付物**:
- FastSpeech2 推理实现
- Mel-spectrogram 输出验证

**技术要点**:
- 使用 `ort` crate（与 NMT 一致）
- 处理动态序列长度
- Mel-spectrogram 形状: `[batch, mel_dim, time_steps]`

---

### Step 4: 实现 HiFiGAN 推理（3-4 小时）

**任务**:
- HiFiGAN 模型加载
- Mel-spectrogram → 音频波形转换
- PCM 格式输出（16-bit, 16kHz, mono）

**交付物**:
- `core/engine/src/tts_streaming/hifigan_vocoder.rs`
- 音频格式验证

**技术要点**:
- 输入: mel-spectrogram `[1, 80, time_steps]`
- 输出: 音频波形 `[1, audio_samples]`
- 采样率: 16kHz
- 位深: 16-bit

---

### Step 5: 实现流式输出（2-3 小时）

**任务**:
- 完整音频生成
- Chunk 分割逻辑
- 实现 `synthesize()` 方法
- 实现 `close()` 方法

**交付物**:
- 流式输出实现
- `TtsStreamChunk` 生成

**技术要点**:
- 每个 chunk 大小: 建议 1024-4096 样本（约 64-256ms @ 16kHz）
- `is_last` 标志：最后一个 chunk 设置为 `true`
- `timestamp_ms`：基于采样率和 chunk 索引计算

---

### Step 6: 创建 Stub 实现（1 小时）

**任务**:
- 实现 `TtsStub`（用于测试）
- 返回空音频或测试音频

**交付物**:
- `core/engine/src/tts_streaming/stub.rs`

---

### Step 7: 集成到 CoreEngine（2-3 小时）

**任务**:
- 在 `translate_and_publish()` 后调用 TTS
- 发布 TTS 事件到 EventBus
- 处理 TTS 错误

**交付物**:
- `CoreEngine::synthesize_and_publish_tts()` 方法
- TTS 事件发布

**集成点**:
```rust
// 在 translate_and_publish() 后
let tts_request = TtsRequest {
    text: translation_response.translated_text.clone(),
    voice: "default".to_string(),
    locale: target_language.clone(),
};
let tts_chunk = self.tts.synthesize(tts_request).await?;
self.publish_tts_event(&tts_chunk, timestamp_ms).await?;
```

---

## 3. 测试计划

### 3.1 单元测试（4 个测试文件）

#### 3.1.1 文本预处理测试
- 数字转文字（中英文）
- 音素转换
- 音素 ID 映射

#### 3.1.2 FastSpeech2 推理测试
- 模型加载
- 输入准备
- 推理执行
- Mel-spectrogram 输出验证

#### 3.1.3 HiFiGAN 推理测试
- 模型加载
- Mel-spectrogram → 音频转换
- 音频格式验证（16-bit, 16kHz）

#### 3.1.4 完整 TTS 流程测试
- 中文文本合成
- 英文文本合成
- 流式输出（chunk 分割）

### 3.2 集成测试

#### 3.2.1 端到端业务流程测试
- ASR → NMT → TTS 完整流程
- 事件发布验证

#### 3.2.2 音频验证测试
- WAV 文件生成
- 格式验证（采样率、位深、声道）
- 手动播放测试（音频质量）

### 3.3 性能测试

#### 3.3.1 推理延迟
- 目标: < 500ms（短文本），< 2s（长文本）

#### 3.3.2 内存使用
- 目标: < 500MB（单次推理）

#### 3.3.3 并发性能
- 目标: 支持至少 2-3 个并发请求

### 3.4 错误处理测试

- 空文本、超长文本
- 不支持的语言
- 模型加载失败
- ONNX 推理错误

---

## 4. 风险评估和缓解措施

### 4.1 技术风险

#### 风险 1: 模型兼容性
- **描述**: FastSpeech2/HiFiGAN 模型的 ONNX IR 版本可能不兼容 `ort` 1.16.3
- **概率**: 中
- **影响**: 高（阻塞实现）
- **缓解措施**:
  - 实现前先验证模型兼容性（使用 Python 脚本）
  - 如不兼容，考虑模型转换或升级 `ort`（需评估影响）

#### 风险 2: 文本预处理复杂性
- **描述**: 中文拼音/音素转换可能需要复杂的外部库
- **概率**: 高
- **影响**: 中（可能延长实现时间）
- **缓解措施**:
  - 先实现简单规则引擎
  - 后续可集成专业库（如 `pinyin`）

#### 风险 3: 音频质量
- **描述**: 生成的音频质量可能不满足要求
- **概率**: 低
- **影响**: 中（需要调优）
- **缓解措施**:
  - 使用已验证的模型（FastSpeech2 + HiFiGAN）
  - 实现后进行音频质量测试

### 4.2 时间风险

#### 风险 4: 实现时间超期
- **描述**: 预计 3-4 天，可能延长到 5-6 天
- **概率**: 中
- **影响**: 低（不影响其他模块）
- **缓解措施**:
  - 分阶段实现（先实现核心功能，再优化）
  - 如时间紧张，可先实现 Stub，后续完善

### 4.3 依赖风险

#### 风险 5: 外部依赖冲突
- **描述**: 新增依赖可能与现有依赖冲突
- **概率**: 低
- **影响**: 中（需要解决冲突）
- **缓解措施**:
- 尽量复用现有依赖（`ort`, `ndarray`）
- 如必须新增依赖，先评估兼容性

---

## 5. 成功标准

### 5.1 功能标准

- ✅ 支持中文和英文文本转语音
- ✅ 输出 PCM 格式音频（16-bit, 16kHz, mono）
- ✅ 支持流式输出（chunk 分割）
- ✅ 集成到 CoreEngine 完整业务流程

### 5.2 性能标准

- ✅ 推理延迟 < 500ms（短文本），< 2s（长文本）
- ✅ 内存使用 < 500MB（单次推理）
- ✅ 支持至少 2-3 个并发请求

### 5.3 质量标准

- ✅ 所有单元测试通过
- ✅ 所有集成测试通过
- ✅ 音频质量人工验证通过（清晰、自然、可理解）

---

## 6. 资源需求

### 6.1 人力资源

- **开发人员**: 1 人
- **预计时间**: 3-4 天（22-32 小时）

### 6.2 技术资源

- **开发环境**: Rust 1.81+, Windows/Linux/macOS
- **测试环境**: Linux/macOS（推荐，避免 Windows 链接问题）
- **模型文件**: 已准备（`models/tts/`）

### 6.3 外部依赖

- **可选**: 中文拼音库（如 `pinyin`）
- **可选**: 英文音素词典（如 CMUdict）

---

## 7. 时间表

| 阶段 | 任务 | 预计时间 | 累计时间 |
|------|------|----------|----------|
| 阶段 1 | Step 1-2: 基础结构和文本预处理 | 6-9 小时 | 6-9 小时 |
| 阶段 2 | Step 3-4: FastSpeech2 和 HiFiGAN 推理 | 7-10 小时 | 13-19 小时 |
| 阶段 3 | Step 5-7: 流式输出和集成 | 5-7 小时 | 18-26 小时 |
| 阶段 4 | 测试和优化 | 4-6 小时 | **22-32 小时** |

**总计**: **3-4 天**

---

## 8. 后续优化（可选）

### 8.1 性能优化

- 模型量化（int8）
- KV Cache 优化（如果适用）
- 批处理推理

### 8.2 功能扩展

- 支持更多语言
- 支持更多语音风格
- 真正的流式生成（实时 chunk 输出）

### 8.3 质量提升

- 音频后处理（降噪、均衡）
- 情感语音合成（结合 Emotion 模块）
- 个性化语音（结合 Persona 模块）

---

## 9. 审核要点

### 9.1 技术方案审核

- [ ] FastSpeech2 + HiFiGAN 方案是否合适？
- [ ] ONNX 模型兼容性是否已验证？
- [ ] 文本预处理方案是否可行？

### 9.2 实现计划审核

- [ ] 实现步骤是否合理？
- [ ] 时间估算是否准确？
- [ ] 测试计划是否充分？

### 9.3 风险评估审核

- [ ] 风险识别是否全面？
- [ ] 缓解措施是否有效？
- [ ] 是否有遗漏的风险？

### 9.4 资源需求审核

- [ ] 人力资源是否充足？
- [ ] 技术资源是否满足？
- [ ] 外部依赖是否可控？

---

## 10. 附录

### 10.1 模型文件信息

**FastSpeech2**:
- 输入: `xs: [1, seq_len, 384]` (音素 ID 序列)
- 输出: `transpose_66.tmp_0: [1, 80, time_steps]` (mel-spectrogram)

**HiFiGAN**:
- 输入: `xs: [1, 80, time_steps]` (mel-spectrogram)
- 输出: `transpose_66.tmp_0: [1, audio_samples]` (音频波形)

### 10.2 相关文档

- `TTS_IMPLEMENTATION_PLAN.md`: 详细实现计划
- `TTS_TEST_CHECKLIST.md`: 详细测试清单
- `COMPLETED_FUNCTIONALITY_SUMMARY.md`: 已完成功能总结

---

**文档版本**: 1.0  
**创建日期**: 2024-12-19  
**审核状态**: 待审核  
**审核部门**: 评估部门

