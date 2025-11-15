# ASR Whisper 实现计划

## 当前状态

### 已完成
- ✅ `AsrStreaming` trait 定义（`core/engine/src/asr_streaming/mod.rs`）
- ✅ `AsrEngine` trait 定义（`core/engine/src/asr_whisper/mod.rs`）
- ✅ CLI 工具（`core/engine/src/asr_whisper/cli.rs`）
- ✅ Whisper 模型文件存在（`core/engine/models/asr/whisper-base/`）

### 待完成
- ❌ 实现 `AsrStreaming` trait（流式推理）
- ❌ 集成 Whisper 推理引擎
- ❌ 音频预处理（PCM/浮点 → Whisper 输入格式）
- ❌ 流式推理实现
- ❌ 测试用例

---

## 技术方案选择

### 方案对比

#### 方案 1: whisper.cpp（推荐）
**优点**:
- ✅ C++ 实现，性能优秀
- ✅ 支持流式推理
- ✅ 模型格式：GGML/GGUF（量化，体积小）
- ✅ 跨平台支持（Windows/Linux/macOS）
- ✅ 有 Rust 绑定（`whisper-rs` crate）

**缺点**:
- ⚠️ 需要编译 C++ 代码（或使用预编译二进制）
- ⚠️ 模型格式需要转换（HuggingFace → GGML/GGUF）

**适用场景**: 生产环境，性能要求高

#### 方案 2: FasterWhisper（Python）
**优点**:
- ✅ Python 实现，易于集成
- ✅ 支持 ONNX Runtime
- ✅ 模型格式：ONNX（与 NMT 一致）

**缺点**:
- ❌ 需要 Python 运行时
- ❌ 性能不如 whisper.cpp
- ❌ 流式推理支持有限

**适用场景**: 快速原型，开发阶段

#### 方案 3: ONNX Runtime（直接使用 ONNX 模型）
**优点**:
- ✅ 与 NMT 使用相同的推理引擎
- ✅ 模型格式统一（ONNX）
- ✅ 无需额外依赖

**缺点**:
- ❌ 需要手动实现 Whisper 的预处理和后处理
- ❌ 流式推理需要自己实现
- ❌ 实现复杂度高

**适用场景**: 如果已有 ONNX 模型，且希望统一推理引擎

---

## 推荐方案：whisper.cpp + whisper-rs

**理由**:
1. 性能最优，适合生产环境
2. 原生支持流式推理
3. 有成熟的 Rust 绑定
4. 模型体积小（GGML/GGUF 量化）

---

## 实现步骤拆分

### 阶段 1: 环境准备和依赖集成（1-2 天）

#### 步骤 1.1: 添加 whisper-rs 依赖
**目标**: 在 `Cargo.toml` 中添加 `whisper-rs` crate

**任务**:
- [ ] 研究 `whisper-rs` crate 的 API 和用法
- [ ] 添加依赖到 `core/engine/Cargo.toml`
- [ ] 处理可能的编译问题（C++ 链接等）

**验收标准**:
- `cargo build` 能成功编译
- 能够导入 `whisper-rs` 的类型和函数

**文件**:
- `core/engine/Cargo.toml`

---

#### 步骤 1.2: 准备 Whisper 模型（GGML/GGUF 格式）
**目标**: 将 HuggingFace 的 Whisper 模型转换为 GGML/GGUF 格式

**任务**:
- [ ] 研究模型转换工具（`convert-h5-to-ggml.py` 或 `convert-pt-to-ggml.py`）
- [ ] 创建转换脚本（`scripts/convert_whisper_to_ggml.py`）
- [ ] 转换 `whisper-base.en` 模型
- [ ] 验证转换后的模型能正常加载

**验收标准**:
- 模型文件存在于 `core/engine/models/asr/whisper-base/`
- 模型格式为 `.ggml` 或 `.gguf`
- 能够用 `whisper-rs` 加载模型

**文件**:
- `scripts/convert_whisper_to_ggml.py`
- `core/engine/models/asr/whisper-base/model.ggml` 或 `model.gguf`

