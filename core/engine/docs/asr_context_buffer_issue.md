# ASR 上下文缓冲区问题分析

## 问题描述

用户反馈：**需要发音非常清晰才能获得正确的识别文本**，询问上下文缓冲区对文本的检测机制是否还在工作。

## 问题分析

### 1. 上下文缓冲区机制确实存在

代码中实现了完整的上下文缓冲区机制：

- **`context_cache: Arc<Mutex<Vec<String>>>`**：存储最近 2 句的文本
- **`get_context_prompt()`**：从缓存中获取前 2 句文本，拼接成上下文提示
- **`update_context_cache()`**：更新缓存，保留最近 2 句
- **在 `infer_on_boundary()` 中**：上下文被传递给 `transcribe_audio_async()`

### 2. 但是上下文没有被实际使用！

**关键问题**：在 `engine.rs` 的 `transcribe_full_with_segments()` 方法中：

```rust
// 上下文提示（如果提供）
if let Some(prompt) = context_prompt {
    eprintln!("[ASR] 🔍 Transcription with context ({} chars): \"{}\"", ...);
    // 注意：上下文已缓存，可以在后处理阶段使用
    // 如果 whisper_rs 将来支持 initial_prompt，可以在这里启用：
    // params.set_initial_prompt(Some(prompt));  // ← 这行被注释掉了！
}
```

**问题**：
- 上下文被获取并传递到了 `transcribe_full_with_segments()`
- 但是**只是记录了日志，并没有真正传递给 Whisper 模型**
- `params.set_initial_prompt(Some(prompt))` 被注释掉了
- 代码注释说："目前 whisper_rs 可能不支持 initial_prompt"

### 3. 影响

由于上下文没有被实际传递给 Whisper 模型，导致：

1. **上下文缓冲区机制形同虚设**：虽然缓存了前 2 句文本，但对识别准确度没有帮助
2. **短句识别准确度低**：没有上下文信息，Whisper 无法利用前文来纠正当前句子的识别
3. **需要发音非常清晰**：因为没有上下文辅助，只能依赖音频本身的清晰度

## 解决方案

### 方案1：检查并启用 `whisper-rs` 的 `initial_prompt` 支持

需要检查 `whisper-rs` 0.15.1 版本是否支持 `initial_prompt`：

1. **检查 API**：查看 `FullParams` 是否有 `set_initial_prompt()` 方法
2. **如果支持**：取消注释并启用 `params.set_initial_prompt(Some(prompt))`
3. **如果不支持**：考虑升级 `whisper-rs` 版本或使用其他方法

### 方案2：使用其他方式传递上下文

如果 `whisper-rs` 不支持 `initial_prompt`，可以考虑：

1. **后处理纠错**：使用上下文信息对识别结果进行后处理纠错
2. **语言模型**：使用语言模型（如 NMT）来纠正识别错误
3. **升级库**：升级到支持 `initial_prompt` 的 `whisper-rs` 版本

### 方案3：改进音频质量

如果无法使用上下文，可以：

1. **音频预处理**：增强音频质量（降噪、增益等）
2. **调整 VAD 阈值**：确保音频片段足够长，提供更多上下文
3. **使用更大的模型**：使用 `whisper-medium` 或 `whisper-large` 提高准确度

## 建议

1. **立即检查**：检查 `whisper-rs` 0.15.1 是否支持 `initial_prompt`
2. **如果支持**：立即启用，让上下文缓冲区真正发挥作用
3. **如果不支持**：考虑升级库或实现后处理纠错机制
4. **添加日志**：在启用后添加详细日志，确认上下文是否被正确使用

## 相关文件

- `core/engine/src/asr_whisper/streaming.rs`：上下文缓冲区的实现
- `core/engine/src/asr_whisper/engine.rs`：Whisper 推理引擎（需要启用 `initial_prompt`）
- `core/engine/docs/asr_context_cache.md`：上下文缓存的设计文档

