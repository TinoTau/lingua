/**
 * 核心引擎事件流测试
 *
 * 用法：
 *   npm run test:engine:flow
 *
 * 说明：
 *   - 假设已构建出 core/engine 的 WASM 模块，并能通过 CoreEngineBridge 加载。
 *   - 该脚本仅提供结构示例，具体实现需根据实际 WASM 接口完成 TODO 部分。
 */

import { resolve } from "path";

interface EngineEvent {
  topic: string;
  payload: unknown;
}

async function loadEngine() {
  // TODO: 替换为真实的 WASM 加载逻辑 (例如 core/bindings/typescript runtime)
  throw new Error("TODO: 实现 loadEngine()");
}

async function pushMockAudio(engine: { submitAudioFrame(frame: unknown): Promise<void> }) {
  // TODO: 读取测试音频（wav）并转换为 AudioFrame，与项目 codec 对齐
  console.log("TODO: push mock audio frames");
  await engine.submitAudioFrame({});
}

async function main() {
  console.log("== CoreEngine 事件流测试 ==");

  const engine = await loadEngine();
  const events: EngineEvent[] = [];

  // TODO: 订阅事件 (engine.subscribe / EventBus)
  console.log("TODO: 订阅 CoreEngine 事件");

  await pushMockAudio(engine);

  // TODO: 等待事件到齐（可设定超时）

  console.log("TODO: 校验事件顺序与 payload");
  console.log("应包含 BoundaryDetected / AsrPartial / AsrFinal / NmtPartial / NmtFinal / EmotionTag / TtsChunk");

  await engine.shutdown?.();
  console.log("== 测试完成 ==");
}

main().catch((error) => {
  console.error("❌ engine flow 测试失败:", error);
  process.exit(1);
});