---

### 阶段 2: 基础推理实现（2-3 天）

#### 步骤 2.1: 实现音频预处理
**目标**: 将 `AudioFrame` 转换为 Whisper 输入格式（mel spectrogram）

**任务**:
- [ ] 研究 Whisper 的音频预处理流程
  - 重采样到 16kHz
  - 转换为 mel spectrogram（80 个 mel bins）
  - 归一化
- [ ] 实现音频预处理函数
  - `resample_audio()`: 重采样
  - `compute_mel_spectrogram()`: 计算 mel spectrogram
  - `normalize_audio()`: 归一化
- [ ] 处理音频格式转换（PCM f32 → Whisper 输入）

**验收标准**:
- 能够将 `AudioFrame` 转换为 Whisper 输入格式
- 输出形状正确：`[n_mel_bins, n_frames]` (80, T)
- 数值范围正确（归一化到 [-1, 1]）

**文件**:
- `core/engine/src/asr_whisper/audio_preprocessing.rs`

**依赖**:
- 可能需要 `rubato` 或 `samplerate` crate（重采样）
- 可能需要 `rustfft` 或 `fftw` crate（FFT）

---

#### 步骤 2.2: 实现基础 Whisper 推理
**目标**: 使用 `whisper-rs` 进行单次推理（非流式）

**任务**:
- [ ] 创建 `WhisperAsrEngine` 结构体
- [ ] 实现模型加载（`new_from_model_path()`）
- [ ] 实现单次推理方法（`transcribe_full()`）
- [ ] 处理 Whisper 输出（token IDs → 文本）
- [ ] 集成 tokenizer（Whisper 使用 GPT-2 tokenizer）

**验收标准**:
- 能够加载 GGML/GGUF 模型
- 能够对完整音频进行推理
- 输出正确的转录文本

**文件**:
- `core/engine/src/asr_whisper/engine.rs`

**参考**:
- `whisper-rs` 的示例代码
- NMT 的模型加载和推理实现

---

#### 步骤 2.3: 实现 `AsrStreaming` trait（基础版本）
**目标**: 实现 `AsrStreaming` trait，支持完整音频推理

**任务**:
- [ ] 为 `WhisperAsrEngine` 实现 `AsrStreaming` trait
- [ ] 实现 `initialize()`: 加载模型
- [ ] 实现 `infer()`: 
  - 收集 `AudioFrame` 到缓冲区
  - 当收到完整音频时，进行推理
  - 返回 `AsrResult`
- [ ] 实现 `finalize()`: 清理资源

**验收标准**:
- 能够通过 `AsrStreaming` trait 调用 Whisper 推理
- 能够处理多个 `AudioFrame` 并返回转录结果
- 能够正确返回 `PartialTranscript` 和 `StableTranscript`

**文件**:
- `core/engine/src/asr_whisper/streaming.rs`

---

### 阶段 3: 流式推理实现（3-4 天）

#### 步骤 3.1: 实现音频缓冲区管理
**目标**: 管理音频帧缓冲区，支持滑动窗口

**任务**:
- [ ] 创建 `AudioBuffer` 结构体
- [ ] 实现音频帧累积（`push_frame()`）
- [ ] 实现滑动窗口（保留最近 N 秒的音频）
- [ ] 实现音频拼接和格式转换

**验收标准**:
- 能够累积多个 `AudioFrame`
- 能够维护固定大小的滑动窗口
- 能够将缓冲区转换为连续音频数组

**文件**:
- `core/engine/src/asr_whisper/audio_buffer.rs`

---

#### 步骤 3.2: 实现流式推理逻辑
**目标**: 使用 Whisper 的流式推理 API

**任务**:
- [ ] 研究 `whisper-rs` 的流式推理 API
- [ ] 实现增量推理（每次处理新的音频块）
- [ ] 处理上下文窗口（保留历史上下文）
- [ ] 实现部分结果输出（`PartialTranscript`）
- [ ] 实现最终结果输出（`StableTranscript`）

