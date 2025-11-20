# TTS 模块实现计划

**版本**: 1.0  
**创建日期**: 2024-12-19  
**状态**: 📋 计划阶段

---

## 📋 目录

1. [概述](#概述)
2. [需要实现的功能](#需要实现的功能)
3. [技术架构](#技术架构)
4. [实现步骤](#实现步骤)
5. [测试计划](#测试计划)
6. [集成点](#集成点)
7. [已知问题和注意事项](#已知问题和注意事项)

---

## 概述

### 目标

实现基于 **FastSpeech2 + HiFiGAN** 的文本转语音（TTS）模块，支持：
- 文本输入 → 音频输出（PCM 格式）
- 流式音频生成（chunk 拼接）
- 多语言支持（中文、英文）
- 集成到完整业务流程

### 当前状态

- ✅ **Trait 定义**: `TtsStreaming` 已定义
- ✅ **模型文件**: FastSpeech2 和 HiFiGAN 模型已准备
- ❌ **实现**: 尚未实现
- ❌ **测试**: 尚未编写

---

## 需要实现的功能

### 1. 核心功能

#### 1.1 文本预处理
- [ ] **文本规范化**（Text Normalization）
  - 数字转文字（"123" → "一百二十三"）
  - 标点符号处理
  - 缩写展开（"Dr." → "Doctor"）
- [ ] **音素转换**（Phoneme Conversion）
  - 文本 → 音素序列（使用 `phone_id_map.txt`）
  - 支持中文拼音/音素
  - 支持英文音素（如 CMUdict）

#### 1.2 FastSpeech2 推理
- [ ] **模型加载**
  - 加载 `fastspeech2_csmsc_streaming.onnx`（中文）
  - 加载 `fastspeech2_ljspeech.onnx`（英文）
  - 支持按语言选择模型
- [ ] **输入准备**
  - 音素 ID 序列 → 张量
  - 处理动态长度
  - 添加必要的 padding
- [ ] **推理执行**
  - 运行 ONNX 模型
  - 提取 mel-spectrogram 输出
  - 处理 `speech_stats.npy`（归一化统计信息）

#### 1.3 HiFiGAN 推理
- [ ] **模型加载**
  - 加载 `hifigan_csmsc.onnx`（中文）
  - 加载 `hifigan_ljspeech.onnx`（英文）
- [ ] **Vocoder 推理**
  - mel-spectrogram → 音频波形
  - 生成 PCM 格式音频（16-bit, 16kHz）

#### 1.4 流式输出
- [ ] **Chunk 生成**
  - 将完整音频分割为多个 chunk
  - 每个 chunk 包含：`audio: Vec<u8>`, `timestamp_ms: u64`, `is_last: bool`
- [ ] **流式接口**
  - 实现 `synthesize()` 方法返回 `TtsStreamChunk`
  - 支持一次性返回完整音频（`is_last=true`）
  - 未来可扩展为真正的流式生成

### 2. 辅助功能

#### 2.1 模型管理
- [ ] **模型路径管理**
  - 根据 `locale` 选择对应的模型
  - 支持模型热加载（可选）
- [ ] **ONNX Runtime 集成**
  - 使用 `ort` crate（与 NMT/Emotion 一致）
  - 管理 Session 生命周期

#### 2.2 错误处理
- [ ] **输入验证**
  - 文本长度限制
  - 语言支持检查
- [ ] **推理错误处理**
  - ONNX 推理失败处理
  - 模型加载失败处理

---

## 技术架构

### 数据流

```
文本输入 (TtsRequest)
    ↓
文本预处理 (Text Normalization)
    ↓
音素转换 (Phoneme → Phone IDs)
    ↓
FastSpeech2 推理 (Phone IDs → Mel-spectrogram)
    ↓
HiFiGAN 推理 (Mel-spectrogram → Audio Waveform)
    ↓
PCM 音频处理 (16-bit, 16kHz)
    ↓
Chunk 分割 (流式输出)
    ↓
TtsStreamChunk 输出
```

### 模型文件结构

```
models/tts/
├── fastspeech2-lite/
│   ├── fastspeech2_csmsc_streaming.onnx  # 中文模型
│   ├── fastspeech2_ljspeech.onnx         # 英文模型
│   ├── phone_id_map.txt                  # 音素 ID 映射
│   └── speech_stats.npy                  # 归一化统计信息
└── hifigan-lite/
    ├── hifigan_csmsc.onnx                # 中文 Vocoder
    └── hifigan_ljspeech.onnx             # 英文 Vocoder
```

### 接口定义

```rust
pub struct TtsRequest {
    pub text: String,      // 输入文本
    pub voice: String,     // 语音风格（可选，暂时不使用）
    pub locale: String,    // 语言代码（"zh", "en"）
}

pub struct TtsStreamChunk {
    pub audio: Vec<u8>,        // PCM 音频数据（16-bit, 16kHz）
    pub timestamp_ms: u64,     // 时间戳
    pub is_last: bool,         // 是否为最后一个 chunk
}

#[async_trait]
pub trait TtsStreaming: Send + Sync {
    async fn synthesize(&self, request: TtsRequest) -> EngineResult<TtsStreamChunk>;
    async fn close(&self) -> EngineResult<()>;
}
```

---

## 实现步骤

### Step 1: 创建基础结构

**文件**: `core/engine/src/tts_streaming/fastspeech2_tts.rs`

**任务**:
1. 定义 `FastSpeech2TtsEngine` 结构体
2. 实现模型加载逻辑
3. 实现基础初始化方法

**预计时间**: 2-3 小时

---

### Step 2: 实现文本预处理

**文件**: `core/engine/src/tts_streaming/text_processor.rs`

**任务**:
1. 实现文本规范化（数字、标点、缩写）
2. 实现音素转换（文本 → 音素序列）
3. 实现音素 ID 映射（使用 `phone_id_map.txt`）

**预计时间**: 4-6 小时

**依赖**:
- 可能需要外部库（如 `pinyin` for 中文）
- 或实现简单的规则引擎

---

### Step 3: 实现 FastSpeech2 推理

**文件**: `core/engine/src/tts_streaming/fastspeech2_tts.rs`

**任务**:
1. 实现输入准备（音素 ID → 张量）
2. 实现 ONNX 推理
3. 实现 mel-spectrogram 提取
4. 处理 `speech_stats.npy`（归一化/反归一化）

**预计时间**: 4-6 小时

**技术要点**:
- 使用 `ort` crate（与 NMT 一致）
- 处理动态序列长度
- mel-spectrogram 形状: `[batch, mel_dim, time_steps]`

---

### Step 4: 实现 HiFiGAN 推理

**文件**: `core/engine/src/tts_streaming/hifigan_vocoder.rs`

**任务**:
1. 实现 HiFiGAN 模型加载
2. 实现 mel-spectrogram → 音频波形转换
3. 实现 PCM 格式输出（16-bit, 16kHz）

**预计时间**: 3-4 小时

**技术要点**:
- 输入: mel-spectrogram `[batch, mel_dim, time_steps]`
- 输出: 音频波形 `[batch, audio_samples]`
- 采样率: 16kHz
- 位深: 16-bit

---

### Step 5: 实现流式输出

**文件**: `core/engine/src/tts_streaming/fastspeech2_tts.rs`

**任务**:
1. 实现完整音频生成
2. 实现 chunk 分割逻辑
3. 实现 `synthesize()` 方法
4. 实现 `close()` 方法

**预计时间**: 2-3 小时

**技术要点**:
- 每个 chunk 大小: 建议 1024-4096 样本（约 64-256ms @ 16kHz）
- `is_last` 标志：最后一个 chunk 设置为 `true`
- `timestamp_ms`：基于采样率和 chunk 索引计算

---

### Step 6: 创建 Stub 实现

**文件**: `core/engine/src/tts_streaming/stub.rs`

**任务**:
1. 实现 `TtsStub`（用于测试）
2. 返回空音频或测试音频

**预计时间**: 1 小时

---

### Step 7: 集成到 CoreEngine

**文件**: `core/engine/src/bootstrap.rs`

**任务**:
1. 在 `translate_and_publish()` 后调用 TTS
2. 发布 TTS 事件到 EventBus
3. 处理 TTS 错误

**预计时间**: 2-3 小时

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

## 测试计划

### 1. 单元测试

#### 1.1 文本预处理测试
- [ ] **测试文件**: `core/engine/tests/tts_text_processor_test.rs`
- [ ] **测试内容**:
  - 数字转文字（"123" → "一百二十三"）
  - 标点符号处理
  - 音素转换（中文、英文）
  - 音素 ID 映射

#### 1.2 FastSpeech2 推理测试
- [ ] **测试文件**: `core/engine/tests/tts_fastspeech2_test.rs`
- [ ] **测试内容**:
  - 模型加载
  - 输入准备（音素 ID → 张量）
  - 推理执行
  - mel-spectrogram 输出形状验证

#### 1.3 HiFiGAN 推理测试
- [ ] **测试文件**: `core/engine/tests/tts_hifigan_test.rs`
- [ ] **测试内容**:
  - 模型加载
  - mel-spectrogram → 音频转换
  - 输出音频格式验证（采样率、位深）

#### 1.4 完整 TTS 流程测试
- [ ] **测试文件**: `core/engine/tests/tts_integration_test.rs`
- [ ] **测试内容**:
  - 文本 → 音频完整流程
  - 中文文本合成
  - 英文文本合成
  - 音频输出验证（WAV 文件）

### 2. 集成测试

#### 2.1 端到端测试
- [ ] **测试文件**: `core/engine/tests/tts_e2e_test.rs`
- [ ] **测试内容**:
  - ASR → NMT → TTS 完整流程
  - 事件发布验证
  - 音频输出验证

#### 2.2 性能测试
- [ ] **测试内容**:
  - 推理延迟测量
  - 内存使用监控
  - 并发性能测试

### 3. 音频验证测试

#### 3.1 WAV 文件输出测试
- [ ] **测试内容**:
  - 生成 WAV 文件
  - 验证 WAV 文件格式（16-bit, 16kHz, mono）
  - 使用音频播放器验证音频质量

**测试脚本示例**:
```rust
#[test]
fn test_tts_output_wav() {
    let engine = FastSpeech2TtsEngine::new(...);
    let request = TtsRequest {
        text: "你好，世界".to_string(),
        voice: "default".to_string(),
        locale: "zh".to_string(),
    };
    let chunk = engine.synthesize(request).await?;
    
    // 保存为 WAV 文件
    let wav_path = "test_output.wav";
    save_pcm_to_wav(&chunk.audio, wav_path, 16000, 16)?;
    
    // 验证文件存在且可播放
    assert!(Path::new(wav_path).exists());
}
```

---

## 集成点

### 1. 业务流程集成

**位置**: `core/engine/src/bootstrap.rs`

**流程**:
```
VAD → ASR → Emotion → Persona → NMT → TTS → EventBus
```

**实现**:
```rust
// 在 translate_and_publish() 后
async fn synthesize_and_publish_tts(
    &self,
    translated_text: &str,
    target_language: &str,
    timestamp_ms: u64,
) -> EngineResult<()> {
    let tts_request = TtsRequest {
        text: translated_text.to_string(),
        voice: "default".to_string(),
        locale: target_language.to_string(),
    };
    
    let tts_chunk = self.tts.synthesize(tts_request).await?;
    
    // 发布 TTS 事件
    self.publish_tts_event(&tts_chunk, timestamp_ms).await?;
    
    Ok(())
}
```

### 2. 事件发布

**事件格式**:
```json
{
  "topic": "TtsChunk",
  "payload": {
    "audio": [/* PCM 音频数据 */],
    "timestamp_ms": 1234567890,
    "is_last": true
  },
  "timestamp_ms": 1234567890
}
```

---

## 已知问题和注意事项

### 1. 模型兼容性

- ⚠️ **ONNX IR 版本**: 需要验证 FastSpeech2 和 HiFiGAN 模型的 IR 版本是否兼容 `ort` 1.16.3
- ⚠️ **动态形状**: FastSpeech2 可能使用动态序列长度，需要正确处理

### 2. 文本预处理复杂性

- ⚠️ **中文文本规范化**: 需要处理数字、日期、时间等
- ⚠️ **音素转换**: 中文需要拼音/音素转换，可能需要外部库
- ⚠️ **英文音素**: 可能需要 CMUdict 或类似词典

### 3. 性能考虑

- ⚠️ **推理延迟**: TTS 推理可能较慢，需要考虑异步处理
- ⚠️ **内存使用**: mel-spectrogram 和音频波形可能占用较多内存
- ⚠️ **流式生成**: 当前实现可能是一次性生成，未来需要真正的流式生成

### 4. 音频格式

- ✅ **PCM 格式**: 16-bit, 16kHz, mono
- ⚠️ **字节序**: 需要确认是小端（little-endian）
- ⚠️ **WAV 文件**: 测试时需要将 PCM 转换为 WAV 格式

### 5. 多语言支持

- ⚠️ **模型选择**: 需要根据 `locale` 选择对应的 FastSpeech2 和 HiFiGAN 模型
- ⚠️ **语言检测**: 可能需要自动检测语言（如果 `locale` 未指定）

---

## 预计时间表

| 步骤 | 任务 | 预计时间 | 累计时间 |
|------|------|----------|----------|
| Step 1 | 创建基础结构 | 2-3 小时 | 2-3 小时 |
| Step 2 | 文本预处理 | 4-6 小时 | 6-9 小时 |
| Step 3 | FastSpeech2 推理 | 4-6 小时 | 10-15 小时 |
| Step 4 | HiFiGAN 推理 | 3-4 小时 | 13-19 小时 |
| Step 5 | 流式输出 | 2-3 小时 | 15-22 小时 |
| Step 6 | Stub 实现 | 1 小时 | 16-23 小时 |
| Step 7 | 集成到 CoreEngine | 2-3 小时 | 18-26 小时 |
| **测试** | 单元测试 + 集成测试 | 4-6 小时 | **22-32 小时** |
| **总计** | | | **3-4 天** |

---

## 下一步行动

### 立即开始

1. **验证模型兼容性**
   - 检查 FastSpeech2 和 HiFiGAN 模型的 IR 版本
   - 测试模型是否可以加载

2. **研究文本预处理方案**
   - 中文文本规范化库（如 `pinyin`）
   - 英文音素转换方案

3. **创建基础结构**
   - 创建 `FastSpeech2TtsEngine` 结构体
   - 实现模型加载逻辑

---

**最后更新**: 2024-12-19  
**状态**: 📋 计划阶段，等待开始实现

