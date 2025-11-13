# 层级 1：模型与环境自检

## 测试目标
- 确认 `core/engine/models` 下的 ASR / NMT / Emotion / Persona / TTS / VAD 模型与配置文件齐全。
- 校验关键模型可被 ONNX Runtime 成功加载。
- 验证 Chrome 插件执行环境所需的依赖、权限、配置是否就绪。

## 功能点与测试内容

| 功能 | 测试要点 | 操作 / 脚本 |
| --- | --- | --- |
| 模型存在性 | Whisper、Marian、XLM-R、FastSpeech2、HiFiGAN、Silero 模型及 tokenizer/config 是否完整 | `npm run test:models:check` （`scripts/checkModels.ts`） |
| ONNX 加载 | 使用 `onnxruntime-node` 尝试加载每个模型，验证输入/输出维度 | 同上脚本输出需看到 “✅ loaded” |
| WASM 占位构建 | 生成并验证 stub `engine.wasm`，确保可被 WebAssembly 实例化 | `npm run build:engine:stub` 以及 `npm run test:wasm` |
| 配置一致性 | `configs/env/templates/chrome.dev.json` 中的语言、模型路径、feature flags 是否与模型目录匹配 | `npx ts-node --project tsconfig.test.json tests/chrome_extension/scripts/verifyConfig.ts` |
| 依赖检查 | Node.js、npm/pnpm、Rust、wasm32 目标、Chrome 权限是否可用 | `npm run test:env` 调用 `scripts/environmentCheck.ts` |

## 运行脚本

1. 安装依赖：
   ```bash
   npm install
   npm install onnxruntime-node
   ```
2. 执行模型检查：
   ```bash
   npm run test:models:check
   ```
3. 执行环境自检：
   ```bash
   npm run test:env
   ```

## 输出记录要求
- 保留脚本输出日志（建议存入 `tests/chrome_extension/results/model-validation-YYYYMMDD.log`）。
- 如发现缺失模型或加载失败，记录根因与修复方案。
- 自检通过后，在本文件末尾添加日期与结论。

## 最新测试记录
- `TODO`：待首次测试完成后填写。