**验收标准**:
- 能够实时处理音频流
- 能够输出部分转录结果（`PartialTranscript`）
- 能够在句子结束时输出最终结果（`StableTranscript`）
- 延迟低（< 1 秒）

**文件**:
- `core/engine/src/asr_whisper/streaming.rs`（更新）

---

#### 步骤 3.3: 实现 VAD 集成（可选）
**目标**: 与 VAD 模块集成，在语音边界触发推理

**任务**:
- [ ] 研究 VAD 模块的接口
- [ ] 在检测到语音边界时触发推理
- [ ] 处理静音段的跳过
- [ ] 优化推理时机（避免频繁推理）

**验收标准**:
- 能够在语音边界自动触发推理
- 能够跳过静音段
- 推理时机合理（不会过于频繁）

**文件**:
- `core/engine/src/asr_whisper/streaming.rs`（更新）

---

### 阶段 4: 测试和优化（2-3 天）

#### 步骤 4.1: 单元测试
**目标**: 为各个模块编写单元测试

**任务**:
- [ ] 测试音频预处理（重采样、mel spectrogram）
- [ ] 测试模型加载
- [ ] 测试单次推理
- [ ] 测试流式推理
- [ ] 测试 `AsrStreaming` trait 实现

**验收标准**:
- 所有单元测试通过
- 测试覆盖率 > 80%

**文件**:
- `core/engine/tests/asr_whisper_audio_preprocessing.rs`
- `core/engine/tests/asr_whisper_engine.rs`
- `core/engine/tests/asr_whisper_streaming.rs`

---

#### 步骤 4.2: 集成测试
**目标**: 测试完整的 ASR 流程

**任务**:
- [ ] 测试从 WAV 文件到转录文本的完整流程
- [ ] 测试流式推理的端到端流程
- [ ] 测试与 `CoreEngine` 的集成
- [ ] 性能测试（延迟、吞吐量）

**验收标准**:
- 能够处理真实音频文件
- 转录准确率 > 90%（在测试集上）
- 延迟 < 1 秒（流式推理）
- 内存使用合理

**文件**:
- `core/engine/tests/asr_whisper_integration.rs`

---

#### 步骤 4.3: 错误处理和优化
**目标**: 完善错误处理和性能优化

**任务**:
- [ ] 添加错误处理（模型加载失败、推理失败等）
- [ ] 优化内存使用（避免不必要的拷贝）
- [ ] 优化推理性能（批处理、并行等）
- [ ] 添加日志和监控

**验收标准**:
- 所有错误情况都有适当的处理
- 内存使用稳定（无泄漏）
- 性能满足要求

**文件**:
- 所有相关文件

---

### 阶段 5: 集成到 CoreEngine（1 天）

#### 步骤 5.1: 更新 CoreEngineBuilder
**目标**: 添加 `asr_with_default_whisper()` 方法

**任务**:
- [ ] 在 `CoreEngineBuilder` 中添加 `asr_with_default_whisper()`
- [ ] 自动加载默认 Whisper 模型
- [ ] 处理模型路径和配置

**验收标准**:
- 能够通过 `CoreEngineBuilder` 使用 Whisper ASR
- 模型路径正确
- 配置合理

**文件**:
- `core/engine/src/bootstrap.rs`

---

#### 步骤 5.2: 端到端测试
**目标**: 测试 ASR → NMT → TTS 完整流程

**任务**:
- [ ] 创建端到端测试
- [ ] 测试音频输入 → 转录 → 翻译 → 合成
- [ ] 验证结果正确性

**验收标准**:
- 完整流程能够正常工作
- 结果正确

**文件**:
- `core/engine/tests/asr_nmt_tts_e2e.rs`

---

## 详细任务清单

