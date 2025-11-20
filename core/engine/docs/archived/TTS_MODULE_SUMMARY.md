# TTS 模块功能、架构与测试总结

**文档版本**: 1.0  
**更新时间**: 2024-12-19  
**状态**: 代码实现完成，待测试验证

---

## 📋 目录

1. [功能概述](#功能概述)
2. [架构设计](#架构设计)
3. [实现状态](#实现状态)
4. [测试情况](#测试情况)
5. [遇到的问题](#遇到的问题)
6. [下一步计划](#下一步计划)

---

## 功能概述

### 核心功能

TTS（Text-to-Speech）模块负责将文本转换为语音音频，支持：

- ✅ **多语言支持**: 中文（zh）和英文（en）
- ✅ **流式输出**: 支持流式音频 chunk 输出
- ✅ **ONNX 推理**: 基于 ONNX Runtime 的模型推理
- ✅ **PCM 音频**: 输出 16-bit PCM 格式音频（16kHz, mono）

### 技术方案

**架构**: FastSpeech2 (声学模型) + HiFiGAN (声码器)

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
TtsStreamChunk 输出
```

### 模型文件

- **FastSpeech2**:
  - 中文: `models/tts/fastspeech2-lite/fastspeech2_csmsc_streaming.onnx`
  - 英文: `models/tts/fastspeech2-lite/fastspeech2_ljspeech.onnx`
- **HiFiGAN**:
  - 中文: `models/tts/hifigan-lite/hifigan_csmsc.onnx`
  - 英文: `models/tts/hifigan-lite/hifigan_ljspeech.onnx`
- **辅助文件**:
  - `phone_id_map.txt`: 音素到 ID 的映射
  - `speech_stats.npy`: Mel-spectrogram 统计信息（待使用）

---

## 架构设计

### 模块结构

```
core/engine/src/tts_streaming/
├── mod.rs                 # 模块声明和 trait 定义
├── fastspeech2_tts.rs     # FastSpeech2 + HiFiGAN 引擎实现
├── text_processor.rs      # 文本预处理（规范化、音素转换）
├── audio_utils.rs         # 音频工具（WAV 保存、验证等）
└── stub.rs                # Stub 实现（用于测试和开发）
```

### 核心接口

#### `TtsStreaming` Trait

```rust
#[async_trait]
pub trait TtsStreaming: Send + Sync {
    async fn synthesize(&self, request: TtsRequest) -> EngineResult<TtsStreamChunk>;
    async fn close(&self) -> EngineResult<()>;
}
```

#### `TtsRequest`

```rust
pub struct TtsRequest {
    pub text: String,      // 要合成的文本
    pub voice: String,     // 语音类型（预留）
    pub locale: String,    // 语言代码（"zh" 或 "en"）
}
```

#### `TtsStreamChunk`

```rust
pub struct TtsStreamChunk {
    pub audio: Vec<u8>,        // PCM 16-bit 音频数据
    pub timestamp_ms: u64,     // 时间戳（毫秒）
    pub is_last: bool,         // 是否是最后一个 chunk
}
```

### 实现类

#### 1. `FastSpeech2TtsEngine`

**功能**: 完整的 TTS 引擎实现

**特性**:
- 加载 FastSpeech2 和 HiFiGAN 模型（中英文）
- 文本预处理 → 音素 ID
- FastSpeech2 推理 → Mel-spectrogram
- HiFiGAN 推理 → 音频波形
- PCM 16-bit 转换

**关键方法**:
- `new_from_dir(model_dir: &Path) -> Result<Self>`: 从模型目录加载引擎
- `run_fastspeech2(phone_ids: &[i64], locale: &str) -> Result<Array3<f32>>`: FastSpeech2 推理
- `run_hifigan(mel: &Array3<f32>, locale: &str) -> Result<Array1<f32>>`: HiFiGAN 推理
- `audio_to_pcm16(audio: &Array1<f32>) -> Vec<u8>`: 音频格式转换

#### 2. `TtsStub`

**功能**: 占位实现，用于测试和开发

**特性**:
- 返回空音频 chunk
- 不依赖模型文件
- 用于单元测试和开发环境

#### 3. `TextProcessor`

**功能**: 文本预处理

**特性**:
- 文本规范化（去除多余空格、标点处理）
- 音素转换（文本 → 音素序列）
- 音素 ID 映射（使用 `phone_id_map.txt`）

**当前限制**:
- ⚠️ 中文拼音转换：简化实现（字符映射）
- ⚠️ 英文音素转换：简化实现（单词映射）
- ⚠️ 数字转文字：未实现
- ⚠️ 日期时间处理：未实现

### 集成状态

#### CoreEngine 集成

**已集成**:
- ✅ `CoreEngine` 结构体包含 `tts: Arc<dyn TtsStreaming>` 字段
- ✅ `CoreEngineBuilder` 支持 `tts()` 方法设置 TTS 引擎
- ✅ `CoreEngine::shutdown()` 调用 `tts.close()`

**未集成**:
- ❌ `process_audio_frame()` 中未调用 TTS
- ❌ NMT 输出 → TTS 输入的流程未实现
- ❌ TTS 输出 → 音频播放的流程未实现

---

## 实现状态

### ✅ 已完成

#### 1. 基础结构
- ✅ `FastSpeech2TtsEngine` 结构体定义
- ✅ 模型加载逻辑（FastSpeech2 和 HiFiGAN，中英文）
- ✅ `TtsStreaming` trait 实现
- ✅ `TtsStub` 占位实现

#### 2. 文本预处理
- ✅ `TextProcessor` 结构体
- ✅ `phone_id_map.txt` 加载
- ✅ 文本规范化（基本处理）
- ✅ 音素到 ID 映射
- ✅ 文本到音素转换（简化版）

#### 3. FastSpeech2 推理
- ✅ 音素 ID 输入处理
- ✅ ONNX 模型推理
- ✅ Mel-spectrogram 输出处理
- ✅ 形状验证和转置处理

#### 4. HiFiGAN 推理
- ✅ Mel-spectrogram 输入处理
- ✅ ONNX 模型推理
- ✅ 音频波形输出处理
- ✅ 形状验证（支持 `[batch, samples]` 和 `[samples]`）

#### 5. 音频处理
- ✅ PCM 16-bit 转换
- ✅ 音频 chunk 分割（用于流式输出）
- ✅ WAV 文件保存工具（`audio_utils.rs`）

#### 6. 错误处理
- ✅ 输入验证
- ✅ 形状验证
- ✅ 空数据检查
- ✅ 详细的错误消息

### ⚠️ 待完善

#### 1. 文本预处理（高优先级）
- ⚠️ **中文拼音转换**: 当前只做字符映射，需要实现文本 → 拼音 → 音素
- ⚠️ **英文音素转换**: 当前只做单词映射，需要实现文本 → 音素（使用 CMUdict）
- ⚠️ **数字转文字**: 需要实现 "123" → "一百二十三" 或 "one hundred twenty three"
- ⚠️ **日期时间处理**: 需要规范化日期时间格式
- ⚠️ **缩写展开**: 需要展开常见缩写（如 "Dr." → "Doctor"）

#### 2. FastSpeech2 输入形状（中优先级）
- ⚠️ **Embedding 层**: 当前假设模型接受整数 ID，可能需要预处理 embedding
- ⚠️ **输入形状验证**: 需要根据实际模型验证输入形状是 `[1, seq_len]` 还是 `[1, seq_len, 384]`

#### 3. Mel-spectrogram 归一化（中优先级）
- ⚠️ **统计信息加载**: 需要加载 `speech_stats.npy` 进行归一化
- ⚠️ **归一化/反归一化**: 需要实现 mel-spectrogram 的归一化和反归一化

#### 4. 流式输出（低优先级）
- ⚠️ **真正的流式 chunk**: 当前返回完整音频，未来可以实现真正的流式分割
- ⚠️ **时间戳计算**: 需要根据实际音频长度计算准确的时间戳

#### 5. 集成到 CoreEngine（待实现）
- ⚠️ **Step 7**: 将 TTS 集成到 `CoreEngine` 的 `process_audio_frame` 流程中

---

## 测试情况

### 测试文件

#### 1. `tts_model_load_test.rs`
**状态**: ✅ 已创建  
**功能**: 测试 TTS 模型加载

**测试内容**:
- 测试 `FastSpeech2TtsEngine::new_from_dir()` 是否能成功加载模型

**运行状态**: ❌ 未运行（受编译环境问题影响）

#### 2. `tts_text_processor_test.rs`
**状态**: ✅ 已创建  
**功能**: 测试文本预处理器

**测试内容**:
- `test_text_processor_load`: 测试 TextProcessor 加载
- `test_text_normalization`: 测试文本规范化
- `test_phoneme_to_id_mapping`: 测试音素 ID 映射

**运行状态**: ❌ 未运行（受编译环境问题影响）

#### 3. `tts_integration_test.rs`
**状态**: ✅ 已创建  
**功能**: 测试完整的 TTS 流程

**测试内容**:
- `test_tts_synthesize_chinese`: 测试中文 TTS 合成
- `test_tts_synthesize_english`: 测试英文 TTS 合成
- `test_tts_empty_text`: 测试空文本处理

**运行状态**: ❌ 未运行（受编译环境问题影响）

### 测试覆盖

| 测试类别 | 测试文件 | 状态 | 备注 |
|---------|---------|------|------|
| 模型加载 | `tts_model_load_test.rs` | ✅ 已创建 | 未运行 |
| 文本预处理 | `tts_text_processor_test.rs` | ✅ 已创建 | 未运行 |
| 集成测试 | `tts_integration_test.rs` | ✅ 已创建 | 未运行 |
| 单元测试 | 各模块内部 | ⚠️ 部分 | 部分方法有测试 |
| 端到端测试 | 无 | ❌ 未创建 | 需要集成到 CoreEngine 后测试 |

### 测试结果

**当前状态**: ❌ 所有测试均未运行

**原因**:
1. Windows 环境编译卡住问题
2. 模型文件可能不存在或路径不正确
3. 需要 Linux/macOS 环境进行测试

---

## 遇到的问题

### 🔴 严重问题

#### 1. Windows 编译环境问题

**问题描述**:
- `cargo build` 和 `cargo check` 命令在 Windows 环境下卡住数小时
- Python 测试脚本也会卡住
- 影响代码编译和测试

**可能原因**:
- 防病毒软件实时扫描大文件（.onnx 文件可能 > 1GB）
- 文件系统 I/O 阻塞
- 系统资源不足

**解决方案**:
- ✅ 将项目目录添加到防病毒软件白名单
- ✅ 使用 WSL 或 Linux 环境进行编译和测试
- ✅ 使用 Docker 容器进行开发

**状态**: ⚠️ 部分解决（提供了解决方案，但未验证）

#### 2. 模型文件缺失或路径问题

**问题描述**:
- 模型文件可能不存在于 `models/tts/` 目录
- 路径配置可能不正确

**解决方案**:
- 检查模型文件是否存在
- 验证路径配置
- 提供模型文件下载脚本（如果需要）

**状态**: ❓ 未验证

### ⚠️ 中等问题

#### 3. 文本预处理简化

**问题描述**:
- 中文拼音转换：当前只做字符映射，需要实现真正的拼音转换
- 英文音素转换：当前只做单词映射，需要实现真正的音素转换（CMUdict）

**影响**:
- TTS 输出质量可能较差
- 某些文本可能无法正确转换

**解决方案**:
- 集成第三方库（如 `pypinyin` 用于中文，`CMUdict` 用于英文）
- 或使用 Rust 原生实现

**状态**: ⚠️ 待实现

#### 4. FastSpeech2 输入形状不确定

**问题描述**:
- 不确定模型输入是 `[1, seq_len]` 还是 `[1, seq_len, 384]`
- 可能需要 embedding 预处理

**影响**:
- 推理可能失败
- 需要根据实际模型调整

**解决方案**:
- 通过实际测试验证模型输入输出形状
- 根据模型文档调整代码

**状态**: ⚠️ 待验证

#### 5. Mel-spectrogram 归一化未实现

**问题描述**:
- 未加载 `speech_stats.npy` 进行归一化
- 可能影响音频质量

**影响**:
- 音频质量可能较差
- 需要根据实际模型调整

**解决方案**:
- 加载 `speech_stats.npy`
- 实现归一化和反归一化逻辑

**状态**: ⚠️ 待实现

### 💡 低优先级问题

#### 6. 流式输出未完全实现

**问题描述**:
- 当前返回完整音频作为单个 chunk
- 未实现真正的流式分割

**影响**:
- 延迟较高
- 内存占用较大

**解决方案**:
- 实现真正的流式 chunk 分割
- 根据音频长度和时间戳分割

**状态**: ⚠️ 待优化

#### 7. 未集成到 CoreEngine 业务流程

**问题描述**:
- TTS 未集成到 `process_audio_frame()` 流程
- NMT 输出 → TTS 输入的流程未实现

**影响**:
- 无法在完整业务流程中使用 TTS
- 需要手动调用 TTS

**解决方案**:
- 在 `process_audio_frame()` 中添加 TTS 调用
- 实现 NMT 输出 → TTS 输入的流程

**状态**: ⚠️ 待实现（Step 7）

---

## 下一步计划

### 优先级 1: 解决编译环境问题

1. **验证模型文件**
   - 检查 `models/tts/` 目录是否存在
   - 验证模型文件路径
   - 创建模型文件检查脚本

2. **在 Linux/macOS 环境测试**
   - 使用 WSL 或 Docker
   - 运行所有测试
   - 验证功能正确性

### 优先级 2: 完善文本预处理

1. **中文拼音转换**
   - 集成 `pypinyin` 或 Rust 原生实现
   - 实现文本 → 拼音 → 音素流程

2. **英文音素转换**
   - 集成 CMUdict 或类似词典
   - 实现文本 → 音素流程

3. **数字转文字**
   - 实现中文数字转换
   - 实现英文数字转换

### 优先级 3: 集成到 CoreEngine

1. **在 `process_audio_frame()` 中添加 TTS 调用**
   - 在 NMT 翻译完成后调用 TTS
   - 处理 TTS 输出

2. **实现 NMT 输出 → TTS 输入流程**
   - 将翻译结果传递给 TTS
   - 处理语言代码映射

3. **端到端测试**
   - 测试完整业务流程
   - 验证音频输出质量

### 优先级 4: 优化和增强

1. **Mel-spectrogram 归一化**
   - 加载 `speech_stats.npy`
   - 实现归一化和反归一化

2. **流式输出优化**
   - 实现真正的流式 chunk 分割
   - 优化内存占用

3. **性能优化**
   - 批处理优化
   - 缓存优化

---

## 总结

### 完成度

- **代码实现**: ✅ 85% 完成
- **测试**: ⚠️ 0% 完成（受环境问题影响）
- **集成**: ⚠️ 30% 完成（基础集成完成，业务流程未集成）

### 主要成就

1. ✅ 完整的 TTS 引擎实现（FastSpeech2 + HiFiGAN）
2. ✅ 文本预处理框架（待完善）
3. ✅ 音频处理工具
4. ✅ 完善的错误处理
5. ✅ 测试框架（待运行）

### 主要挑战

1. 🔴 Windows 编译环境问题
2. ⚠️ 文本预处理简化
3. ⚠️ 模型输入形状不确定
4. ⚠️ 未集成到业务流程

### 建议

1. **优先解决编译环境问题**，在 Linux/macOS 环境进行测试
2. **完善文本预处理**，提高 TTS 输出质量
3. **集成到 CoreEngine**，实现完整的业务流程
4. **进行端到端测试**，验证功能正确性

---

**文档维护**: 本文档应随 TTS 模块开发进度更新。

