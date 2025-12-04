# Segments 并行处理实现说明

## 配置检查

✅ **已验证配置**：
- `tts_incremental_enabled = true`（在 `core_engine.rs` 中设置）
- `tts_buffer_sentences = 0`（立即发布模式）

## 实现方案

### 修改内容

**文件**：`core/engine/src/bootstrap.rs`

**主要变更**：

1. **添加依赖**：
   ```rust
   use futures::future::join_all;
   ```

2. **并行处理架构**：
   - 预先准备所有 TTS 请求参数（包括异步的 voice 获取）
   - 为每个 segment 创建独立的异步任务（future）
   - 使用 `join_all` 并行执行所有任务
   - 按索引排序结果，确保播放顺序

### 关键改进

**之前（顺序处理）**：
```rust
for (idx, segment) in segments.iter().enumerate() {
    let chunk = self.tts.synthesize(tts_request).await?;  // 等待完成
    // 处理并发布
}
// 总时间 = segment1 + segment2 + segment3 + ...
```

**现在（并行处理）**：
```rust
// 创建所有任务的 future
let segment_futures = segments.iter().map(|segment| {
    async move {
        self.tts.synthesize(tts_request).await
    }
}).collect();

// 并行执行
let results = join_all(segment_futures).await;

// 按顺序发布（排序）
results.sort_by_key(|(idx, _, _, _)| *idx);
for (idx, chunk, ...) in results {
    publish(chunk);  // 按顺序发布
}
// 总时间 ≈ max(segment1, segment2, segment3, ...)
```

## 性能提升

### 预期效果

假设有 3 个 segments，每个需要 1 秒：

- **之前**：1s + 1s + 1s = **3 秒**
- **现在**：max(1s, 1s, 1s) = **1 秒**

**延迟减少**：约 66%

### 播放顺序保证

1. **并行处理**：所有 segments 同时合成
2. **结果排序**：使用 `sort_by_key(|(idx, _, _, _)| *idx)` 按索引排序
3. **顺序发布**：按排序后的顺序发布，确保播放顺序正确

## 日志输出

新的日志会显示：
```
[TTS] ⚡ Starting parallel synthesis of 3 segments...
[TTS] ⚡ Queueing segment  1 for parallel synthesis: '...'
[TTS] ⚡ Queueing segment  2 for parallel synthesis: '...'
[TTS] ⚡ Executing 3 segments in parallel...
[TTS] ✅ Segment  1 completed in 1200ms: '...' (audio_size: ... bytes)
[TTS] ✅ Segment  2 completed in 1100ms: '...' (audio_size: ... bytes)
[TTS] ✅ Segment  3 completed in 1300ms: '...' (audio_size: ... bytes)
[TTS] 📤 Published segment  1 immediately (timestamp: ...ms)
[TTS] 📤 Published segment  2 immediately (timestamp: ...ms)
[TTS] 📤 Published segment  3 immediately (timestamp: ...ms)
[TTS] ⚡ Parallel synthesis completed: 3 segments in 1350ms (avg: 450.0ms/segment)
```

## 优势

1. **大幅减少延迟**：segments 并行处理
2. **保持顺序**：结果排序后按顺序发布
3. **真正的连续处理**：下一句话可以立即开始处理
4. **向后兼容**：不影响现有功能

## 注意事项

1. **内存使用**：所有音频同时生成，可能增加内存使用
2. **服务负载**：TTS 服务需要同时处理多个请求
3. **错误处理**：如果某个 segment 失败，会返回错误（后续可以改进为容错模式）

## 测试建议

1. **测试连续输入**：快速说多句话
2. **检查播放顺序**：确保输出顺序正确
3. **性能监控**：查看日志中的并行执行时间
4. **错误处理**：测试某个 segment 失败的情况

