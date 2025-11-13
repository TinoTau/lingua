# Layer 4 任务拆解 (Chrome 插件 UI & 真实推理)

1. CoreEngine WASM 构建与加载  
   - 目标：将 `core/engine` 编译为 `engine.wasm`，背景页可加载运行。  
   - 关键点：Cargo 配置、wasm-bindgen/wasm-pack 输出、模型路径解析。  
   - 测试：Node 脚本调用 WASM 的 `boot()/shutdown()`。

2. ASR 推理接口（Whisper）  
   - 目标：在 WASM 中实现 `VoiceActivityDetector` + `AsrStreaming`。  
   - 关键点：集成 Whisper.cpp/FasterWhisper、音频预处理。  
   - 测试：注入 `AudioFrame`，检查 `AsrPartial/Final`。

3. NMT 翻译模块  
   - 目标：接入 Marian ONNX，完成 `NmtIncremental`。  
   - 关键点：加载语言对模型、wait-k 策略。  
   - 测试：给定英文转写，得到中文译文事件。

4. Persona & Emotion 真实实现  
   - 目标：输出真实情绪标签、文本个性化结果。  
   - 关键点：XLM-R 推理、语气规则。  
   - 测试：模拟不同文本，检查 `EmotionTag`/`persona`。

5. TTS 合成与音频缓冲  
   - 目标：集成 FastSpeech2 + HiFiGAN，输出 PCM & 播放。  
   - 关键点：WebAudio 播放、chunk 拼接。  
   - 测试：调用 TTS 接口，听取生成语音。

6. Popup UI 基础  
   - 目标：搭建 Popup 界面，显示字幕、翻译、情绪、播放状态。  
   - 关键点：`UiStateStore`、事件订阅、React/Vue 架构。  
   - 测试：用 stub 事件驱动 UI。

7. Options 设置页  
   - 目标：管理语言、模式、Persona 配置并持久化。  
   - 关键点：Chrome storage、与后台同步。  
   - 测试：修改设置后刷新，确认生效。

8. Overlay/字幕层  
   - 目标：在页面渲染字幕浮层。  
   - 关键点：CSS 注入、开关控制。  
   - 测试：启用浮层，观察字幕滚动。

9. 错误处理与降级  
   - 目标：捕获 `EngineError`、权限拒绝、模型缺失等，提供提示。  
   - 关键点：后台监听错误、UI toast、重试机制。  
   - 测试：刻意制造错误，验证提示与恢复。

10. Playwright/Puppeteer E2E  
    - 目标：自动化验证打开网页→翻译→TTS 的全流程。  
    - 关键点：加载扩展、注入音频、断言 UI/语音。  
    - 测试：编写脚本模拟真实操作。

