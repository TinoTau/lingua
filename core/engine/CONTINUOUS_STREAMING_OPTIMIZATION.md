# 连续输入/输出延迟优化方案

## 问题总结

1. **延迟累积**：第三句话输入时，第二句话才翻译出来
2. **没有真正的连续处理**：TTS segments 顺序处理，导致延迟累积

## 当前架构

### 异步音频段处理（✅ 已实现）
- `process_audio_frame_continuous` 使用 `tokio::spawn` 异步处理每个音频段
- 可以同时处理多个音频段（第一句话还在处理时，第二句话可以开始）

### TTS Segments 顺序处理（❌ 问题所在）
- `synthesize_and_publish_incremental` 中，segments 在循环中**顺序等待**
- 即使 `tts_buffer_sentences == 0` 立即发布，仍需要等待前一个 segment 完成

## 优化方案

### 方案 1: 并行处理 Segments（推荐）

修改 `synthesize_and_publish_incremental`，使用 `futures::future::join_all` 并行处理所有 segments，然后按顺序发布：

```rust
// 创建所有 TTS 任务的 future
let segment_futures: Vec<_> = segments_with_pause.iter().enumerate().map(|(idx, segment)| {
    // 为每个 segment 创建异步任务
    // ... TTS 合成逻辑
}).collect();

// 并行执行所有任务
let results = futures::future::join_all(segment_futures).await;

// 按顺序发布结果
for (idx, chunk) in results.into_iter().enumerate() {
    self.publish_tts_event(&chunk, timestamp).await?;
}
```

**优点**：
- 所有 segments 并行处理
- 保持发布顺序
- 大幅减少延迟

**缺点**：
- 需要重构代码
- 可能增加内存使用（所有音频同时生成）

### 方案 2: 流水线处理

让每个 segment 处理完成后立即发布，不等待其他 segments：

```rust
// 为每个 segment spawn 独立任务
for (idx, segment) in segments_with_pause.iter().enumerate() {
    let engine_clone = self.clone();
    let segment_clone = segment.clone();
    tokio::spawn(async move {
        let chunk = engine_clone.synthesize_segment(segment_clone).await;
        engine_clone.publish_tts_event(&chunk, timestamp).await;
    });
}
```

**优点**：
- 最简单的实现
- 真正的流式处理

**缺点**：
- 发布顺序可能混乱（需要序号管理）
- 需要额外的同步机制

### 方案 3: 配置优化（短期）

确保配置正确：

```toml
# 在配置中确保
tts_incremental_enabled = true
tts_buffer_sentences = 0  # 立即发布
```

同时优化 VAD 配置以减少延迟：
```toml
[vad]
min_silence_duration_ms = 200  # 减小以更快触发
```

## 推荐实施

### 短期（立即）
1. ✅ 检查并确保配置正确
2. ✅ 优化 VAD 参数
3. ✅ 添加性能监控日志

### 中期（1-2周）
1. 实现方案 1：并行处理 segments
2. 添加配置选项控制并行度
3. 性能测试和优化

### 长期（架构级）
1. 实现任务队列系统
2. 支持优先级队列
3. 添加背压控制

## 配置检查清单

- [ ] `tts_incremental_enabled = true`
- [ ] `tts_buffer_sentences = 0`（立即发布）
- [ ] `min_silence_duration_ms = 200`（快速响应）
- [ ] `continuous_mode = true`

## 性能目标

- **目标延迟**：每个 segment 处理时间 < 500ms
- **总延迟**：第一句话完整输出 < 2秒
- **并发能力**：同时处理 3-5 个音频段

