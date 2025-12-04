# Web 端到 CoreEngine 流程检查清单

## 1. WebSocket 路由配置

### Web 端
- **文件**: `clients/web_pwa/app_realtime.js`
- **连接 URL**: `ws://127.0.0.1:9000/stream` (line 122)
- **连接方式**: `new WebSocket(wsUrl)` (line 125)

### CoreEngine 端
- **文件**: `core/engine/src/bin/core_engine.rs`
- **路由**: `.route("/stream", get(stream_handler))` (line 262)
- **处理器**: `stream_handler` (line 610)
- **WebSocket 升级**: `WebSocketUpgrade` (line 611)

✅ **状态**: 路由配置正确

## 2. 消息格式对比

### Web 端发送的消息格式
```javascript
{
    type: 'audio_frame',
    data: base64Audio,           // base64 编码的 PCM 数据
    timestamp_ms: Date.now() - (this.recordStartTime || Date.now()),
    sample_rate: 16000,
    channels: 1
}
```

### CoreEngine 期望的消息格式
```rust
{
    "type": "audio_frame",
    "data": base64_audio,        // base64 编码的字符串
    "timestamp_ms": u64,
    "sample_rate": u64,
    "channels": u64
}
```

✅ **状态**: 消息格式匹配

## 3. 配置消息格式

### Web 端发送的配置
```javascript
{
    type: 'config',
    src_lang: document.getElementById('srcLang').value,
    tgt_lang: document.getElementById('tgtLang').value
}
```

### CoreEngine 接收的配置
```rust
if json_msg["type"] == "config" {
    if let Some(lang) = json_msg["src_lang"].as_str() {
        src_lang = lang.to_string();
    }
    if let Some(lang) = json_msg["tgt_lang"].as_str() {
        tgt_lang = lang.to_string();
    }
}
```

✅ **状态**: 配置格式匹配

## 4. 音频数据处理流程

### Web 端
1. 使用 `ScriptProcessorNode` 捕获音频 (line 60-96)
2. 转换为 16-bit PCM (line 77-78)
3. Base64 编码 (line 80)
4. 通过 WebSocket 发送 (line 92)

### CoreEngine 端
1. 接收 JSON 消息 (line 633)
2. 解析 `audio_frame` 类型 (line 650)
3. Base64 解码 (line 661)
4. 转换为 f32 数组 (line 670-674)
5. 创建 `AudioFrame` (line 680-685)
6. 调用 `process_audio_frame` (line 688)

✅ **状态**: 数据处理流程正确

## 5. 潜在问题检查

### 问题 1: WebSocket 连接可能失败
- **检查点**: 查看日志 `[WebSocket] ✅ Client connected`
- **如果未出现**: Web 端连接失败，检查端口和 URL

### 问题 2: 音频帧未接收
- **检查点**: 查看日志 `[WebSocket] 📥 Received audio frame #50: ...`
- **如果未出现**: 
  - Web 端可能未发送音频帧
  - 消息格式可能不匹配
  - WebSocket 连接可能已断开

### 问题 3: Base64 解码失败
- **检查点**: 查看日志 `[WebSocket] ❌ Failed to decode base64 audio`
- **如果出现**: Web 端的 base64 编码可能有问题

### 问题 4: 音频数据格式问题
- **检查点**: 查看日志中的 `max` 和 `rms` 值
- **如果 max < 0.001**: 音频可能太安静或格式错误
- **如果 rms = 0**: 音频数据可能为空

## 6. 调试建议

1. **启用详细日志**: 已添加日志输出
2. **检查 Web 端控制台**: 查看是否有 JavaScript 错误
3. **检查网络连接**: 确认 WebSocket 连接状态
4. **验证音频捕获**: 确认麦克风权限和音频捕获是否正常

## 7. 已知问题

### 缓冲区溢出问题
- **现象**: `[VAD] Buffer overflow detected, forcing boundary`
- **原因**: VAD 长时间未检测到边界，导致缓冲区累积超过 5000ms
- **修复**: 已添加强制边界处理逻辑

### 音频帧丢失
- **可能原因**: 
  - WebSocket 连接不稳定
  - 处理速度跟不上接收速度
  - 缓冲区溢出导致帧丢失

## 8. 下一步调试步骤

1. 运行服务并查看日志
2. 检查是否出现 `[WebSocket] ✅ Client connected`
3. 检查是否出现 `[WebSocket] 📥 Received audio frame`
4. 如果未出现音频帧日志，检查 Web 端是否正常发送
5. 如果出现音频帧但仍有问题，检查音频数据质量（max, rms 值）

