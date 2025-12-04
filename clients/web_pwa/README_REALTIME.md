# Lingua 实时流式翻译 Web 前端

## 功能说明

这是一个支持实时语音输入的 Web 前端，通过 WebSocket 连接到 CoreEngine，实现：
- **实时音频流传输**：使用 Web Audio API 实时捕获麦克风音频
- **实时 ASR 识别**：音频帧实时发送到服务器进行语音识别
- **实时翻译**：识别结果实时翻译
- **实时 TTS 播放**：翻译结果实时合成语音并播放

## 使用方法

### 1. 启动 CoreEngine 服务

确保 CoreEngine 服务正在运行（默认端口 9000）：

```bash
# 在 core/engine 目录下
cargo run --bin core_engine -- --config lingua_core_config.toml
```

### 2. 启动 Web 前端服务器

```bash
# 在 clients/web_pwa 目录下
.\start_web_server.ps1
```

或者使用 Python：

```bash
python -m http.server 8080 --directory clients/web_pwa
```

### 3. 访问实时流式页面

在浏览器中打开：
- **实时流式模式**：`http://localhost:8080/index_realtime.html`
- **整句翻译模式**：`http://localhost:8080/index.html`（原有功能）

### 4. 使用步骤

1. **配置服务地址**：确保服务地址指向 CoreEngine（默认 `http://127.0.0.1:9000`）
2. **选择语言**：选择源语言和目标语言
3. **点击"开始实时录音"**：浏览器会请求麦克风权限
4. **开始说话**：系统会实时识别和翻译您的语音
5. **查看结果**：转录文本和翻译文本会实时更新
6. **点击"停止录音"**：停止录音并断开连接

## 技术实现

### WebSocket 协议

#### 客户端 → 服务器消息

**音频帧消息**：
```json
{
  "type": "audio_frame",
  "data": "base64编码的PCM音频数据",
  "timestamp_ms": 12345,
  "sample_rate": 16000,
  "channels": 1
}
```

**配置消息**：
```json
{
  "type": "config",
  "src_lang": "zh",
  "tgt_lang": "en"
}
```

#### 服务器 → 客户端消息

**结果消息**：
```json
{
  "transcript": "识别的文本",
  "translation": "翻译的文本",
  "audio": "base64编码的TTS音频数据（可选）"
}
```

### 音频处理流程

1. **音频捕获**：使用 `getUserMedia` 获取麦克风音频流
2. **音频处理**：使用 `ScriptProcessorNode` 实时处理音频数据
3. **格式转换**：将 Float32 音频转换为 16-bit PCM
4. **Base64 编码**：将 PCM 数据编码为 Base64
5. **WebSocket 传输**：通过 WebSocket 发送到服务器
6. **实时处理**：服务器实时处理音频帧，返回识别和翻译结果
7. **结果展示**：实时更新页面上的转录和翻译文本
8. **音频播放**：播放服务器返回的 TTS 音频

## 注意事项

1. **浏览器兼容性**：建议使用 Chrome、Firefox 或 Edge 浏览器
2. **麦克风权限**：首次使用需要授予麦克风权限
3. **网络延迟**：实时处理会有一定的网络延迟
4. **音频质量**：建议在安静的环境中使用，以获得更好的识别效果
5. **服务状态**：确保 CoreEngine 服务正常运行，否则 WebSocket 连接会失败

## 故障排除

### WebSocket 连接失败

- 检查 CoreEngine 服务是否运行
- 检查服务地址是否正确
- 检查防火墙设置

### 没有识别结果

- 检查麦克风权限是否已授予
- 检查音频输入是否正常（查看浏览器控制台）
- 检查服务器日志，查看是否有错误信息

### 音频播放失败

- 检查浏览器是否支持音频播放
- 检查返回的音频数据是否有效

## 文件说明

- `index_realtime.html` - 实时流式模式的 HTML 页面
- `app_realtime.js` - 实时流式模式的 JavaScript 代码
- `index.html` - 整句翻译模式的 HTML 页面（原有）
- `app.js` - 整句翻译模式的 JavaScript 代码（原有）

