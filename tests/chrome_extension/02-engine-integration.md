# 层级 2：核心引擎集成测试

## 测试目标
- 在 Node.js 环境中直接调用 WASM 化的 `CoreEngine`，验证各模块事件顺序与数据正确性。
- 检查多语言翻译、情绪检测、Persona、TTS 流程是否完整。
- 评估性能指标（延迟、吞吐、资源占用），确保满足文档中 ≤1.5 s 的端到端目标。

## 功能点与测试内容

| 功能 | 测试要点 | 脚本 / 命令 |
| --- | --- | --- |
| 事件流完整性 | `BoundaryDetected` → `AsrPartial/Final` → `NmtPartial/Final` → `EmotionTag` → `TtsChunk` 顺序是否正确；payload 是否含置信度、语言标签等关键信息 | `npm run test:engine:flow` 调用 `scripts/engineFlowTest.ts` |
| 多语言覆盖 | 测试英→中、英→日、英→西，确认切换配置时能加载对应 Marian 模型 | `scripts/engineFlowTest.ts --languages en-zh,en-ja,en-es` |
| Persona / 情绪 | 对同一句子，验证 Persona 不同语气输出差异、`EmotionTag` 是否识别情绪（如正面/负面） | `scripts/enginePersonaEmotionTest.ts` |
| TTS Pipeline | `TtsChunk` 是否按 streaming 顺序输出；结果音频可播放且长度合理 | `scripts/engineTtsSmokeTest.ts` 输出 WAV 文件供人工聆听 |
| 性能与资源 | 记录各阶段耗时、CPU、内存占用；检测背压策略 | `scripts/enginePerfProbe.ts`（需 Node 18+，调用 `perf_hooks`） |

## 准备工作

1. 构建 WASM 引擎：
   ```bash
   cd core/engine
   rustup target add wasm32-unknown-unknown
   cargo build --target wasm32-unknown-unknown --release
   ```
   将生成的 `engine.wasm` 拷贝到 `clients/chrome_extension/background/` 或脚本所需位置。

2. 安装测试依赖：
   ```bash
   npm install
   npm install onnxruntime-node wav-encoder
   ```

## 执行示例

```bash
npm run test:engine:flow
npm run test:engine:perf -- --iterations 5 --languages en-zh
```

## 输出记录要求
- 每次测试将事件日志、性能数据存入 `tests/chrome_extension/results/engine-integration-YYYYMMDD.log`。
- 对于异常（事件缺失、模型加载失败、耗时过长）记录详细场景和可能原因。

## 最新测试记录
- `TODO`：待首次执行后补充结论。*** End Patch

