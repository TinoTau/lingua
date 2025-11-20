# 全面功能测试指南

**鏈€鍚庢洿鏂?*: 2024-12-19

---


**������**: 2024-12-19

---

# 全面功能测试指南

本文档列出了所有测试文件及其对应的测试命令，用于全面验证项目功能�?

## ⚠️ 重要提示：测试命令格�?

**关键区别**�?
- �?`cargo test --lib test_name` - 只运�?`src/lib.rs` 中的单元测试
- �?`cargo test test_name` - 运行所有匹配的测试（包�?`tests/` 目录下的集成测试�?

**本项目的大部分测试都�?`tests/` 目录下，所以应该使�?`cargo test` 而不�?`cargo test --lib`**

---

## 测试分类

### 1. 基础编译和依赖测试（优先级：P0�?

**目的**：确保代码能正常编译，依赖库正常工作

```powershell
# 测试 1.1: 简单编译测�?
cargo test test_compile -- --nocapture

# 测试 1.2: ASR Whisper 依赖测试
cargo test asr_whisper_dependency_test -- --nocapture

# 测试 1.3: ONNX Runtime 环境测试
cargo test nmt_onnx_env -- --nocapture
```

---

### 2. TTS 模块测试（优先级：P0 - 新功能）

**目的**：验�?TTS 模块的各个组�?

```powershell
# 测试 2.1: TTS 文本处理器加�?
cargo test test_text_processor_load -- --nocapture

# 测试 2.2: TTS 文本处理器功能（运行所�?TTS 文本处理器测试）
cargo test tts_text_processor_test -- --nocapture

# 测试 2.3: TTS 模型加载
cargo test test_tts_model_load -- --nocapture

# 测试 2.4: TTS 中文合成（需要模型文件）
cargo test test_tts_synthesize_chinese -- --nocapture

# 测试 2.5: TTS 英文合成（需要模型文件）
cargo test test_tts_synthesize_english -- --nocapture

# 测试 2.6: TTS 空文本处�?
cargo test test_tts_empty_text -- --nocapture
```

---

### 3. ASR 模块测试（优先级：P0�?

**目的**：验证语音识别功�?

```powershell
# 测试 3.1: ASR Whisper 模型加载
cargo test asr_whisper_model_load_test -- --nocapture

# 测试 3.2: ASR Whisper 简单测�?
cargo test asr_whisper_simple_test -- --nocapture

# 测试 3.3: ASR Whisper 引擎测试
cargo test asr_whisper_engine_test -- --nocapture

# 测试 3.4: ASR 音频预处�?
cargo test asr_whisper_audio_preprocessing_test -- --nocapture

# 测试 3.5: ASR 语言设置
cargo test asr_whisper_language_test -- --nocapture

# 测试 3.6: ASR 流式推理
cargo test asr_whisper_streaming_test -- --nocapture

# 测试 3.7: ASR 转录测试（需要音频文件）
cargo test asr_whisper_transcribe_test -- --nocapture

# 测试 3.8: ASR 集成测试
cargo test asr_whisper_integration_test -- --nocapture
```

---

### 4. NMT 模块测试（优先级：P0�?

**目的**：验证机器翻译功�?

```powershell
# 测试 4.1: NMT ONNX 模型加载
cargo test nmt_onnx_model_load -- --nocapture

# 测试 4.2: NMT 快速测�?
cargo test nmt_quick_test -- --nocapture

# 测试 4.3: NMT Decoder 单步测试
cargo test nmt_onnx_decoder_step -- --nocapture

# 测试 4.4: NMT Tokenizer 多语言测试
cargo test nmt_tokenizer_multi_lang -- --nocapture

# 测试 4.5: NMT 完整翻译测试
cargo test nmt_translate_full -- --nocapture

# 测试 4.6: NMT 综合测试
cargo test nmt_comprehensive_test -- --nocapture
```

---

### 5. Emotion 模块测试（优先级：P1�?

**目的**：验证情感分析功�?

```powershell
# 测试 5.1: Emotion Stub 测试
cargo test test_emotion_stub -- --nocapture

# 测试 5.2: Emotion 模型加载
cargo test test_xlmr_emotion_engine_load -- --nocapture

# 测试 5.3: Emotion 推理测试（需要模型文件）
cargo test test_xlmr_emotion_inference -- --nocapture

# 测试 5.4: Emotion 多文本测�?
cargo test test_xlmr_emotion_multiple_texts -- --nocapture
```

---

### 6. Persona 模块测试（优先级：P1�?

**目的**：验证个性化功能

```powershell
# 测试 6.1: Persona Stub 测试
cargo test test_persona_stub -- --nocapture

# 测试 6.2: Persona 规则基础测试（英文正式）
cargo test test_rule_based_formal_english -- --nocapture

# 测试 6.3: Persona 规则基础测试（英文随意）
cargo test test_rule_based_casual_english -- --nocapture

# 测试 6.4: Persona 规则基础测试（英文友好）
cargo test test_rule_based_friendly_english -- --nocapture

# 测试 6.5: Persona 规则基础测试（中文正式）
cargo test test_rule_based_formal_chinese -- --nocapture

# 测试 6.6: Persona 规则基础测试（中文随意）
cargo test test_rule_based_casual_chinese -- --nocapture

# 测试 6.7: Persona 规则基础测试（中文友好）
cargo test test_rule_based_friendly_chinese -- --nocapture

# 测试 6.8: Persona 多组合测�?
cargo test test_rule_based_multiple_combinations -- --nocapture
```

