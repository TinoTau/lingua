# Layer 4 任务检查清单

**最后更新**: 2024-12-19

---

> 用于时刻对照项目进度的任务列表。完成后在 `[ ]` 中打 `x` 标记为 `[x]`。

## 📋 阶段 1：本地环境跑通核心推理

### 1. NMT 翻译模块

- [ ] **NMT-1**: 将 `bootstrap.rs` 中的 `MarianNmtStub` 替换为 `MarianNmtOnnx`
  - 文件：`core/engine/src/bootstrap.rs`
  - 优先级：🔴 高（立即完成）
  - 预计时间：1-2 小时

- [ ] **NMT-2**: 优化 NMT KV cache（修复 workaround 模式，提升性能）
  - 文件：`core/engine/src/nmt_incremental/mod.rs`
  - 优先级：🟡 中（可后续优化）
  - 预计时间：2-3 天

### 2. ASR Whisper 原型

- [ ] **ASR-1**: 选择 Whisper 实现方案（whisper.cpp 或 FasterWhisper）
  - 优先级：🔴 高
  - 预计时间：1 天（调研）

- [ ] **ASR-2**: 实现 `AsrStreaming` trait
  - 文件：`core/engine/src/asr_streaming/mod.rs`
  - 优先级：🔴 高
  - 预计时间：2-3 天

- [ ] **ASR-3**: 实现音频预处理（PCM/浮点 → Whisper 输入格式）
  - 文件：`core/engine/src/asr_whisper/`
  - 优先级：🔴 高
  - 预计时间：1-2 天

- [ ] **ASR-4**: 实现流式推理（AsrPartial/AsrFinal 输出）
  - 文件：`core/engine/src/asr_whisper/`
  - 优先级：🔴 高
  - 预计时间：1-2 天

- [ ] **ASR-5**: 添加 ASR 测试用例（本地 CLI 或单元测试）
  - 文件：`core/engine/tests/`
  - 优先级：🟡 中
  - 预计时间：1 天

### 3. Emotion & Persona

- [ ] **EMOTION-1**: 集成 XLM-R ONNX 模型（加载和推理）
  - 文件：`core/engine/src/emotion_adapter/`
  - 优先级：🟡 中
  - 预计时间：2-3 天

- [ ] **EMOTION-2**: 实现 `EmotionAdapter` trait（情感分类逻辑）
  - 文件：`core/engine/src/emotion_adapter/mod.rs`
  - 优先级：🟡 中
  - 预计时间：1-2 天

- [ ] **EMOTION-3**: 添加 Emotion 测试用例
  - 文件：`core/engine/tests/`
  - 优先级：🟡 中
  - 预计时间：1 天

- [ ] **PERSONA-1**: 实现 `PersonaAdapter` trait（文本个性化规则/模板）
  - 文件：`core/engine/src/persona_adapter/mod.rs`
  - 优先级：🟡 中
  - 预计时间：1-2 天

- [ ] **PERSONA-2**: 添加 Persona 测试用例
  - 文件：`core/engine/tests/`
  - 优先级：🟡 中
  - 预计时间：1 天

### 4. TTS 合成

- [ ] **TTS-1**: 集成 FastSpeech2 + HiFiGAN 模型
  - 文件：`core/engine/src/tts_streaming/`
  - 优先级：🟡 中
  - 预计时间：3-5 天

- [ ] **TTS-2**: 实现 `TtsStreaming` trait（PCM 音频生成）
  - 文件：`core/engine/src/tts_streaming/mod.rs`
  - 优先级：🟡 中
  - 预计时间：2-3 天

- [ ] **TTS-3**: 实现流式音频输出（chunk 拼接）
  - 文件：`core/engine/src/tts_streaming/`
  - 优先级：🟡 中
  - 预计时间：1-2 天

- [ ] **TTS-4**: 添加 TTS 测试用例（输出 WAV 文件验证）
  - 文件：`core/engine/tests/`
  - 优先级：🟡 中
  - 预计时间：1 天

---

## 📋 阶段 2：准备 WASM 构建环境

### 5. 工具链与依赖

- [ ] **WASM-1**: 安装 wasm32 目标：`rustup target add wasm32-unknown-unknown`
  - 优先级：🔴 高
  - 预计时间：10 分钟

- [ ] **WASM-2**: 配置 wasm-bindgen/wasm-pack
  - 文件：`core/engine/Cargo.toml`、构建脚本
  - 优先级：🔴 高
  - 预计时间：1-2 天

- [ ] **WASM-3**: 处理模型文件在浏览器中的加载方式（fetch/WebFileSystem）
  - 文件：`core/engine/src/`、`core/bindings/typescript/`
  - 优先级：🔴 高
  - 预计时间：2-3 天

