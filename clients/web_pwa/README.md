# Lingua Web PWA - 极简网页应用

这是一个极简的网页应用，用于验证 Lingua CoreEngine 的 S2S 翻译功能。

## 功能

- ✅ 麦克风音频采集
- ✅ 调用 CoreEngine S2S 接口
- ✅ 显示转录和翻译文本
- ✅ 播放翻译后的语音
- ✅ 简洁的用户界面

## 使用方法

### 1. 启动 CoreEngine 服务

确保 CoreEngine 服务正在运行：

```bash
# 启动所有服务
.\start_lingua_core.ps1  # Windows
# 或
bash start_lingua_core.sh  # Linux/macOS
```

### 2. 打开网页

直接在浏览器中打开 `index.html` 文件，或使用本地服务器：

```bash
# 使用 Python 简单服务器
cd clients/web_pwa
python -m http.server 8080

# 或使用 Node.js
npx http-server -p 8080
```

### 3. 访问应用

在浏览器中打开：`http://localhost:8080`

### 4. 使用步骤

1. 配置服务地址（默认：`http://127.0.0.1:9000`）
2. 选择源语言和目标语言
3. 点击"开始录音"按钮
4. 对着麦克风说话
5. 点击"停止录音"按钮
6. 等待处理完成，查看转录和翻译结果
7. 自动播放翻译后的语音

## 浏览器要求

- Chrome 60+
- Firefox 55+
- Edge 79+
- Safari 11+（部分功能可能受限）

## 注意事项

1. **HTTPS 要求**：某些浏览器要求 HTTPS 才能访问麦克风。如果遇到权限问题，请使用 HTTPS 或 localhost。

2. **CORS 问题**：如果 CoreEngine 服务在不同端口，可能需要配置 CORS。当前代码假设服务在同一域名下。

3. **音频格式**：当前使用 WebM 格式录制，然后转换为 WAV。某些浏览器可能不支持，会自动降级到默认格式。

## 故障排除

### 无法访问麦克风

- 检查浏览器权限设置
- 确保使用 HTTPS 或 localhost
- 检查系统麦克风权限

### 调用服务失败

- 检查 CoreEngine 服务是否运行
- 检查服务地址是否正确
- 检查浏览器控制台的错误信息

### 音频播放失败

- 检查浏览器是否支持 WAV 格式
- 检查返回的音频数据是否有效

## 后续改进

- [ ] 支持实时流式翻译（WebSocket）
- [ ] 支持音频可视化
- [ ] 支持历史记录
- [ ] 支持多语言界面
- [ ] 支持音频下载