---

### 7. 模块集成测试（优先级：P0�?

**目的**：验证各模块之间的集�?

```powershell
# 测试 7.1: ASR + VAD 集成测试
cargo test asr_vad_integration_test -- --nocapture

# 测试 7.2: ASR + Bootstrap 集成测试
cargo test asr_bootstrap_integration -- --nocapture

# 测试 7.3: NMT + Bootstrap 集成测试
cargo test nmt_bootstrap_integration -- --nocapture

# 测试 7.4: ASR 流式部分结果测试
cargo test asr_streaming_partial_test -- --nocapture
```

---

### 8. 端到端业务流程测试（优先级：P0�?

**目的**：验证完整的业务流程（ASR �?Emotion �?Persona �?NMT �?TTS�?

```powershell
# 测试 8.1: 业务流程端到端测试（完整流程�?
cargo test business_flow_e2e_test -- --nocapture

# 测试 8.2: 业务流程分步测试（逐步验证�?
cargo test business_flow_step_by_step_test -- --nocapture

# 测试 8.3: 全栈冒烟测试
cargo test full_stack_smoke -- --nocapture

# 测试 8.4: 简单冒烟测�?
cargo test smoke -- --nocapture
```

---

## 快速测试命令（按优先级�?

### 最优先测试（确保核心功能正常）

```powershell
# 1. 基础编译
cargo test test_compile -- --nocapture

# 2. TTS 文本处理器（新功能）
cargo test test_text_processor_load -- --nocapture

# 3. ASR 模型加载
cargo test asr_whisper_model_load_test -- --nocapture

# 4. NMT 模型加载
cargo test nmt_onnx_model_load -- --nocapture

# 5. 端到端业务流程（最重要�?
cargo test business_flow_e2e_test -- --nocapture
```

### 完整测试套件（按模块顺序�?

```powershell
# 一次性运行所有测试（可能需要较长时间）
cargo test -- --nocapture

# 或者按模块分别运行�?

# TTS 模块（所�?TTS 相关测试�?
cargo test tts -- --nocapture

# ASR 模块（所�?ASR 相关测试�?
cargo test asr -- --nocapture

# NMT 模块（所�?NMT 相关测试�?
cargo test nmt -- --nocapture

# Emotion 模块
cargo test emotion -- --nocapture

# Persona 模块
cargo test persona -- --nocapture

# 业务流程
cargo test business -- --nocapture
```

---

## 测试文件说明

### 需要模型文件的测试

以下测试需要相应的模型文件才能运行，如果模型文件不存在，测试会自动跳过�?

- **TTS 测试**：需�?`models/tts/` 目录下的模型文件
- **ASR 测试**：需�?`models/asr/` 目录下的 Whisper 模型文件
- **NMT 测试**：需�?`models/nmt/marian-*/` 目录下的模型文件
- **Emotion 测试**：需�?`models/emotion/xlm-r/` 目录下的模型文件

### 需要音频文件的测试

- `asr_whisper_transcribe_test.rs`：需�?`third_party/` 目录下的音频文件

---

## 测试结果解读

- �?**测试通过**：功能正�?
- ⚠️ **测试跳过**：通常是因为模型文件不存在，这是正常的
- �?**测试失败**：需要检查错误信息并修复
- **`running 0 tests`**：通常是因为使用了 `--lib` 标志，应该使�?`cargo test` 而不�?`cargo test --lib`

---

## 常见问题

### Q: 为什么显�?`running 0 tests`�?

**A**: 这是因为使用�?`cargo test --lib` 而不�?`cargo test`。`--lib` 只运�?`src/lib.rs` 中的单元测试，不运行 `tests/` 目录下的集成测试�?

**解决方案**：去�?`--lib` 标志，使�?`cargo test test_name -- --nocapture`

### Q: 如何查看所有可用的测试�?

**A**: 运行 `cargo test -- --list` 可以列出所有测�?

### Q: 如何只运行特定文件中的测试？

**A**: 使用测试函数名称的一部分，例如：
- `cargo test tts_text_processor` - 运行所有包�?"tts_text_processor" 的测�?
- `cargo test asr_whisper` - 运行所有包�?"asr_whisper" 的测�?

---

## 注意事项

1. **Windows 环境**：某些测试可能在 Windows 上需要额外的配置
2. **模型文件路径**：确保模型文件在正确的路径下
3. **编译时间**：首次编译可能需要较长时�?
4. **链接器错�?*：如果遇�?MSVC 链接器错误，请参�?`WINDOWS_RUNTIME_LIBRARY_MISMATCH_FIX.md`
5. **测试命令格式**：记住使�?`cargo test` 而不�?`cargo test --lib` 来运行集成测�?