### 6. 序列化/事件队列

- [ ] **WASM-4**: 添加 WASM 绑定（`#[wasm_bindgen]`）到 CoreEngine
  - 文件：`core/engine/src/bootstrap.rs`
  - 优先级：🔴 高
  - 预计时间：1-2 天

- [ ] **WASM-5**: 实现 WASM 接口：`initialize()`、`push_audio_frame()`、`poll_events()`、`dispose()`
  - 文件：`core/engine/src/`
  - 优先级：🔴 高
  - 预计时间：2-3 天

- [ ] **WASM-6**: 实现事件序列化（JSON 或结构体）
  - 文件：`core/engine/src/event_bus.rs`
  - 优先级：🔴 高
  - 预计时间：1-2 天

- [ ] **WASM-7**: Node 环境测试 WASM 加载和基本功能
  - 文件：`tests/` 或 `scripts/`
  - 优先级：🔴 高
  - 预计时间：1-2 天

---

## 📋 阶段 3：导出 WASM 并接入 TypeScript

### 7. JS/TS 层更新

- [ ] **CHROME-1**: 更新 `core/bindings/typescript/runtime.ts`（WASM 加载）
  - 文件：`core/bindings/typescript/runtime.ts`
  - 优先级：🔴 高
  - 预计时间：1-2 天

- [ ] **CHROME-2**: 实现模型文件加载器（`modelLoader.ts`）
  - 文件：`core/bindings/typescript/modelLoader.ts`
  - 优先级：🔴 高
  - 预计时间：1-2 天

### 8. Chrome 插件集成

- [ ] **CHROME-3**: 替换 `clients/chrome_extension/background/main.ts` 中的 stub wasm
  - 文件：`clients/chrome_extension/background/main.ts`
  - 优先级：🔴 高
  - 预计时间：1 天

- [ ] **CHROME-4**: 实现 Popup UI 基础（显示字幕、翻译、情绪、播放状态）
  - 文件：`clients/chrome_extension/ui/`
  - 优先级：🟡 中
  - 预计时间：3-5 天

- [ ] **CHROME-5**: 实现 Options 设置页（语言、模式、Persona 配置持久化）
  - 文件：`clients/chrome_extension/options/`
  - 优先级：🟡 中
  - 预计时间：2-3 天

- [ ] **CHROME-6**: 实现 Overlay/字幕层（页面渲染字幕浮层）
  - 文件：`clients/chrome_extension/content/`
  - 优先级：🟡 中
  - 预计时间：2-3 天

- [ ] **CHROME-7**: 实现错误处理与降级（EngineError、权限拒绝、模型缺失提示）
  - 文件：`clients/chrome_extension/`
  - 优先级：🟡 中
  - 预计时间：2-3 天

- [ ] **CHROME-8**: 编写 Playwright/Puppeteer E2E 测试（完整流程验证）
  - 文件：`tests/e2e/`
  - 优先级：🟡 中
  - 预计时间：3-5 天

---

## 📊 进度统计

### 按阶段统计

- **阶段 1（本地环境）**: 0/18 任务完成
- **阶段 2（WASM 构建）**: 0/7 任务完成
- **阶段 3（Chrome 集成）**: 0/8 任务完成
- **总计**: 0/33 任务完成

### 按优先级统计

- **🔴 高优先级**: 0/15 任务完成
- **🟡 中优先级**: 0/18 任务完成

---

## 🎯 当前推荐任务顺序

### 立即执行（本周）

1. **NMT-1**: 替换 bootstrap.rs 中的 NMT Stub → 真实实现
2. **ASR-1**: 选择 Whisper 实现方案
3. **ASR-2**: 开始实现 AsrStreaming trait

### 短期目标（2-3 周）

4. **ASR-3, ASR-4**: 完成 ASR 流式推理
5. **WASM-1, WASM-2**: 搭建 WASM 构建环境
6. **EMOTION-1, EMOTION-2**: 实现 Emotion 适配器

### 中期目标（1-2 个月）

7. **WASM-3 到 WASM-7**: 完成 WASM 集成
8. **CHROME-1 到 CHROME-3**: 完成 Chrome 插件基础集成
9. **TTS-1 到 TTS-4**: 实现 TTS 模块

### 长期目标（2-3 个月）

10. **CHROME-4 到 CHROME-8**: 完成 Chrome 插件 UI 和 E2E 测试

---

## 📝 使用说明

1. 完成任务后，将对应的 `[ ]` 改为 `[x]`
2. 更新"进度统计"部分的数字
3. 在任务下方记录完成日期和备注（可选）

---

**最后更新**: 2024-12-19
**当前进度**: 0/33 任务完成 (0%)

