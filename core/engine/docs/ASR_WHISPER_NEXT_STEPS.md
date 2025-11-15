# ASR Whisper 下一步行动指南

## 当前进度

### ✅ 已完成
1. **步骤 1.1**: 添加 `whisper-rs` 依赖 ✅
2. **步骤 1.2**: 准备 Whisper 模型（GGML 格式）✅
3. **步骤 2.1**: 实现音频预处理模块 ✅
   - ✅ `preprocess_audio_frame()`: 将 `AudioFrame` 转换为 Whisper 输入格式
   - ✅ `convert_to_mono()`: 多声道转单声道
   - ✅ `resample_audio()`: 重采样到 16kHz
   - ✅ `normalize_audio()`: 归一化到 [-1.0, 1.0]
   - ✅ `accumulate_audio_frames()`: 累积多个音频帧
   - ✅ 所有单元测试通过

### 📝 下一步：步骤 2.2 - 实现基础 Whisper 推理引擎

## 步骤 2.2: 实现基础 Whisper 推理引擎

### 目标
创建 `WhisperAsrEngine` 结构体，封装 Whisper 模型的加载和推理逻辑。

### 任务清单

#### 1. 创建 `WhisperAsrEngine` 结构体
**文件**: `core/engine/src/asr_whisper/engine.rs`

**结构体设计**:
```rust
pub struct WhisperAsrEngine {
    ctx: WhisperContext,
    model_path: PathBuf,
    language: Option<String>,
}
```

**需要实现的方法**:
- `new_from_model_path()`: 从模型路径加载
- `new_from_dir()`: 从模型目录加载（类似 NMT）
- `transcribe_full()`: 对完整音频进行转录
- `set_language()`: 设置语言

#### 2. 实现模型加载
- 使用 `WhisperContext::new_with_params()` 加载模型
- 处理模型路径和错误

#### 3. 实现单次推理
- 接收预处理后的音频数据（`Vec<f32>`）
- 使用 `WhisperContext::create_state()` 创建状态
- 调用 `state.full()` 进行推理
- 从 `WhisperSegment` 提取文本结果

#### 4. 处理输出格式
- 将 `WhisperSegment` 转换为 `PartialTranscript` 或 `StableTranscript`
- 处理时间戳
- 合并多个片段为完整文本

### 验收标准
- ✅ 能够加载 GGML 模型
- ✅ 能够对完整音频进行推理
- ✅ 输出正确的转录文本
- ✅ 能够处理不同语言的音频

### 参考代码
- 测试文件: `core/engine/tests/asr_whisper_simple_test.rs`
- NMT 实现: `core/engine/src/nmt_incremental/mod.rs`

---

## 步骤 2.3: 实现 `AsrStreaming` trait（基础版本）

### 目标
为 `WhisperAsrEngine` 实现 `AsrStreaming` trait，支持完整音频推理。

### 任务清单

#### 1. 实现 `AsrStreaming` trait
**文件**: `core/engine/src/asr_whisper/streaming.rs`

**需要实现的方法**:
- `initialize()`: 加载模型（已在 `new_from_model_path` 中完成）
- `infer()`: 
  - 收集 `AudioFrame` 到缓冲区
  - 当收到完整音频时，进行推理
  - 返回 `AsrResult`（包含 `PartialTranscript` 和 `StableTranscript`）
- `finalize()`: 清理资源

#### 2. 音频缓冲区管理
- 使用 `Vec<AudioFrame>` 累积音频帧
- 在 `infer()` 中累积帧
- 当检测到完整音频时（例如通过 VAD 或显式信号），进行推理

#### 3. 结果转换
- 将 Whisper 输出转换为 `AsrResult`
- `PartialTranscript`: 部分结果（如果需要）
- `StableTranscript`: 最终结果

### 验收标准
- ✅ 能够通过 `AsrStreaming` trait 调用 Whisper 推理
- ✅ 能够处理多个 `AudioFrame` 并返回转录结果
- ✅ 能够正确返回 `PartialTranscript` 和 `StableTranscript`

---

## 推荐执行顺序

### 立即开始（步骤 2.2）
1. 创建 `core/engine/src/asr_whisper/engine.rs`
2. 实现 `WhisperAsrEngine` 结构体
3. 实现模型加载方法
4. 实现 `transcribe_full()` 方法
5. 创建测试验证功能

### 然后（步骤 2.3）
1. 创建 `core/engine/src/asr_whisper/streaming.rs`
2. 为 `WhisperAsrEngine` 实现 `AsrStreaming` trait
3. 实现音频缓冲区管理
4. 测试完整流程

---

## 代码结构建议

```
core/engine/src/asr_whisper/
├── mod.rs                    # 模块导出
├── cli.rs                    # CLI 工具（已有）
├── audio_preprocessing.rs    # 音频预处理（已完成）✅
├── engine.rs                 # Whisper 推理引擎（待实现）
└── streaming.rs              # AsrStreaming trait 实现（待实现）
```

---

## 关键注意事项

1. **API 使用**: 
   - `WhisperContext::new_with_params()` 用于加载模型
   - `ctx.create_state()` 创建推理状态
   - `state.full()` 进行推理
   - `state.get_segment(i)` 获取结果片段

2. **音频格式**: 
   - 使用 `audio_preprocessing::preprocess_audio_frame()` 预处理
   - 确保输入是 16kHz 单声道 PCM f32

3. **错误处理**: 
   - 使用 `anyhow::Result` 进行错误传播
   - 转换为 `EngineResult` 用于 trait 实现

4. **测试**: 
   - 复用 `asr_whisper_simple_test.rs` 中的测试逻辑
   - 确保新实现与测试脚本兼容

---

## 预计时间

- **步骤 2.2**: 1-2 小时
- **步骤 2.3**: 1-2 小时
- **总计**: 2-4 小时

---

## 开始实现

建议从 `core/engine/src/asr_whisper/engine.rs` 开始，参考测试代码中的逻辑进行封装。

