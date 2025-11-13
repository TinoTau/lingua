# 层级 3 手动测试指南

## 1. 准备工作
- 确保运行过层级 1、层级 2 的自检脚本，模型与桥接逻辑已正常。
- 执行 `npm install`（如仓库无 node_modules），构建扩展：
  ```bash
  npm run build:extension   # 如未提供，可使用 webpack/vite 脚本
  ```
- 准备一段测试音频（例如 `tests/chrome_extension/manual/audio/sample.wav`），包含明确的语音内容以便观察转写效果。

## 2. 安装扩展
1. 打开 Chrome → `chrome://extensions`
2. 开启“开发者模式”
3. 点击“加载已解压的扩展”，选择构建后的输出目录（如 `dist/chrome_extension`）
4. 确认扩展图标出现并后台 service worker 启动无报错（可在扩展详情页查看 service worker 控制台）

## 3. 手动测试场景

### 场景 A：音频采集与基本翻译
1. 在任意网页执行以下脚本，推送测试音频帧：
   ```js
   (async () => {
     const response = await fetch(chrome.runtime.getURL("tests/chrome_extension/manual/audio/sample.wav"));
     const buffer = await response.arrayBuffer();
     chrome.runtime.sendMessage({
       type: "engine/boot",
       payload: undefined,
     });
     chrome.runtime.sendMessage({
       type: "engine/subscribe",
       payload: { topic: "AsrFinal" },
     });
     chrome.runtime.sendMessage({
       type: "engine/subscribe",
       payload: { topic: "NmtFinal" },
     });
     chrome.runtime.sendMessage({
       type: "engine/push-audio",
       payload: {
         sampleRate: 16000,
         channels: 1,
         data: Array.from(new Float32Array(buffer)),
         timestampMs: 0,
       },
     });
   })();
   ```
2. 打开背景页 console，确认出现 `AsrFinal` 和 `NmtFinal` 日志。
3. 在 Popup 中查看字幕/翻译是否更新。

### 场景 B：真实麦克风采集
1. 打开任意网页，按 F12 → console，执行：
   ```js
   chrome.runtime.sendMessage({ type: "capture/start" });
   ```
2. 授权麦克风后，对着麦克风说一句英语（例如“Hello everyone”），观察后台 console 是否输出 `AsrPartial` / `AsrFinal`。
3. 停止采集：
   ```js
   chrome.runtime.sendMessage({ type: "capture/stop" });
   ```
4. 确认麦克风设备已释放。

### 场景 C：Persona / 情绪切换
1. 在 Options 页面调整 Persona 设定（如正式/口语），保存设置。
2. 重新进行场景 B。
3. 在后台日志中确认 `NmtFinal` 事件 payload 中携带的 persona 信息和 `EmotionTag` 标签符合预期。

### 场景 D：错误处理
1. 手动删除某个模型文件（例如 `core/engine/models/nmt/marian-en-zh/model.onnx`）
2. 再次执行场景 B。
3. 观察后台是否报错并发送 `EngineError`；在 UI 中是否出现友好提示。
4. 恢复模型后重复测试，确保错误消失。

## 4. 注意事项
- 若使用 Playwright 或 Puppeteer 做自动化，可在 `tests/chrome_extension/scripts/e2e/` 基础上填充具体实现（加载页面、喂 mock 音频、断言 UI）。
- 当前桥接依赖本地 `engine.wasm`，确保构建输出包含该文件并在 `manifest.json` 中声明 web_accessible_resources。
- 执行完测试后，记得在背景 console 中调用 `chrome.runtime.sendMessage({ type: "engine/shutdown" })` 或通过扩展菜单停用插件，以释放资源。