### 优先级 P0（必须完成）
1. ✅ 步骤 1.1: 添加 whisper-rs 依赖
2. ✅ 步骤 1.2: 准备 Whisper 模型（GGML/GGUF）
3. ✅ 步骤 2.1: 实现音频预处理
4. ✅ 步骤 2.2: 实现基础 Whisper 推理
5. ✅ 步骤 2.3: 实现 `AsrStreaming` trait（基础版本）
6. ✅ 步骤 4.1: 单元测试
7. ✅ 步骤 5.1: 更新 CoreEngineBuilder

### 优先级 P1（重要）
8. ✅ 步骤 3.1: 实现音频缓冲区管理
9. ✅ 步骤 3.2: 实现流式推理逻辑
10. ✅ 步骤 4.2: 集成测试

### 优先级 P2（可选）
11. ⚠️ 步骤 3.3: 实现 VAD 集成
12. ⚠️ 步骤 4.3: 错误处理和优化
13. ⚠️ 步骤 5.2: 端到端测试

---

## 技术细节

### 音频预处理流程
```
AudioFrame (PCM f32, 任意采样率)
  ↓
重采样到 16kHz
  ↓
计算 mel spectrogram (80 bins)
  ↓
归一化到 [-1, 1]
  ↓
Whisper 输入格式 [80, T]
```

### Whisper 推理流程
```
Mel Spectrogram [80, T]
  ↓
Encoder (Transformer)
  ↓
Decoder (Transformer, 自回归)
  ↓
Token IDs
  ↓
Tokenizer 解码
  ↓
文本输出
```

### 流式推理策略
1. **滑动窗口**: 维护最近 N 秒的音频（如 30 秒）
2. **增量推理**: 每次处理新的音频块（如 1 秒）
3. **上下文保留**: 保留历史上下文，避免重复计算
4. **部分结果**: 在句子中间输出部分转录
5. **最终结果**: 在检测到句子结束时输出最终转录

---

## 依赖项

### Rust Crates
- `whisper-rs`: Whisper 推理引擎绑定
- `rubato` 或 `samplerate`: 音频重采样
- `rustfft` 或 `fftw`: FFT 计算（mel spectrogram）
- `ndarray`: 数组操作（与 NMT 一致）

### 系统依赖
- C++ 编译器（用于编译 whisper.cpp）
- CMake（用于构建 whisper.cpp）

---

## 风险评估

### 高风险
1. **模型转换**: HuggingFace → GGML/GGUF 转换可能失败
   - **缓解**: 使用官方转换工具，提前测试

2. **音频预处理**: mel spectrogram 计算可能不准确
   - **缓解**: 参考 Whisper 官方实现，使用标准库

3. **流式推理**: `whisper-rs` 的流式 API 可能不完善
   - **缓解**: 研究 `whisper-rs` 文档和示例，必要时自己实现

### 中风险
1. **性能**: 推理延迟可能过高
   - **缓解**: 使用量化模型，优化推理流程

2. **内存**: 模型和缓冲区可能占用大量内存
   - **缓解**: 使用量化模型，限制缓冲区大小

---

## 时间估算

| 阶段 | 任务数 | 预计时间 | 累计时间 |
|------|--------|----------|----------|
| 阶段 1: 环境准备 | 2 | 1-2 天 | 1-2 天 |
| 阶段 2: 基础推理 | 3 | 2-3 天 | 3-5 天 |
| 阶段 3: 流式推理 | 2-3 | 3-4 天 | 6-9 天 |
| 阶段 4: 测试优化 | 3 | 2-3 天 | 8-12 天 |
| 阶段 5: 集成 | 2 | 1 天 | 9-13 天 |

**总计**: 9-13 个工作日（约 2-3 周）

---

## 下一步行动

1. **立即开始**: 步骤 1.1（添加 whisper-rs 依赖）
2. **并行进行**: 步骤 1.2（准备模型）
3. **完成后**: 开始阶段 2（基础推理实现）

---

## 参考资料

- [whisper-rs GitHub](https://github.com/tazz4843/whisper-rs)
- [whisper.cpp GitHub](https://github.com/ggerganov/whisper.cpp)
- [Whisper 论文](https://arxiv.org/abs/2212.04356)
- [Whisper 官方实现](https://github.com/openai/whisper)

