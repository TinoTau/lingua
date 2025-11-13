# Layer 4 实现指南（逐步实现真实推理）

本文旨在帮助你按“先本地、后 WASM、再接入 Chrome 插件”的顺序实现层级 4 中的真实推理功能。每一步列出了推荐的文件位置、主要方法以及建议的测试方式。

---

## 阶段 1：本地环境跑通核心推理

### 1. Whisper ASR 原型
- **文件建议**：`core/engine/src/asr_whisper/`
  - `mod.rs`：实现 `VoiceActivityDetector`、`AsrStreaming` trait。
  - 可能需要 C/C++ Binding（whisper.cpp）：可放在 `core/engine/src/ffi/`。
- **主要方法**：
  - 音频前处理（PCM/浮点 → Whisper 输入格式）。
  - 调用 Whisper 模型得到文本；输出 `AsrPartial/AsrFinal`。
- **测试用例**：
  - 在本地写一个 CLI 或单元测试，读取 WAV 文件，检查输出文本是否与预期匹配。

### 2. NMT 翻译原型
- **文件建议**：`core/engine/src/nmt_incremental/`
  - `mod.rs`：封装 ONNX Runtime 调用。
  - 可新建 `core/engine/src/onnx_utils.rs`，统一加载/缓存 ONNX 模型。
- **主要方法**：
  - 载入 Marian 模型，使用 wait-k 策略生成增量翻译。
- **测试用例**：
  - `cargo test`：传入固定英文段落，断言翻译结果与预期中文匹配。

### 3. Emotion & Persona
- **文件建议**：`core/engine/src/emotion_adapter/`、`core/engine/src/persona_adapter/`
- **主要方法**：
  - Emotion：使用 XLM-R（ONNX）对文本进行情感分类，返回 `EmotionTag`。
  - Persona：根据用户配置（formal/ casual 等）调整翻译文本（可先实现简单模板）。
- **测试用例**：
  - 输入不同文本，检查情绪标签是否有变化； persona 输出是否带有预期语气。

### 4. TTS 合成
- **文件建议**：`core/engine/src/tts_streaming/`
- **主要方法**：
  - 结合 FastSpeech2 + HiFiGAN 模型生成 PCM 音频，按 chunk 输出。
- **测试用例**：
  - `cargo test` 或 CLI 输出 WAV 文件，人耳试听确认。

---

## 阶段 2：准备 WASM 构建环境

### 5. 工具链与依赖
- **操作**：
  - 安装 wasm32 目标：`rustup target add wasm32-unknown-unknown`
  - 若依赖 whisper.cpp 等，需要 EMSDK/clang；可在 `scripts/build` 中新增构建脚本。
  - 评估使用 `wasm-bindgen`、`wasm-pack` 或其它绑定方案。
- **注意事项**：
  - 处理模型加载（ONNX/Whisper 模型）在浏览器中的读取方式（通过 fetch 或 WebFileSystem）。

### 6. 序列化/事件队列
- **文件建议**：`core/engine/src/lib.rs`
  - 添加事件队列（Vec/CoreEvent），序列化成 JSON 或结构体。
- **主要方法**：
  - 提供 `initialize(options)`、`push_audio_frame(frame)`、`poll_events()`、`dispose()`接口（Rust `#[wasm_bindgen]`）。
- **测试**：
  - 在 Node 环境利用 `wasm-bindgen-test` 或自定义脚本加载 wasm，模拟音频输入，检查事件输出顺序。

---

## 阶段 3：导出 WASM 并接入 TypeScript

### 7. JS/TS 层更新
- **文件建议**：`core/bindings/typescript/runtime.ts`
  - 调整加载方式（`WebAssembly.instantiateStreaming` 等），处理模型文件路径。
  - 可能需新增 `core/bindings/typescript/modelLoader.ts` 用于下载/缓存 model.onnx。
- **测试用例**：
  - 在 Node 中运行 `wasmLoadTest` + 模拟事件脚本，确认真实 wasm 能产生翻译/情绪/TTS 事件。

### 8. Chrome 插件集成
- **文件建议**：
  - `clients/chrome_extension/background/main.ts`：替换 stub wasm 路径为真实 wasm。
  - `clients/chrome_extension/background/commandRouter.ts`：维持订阅逻辑。
  - 内容脚本保持不变。
- **测试**：
  - 利用手动脚本或 Playwright 驱动，喂入音频，观察事件流是否包含真实数据（非 stub）。

---

## 补充说明
- 建议先实现 ASR → NMT → TTS 这条主链，Emotion/Persona 可在主链稳定后跟进。
- 每个模块完成后都写单独的测试（在 `core/engine/tests/` 或 `tests/chrome_extension/scripts/` 中补充）。
- 构建脚本与依赖安装最好记录到 `README` 或 `scripts/build/` 中，便于团队共享。

完成以上步骤后，即可用真实推理替换 stub wasm，继续进行层级 4 的其它任务（UI、E2E 等）。***

