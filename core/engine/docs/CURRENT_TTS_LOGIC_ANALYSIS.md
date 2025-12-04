# 当前 TTS 语音生成逻辑分析

## 1. TTS 增量合成流程 (`synthesize_and_publish_incremental`)

### 流程概述
1. **文本分割**：将翻译后的文本分割成多个短句（基于逗号、句号等标点）
2. **并行合成**：使用 `join_all` 并行合成所有短句
3. **按序处理**：合成完成后，按索引排序，确保播放顺序
4. **立即发布**：如果 `tts_buffer_sentences == 0`，每个 segment 合成完成后立即调用 `publish_tts_event`
5. **合并返回**：最后合并所有 chunks 的音频数据，返回完整的 `TtsStreamChunk`

### 关键代码位置
- `core/engine/src/bootstrap/engine.rs:1647-2017`
- 并行合成：`join_all(segment_futures).await` (line 1916)
- 按序处理：`results_with_idx.sort_by_key(|(idx, _, _, _)| *idx)` (line 1930)
- 立即发布：`Self::publish_tts_event(self, &chunk, current_timestamp).await?` (line 1949)

### 时间戳管理
- 每个 segment 的 `timestamp_ms` 按顺序递增（每个间隔 100ms）
- 第一个 segment: `timestamp_ms`
- 第二个 segment: `timestamp_ms + 100`
- 第三个 segment: `timestamp_ms + 200`
- ...

## 2. 事件发布机制 (`publish_tts_event`)

### 事件内容
```json
{
  "audio": "<base64编码的音频数据>",
  "audio_length": 12345,
  "timestamp_ms": 1000,
  "is_last": false
}
```

### 发布流程
- 位置：`core/engine/src/bootstrap/events.rs:56-77`
- 将音频数据编码为 base64
- 通过 `event_bus.publish(event)` 发布事件

## 3. 事件总线实现 (`SimpleEventBus`)

### 当前实现
- 位置：`core/engine/src/bin/core_engine.rs:117-136`
- **问题**：`publish` 方法只是简单地返回 `Ok(())`，**没有实际存储或转发事件**
- **问题**：`subscribe` 方法只是返回一个 `EventSubscription`，**没有实际订阅机制**

```rust
async fn publish(&self, _event: CoreEvent) -> EngineResult<()> {
    Ok(())  // ❌ 事件被丢弃了！
}
```

## 4. WebSocket 处理逻辑

### 当前实现
- 位置：`core/engine/src/bin/core_engine.rs:695-716`
- **只从 `process_audio_frame` 的返回值获取结果**
- **没有监听事件总线中的 TTS 事件**

```rust
match state.engine.process_audio_frame(audio_frame, Some(src_lang.clone())).await {
    Ok(Some(result)) => {
        // 发送 ASR 转录、NMT 翻译和 TTS 音频
        let response_json = serde_json::json!({
            "transcript": ...,
            "translation": ...,
            "audio": result.tts.as_ref().and_then(|t| {
                Some(general_purpose::STANDARD.encode(&t.audio))
            }),
        });
        socket.send(Message::Text(response_json.to_string())).await;
    }
    Ok(None) => {
        // 没有最终结果，继续处理
    }
}
```

## 5. 连续模式处理 (`process_audio_frame_continuous`)

### 当前实现（修复后）
- 位置：`core/engine/src/bootstrap/engine.rs:817-977`
- **修复前**：使用 `tokio::spawn` 异步执行 `process_audio_segment`，结果被丢弃
- **修复后**：直接 `await` `process_audio_segment` 的结果并返回

### 问题
- **修复后的问题**：现在会同步等待 TTS 合成完成（2-8秒），**阻塞了音频接收**
- **这不是预期的行为**：应该异步处理，通过事件总线传递结果

## 6. 问题总结

### ✅ 已实现的功能
1. TTS 增量合成（并行合成多个短句）
2. 按顺序发布事件（通过 `timestamp_ms` 排序）
3. 事件包含完整的音频数据（base64 编码）

### ❌ 存在的问题
1. **事件总线是空的**：`SimpleEventBus.publish` 不存储事件，事件被丢弃
2. **WebSocket 不监听事件**：只从 `process_audio_frame` 返回值获取结果
3. **同步阻塞问题**：修复后，`process_audio_frame_continuous` 会同步等待 TTS 完成

## 7. 预期的正确流程

### 持续输入（异步）
1. WebSocket 接收音频帧 → `process_audio_frame_continuous`
2. VAD 检测边界 → 触发 `process_audio_segment`
3. **立即返回 `Ok(None)`**，继续接收新音频（不阻塞）

### 持续输出（异步）
1. `process_audio_segment` 异步执行（`tokio::spawn`）
2. TTS 增量合成 → 每个 segment 合成完成后立即发布事件
3. **事件总线存储事件**（使用 channel 或队列）
4. **WebSocket 监听事件总线**，按 `timestamp_ms` 顺序发送给客户端
5. 客户端按顺序播放音频

### 时序保证
- 每个 TTS segment 都有 `timestamp_ms`
- WebSocket 按 `timestamp_ms` 排序后发送
- 客户端按顺序播放

## 8. 需要修复的地方

1. **实现真正的事件总线**：
   - 使用 `tokio::sync::mpsc::channel` 存储事件
   - 支持多个订阅者（WebSocket 可以订阅）

2. **WebSocket 监听事件总线**：
   - 在 WebSocket 连接时订阅 `Tts` 事件
   - 按 `timestamp_ms` 排序后发送给客户端

3. **恢复异步处理**：
   - `process_audio_segment` 应该使用 `tokio::spawn` 异步执行
   - `process_audio_frame_continuous` 应该立即返回 `Ok(None)`

