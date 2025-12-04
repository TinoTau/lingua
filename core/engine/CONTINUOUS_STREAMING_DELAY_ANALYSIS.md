# 连续输入/输出延迟问题分析

## 问题描述

1. **没有实现连续输入+输出**：系统不是真正流式的
2. **延迟累积**：第三句话输入时，第二句话才翻译出来

## 根本原因

### 1. 同步顺序处理

当前流程：
```
输入音频帧 → VAD检测 → ASR识别 → NMT翻译 → TTS合成 → 发布
```

问题：
- 每个步骤都是**同步等待**的
- 一个完整的句子必须经过整个流程才能输出
- 无法实现真正的连续处理

### 2. 增量合成的顺序执行

在 `synthesize_and_publish_incremental` 中：
```rust
for (idx, segment) in segments_with_pause.iter().enumerate() {
    let chunk = self.tts.synthesize(tts_request).await?;  // 等待完成
    // ... 发布
}
```

问题：
- 虽然 `tts_buffer_sentences == 0` 时会立即发布
- 但循环仍然是**顺序等待**每个 TTS 合成完成
- 所有 segments 处理完，函数才返回

### 3. 延迟累积示例

假设：
- ASR: 500ms
- NMT: 300ms
- TTS (3个segments): 每个 1000ms = 3000ms
- **总延迟**: ~3800ms

第二句话开始处理时，第一句话还在 TTS 阶段。

## 解决方案

### 方案 1: 异步并行处理 Segments（推荐）

让每个 segment 的 TTS 合成异步并行执行：
- 不等待所有 segments 完成
- 每个 segment 完成后立即发布
- 函数可以提前返回，让下一句话开始处理

### 方案 2: 任务队列（架构级别）

使用异步任务队列：
- 每个输入作为独立任务
- 多个任务可以并行处理
- 输出按顺序发布

### 方案 3: 优化配置

- 确保 `tts_buffer_sentences = 0`（立即发布）
- 确保 `tts_incremental_enabled = true`（增量合成）
- 减少 VAD 延迟（`min_silence_duration_ms`）

## 当前配置检查

需要检查：
- `tts_incremental_enabled` 是否启用
- `tts_buffer_sentences` 是否为 0
- `min_silence_duration_ms` 是否太小（导致误触发）

## 下一步

1. 修改 `synthesize_and_publish_incremental` 为真正的异步并行
2. 添加配置验证和优化
3. 添加性能监控日志

