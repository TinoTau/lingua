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

import { EngineEventTopic, AudioFrame, EngineEvent } from "../../../clients/chrome_extension/shared/coreTypes";
import { createStubBridge } from "./utils/stubBridge";

async function main() {
  console.log("== CoreEngine 事件流测试 ==");

  const expectedTopics: EngineEventTopic[] = [
    "BoundaryDetected",
    "AsrPartial",
    "AsrFinal",
    "NmtPartial",
    "NmtFinal",
    "EmotionTag",
    "TtsChunk",
  ];

  const mockEvents: EngineEvent[] = [
    { topic: "BoundaryDetected", timestampMs: Date.now(), payload: { confidence: 0.9 } },
    {
      topic: "AsrPartial",
      timestampMs: Date.now() + 1,
      payload: { text: "hello", confidence: 0.8, isFinal: false },
    },
    {
      topic: "AsrFinal",
      timestampMs: Date.now() + 2,
      payload: { text: "hello", speakerId: "user", language: "en" },
    },
    {
      topic: "NmtPartial",
      timestampMs: Date.now() + 3,
      payload: { translatedText: "你好", isStable: false },
    },
    {
      topic: "NmtFinal",
      timestampMs: Date.now() + 4,
      payload: { translatedText: "你好", isStable: true },
    },
    {
      topic: "EmotionTag",
      timestampMs: Date.now() + 5,
      payload: { label: "positive", confidence: 0.92 },
    },
    {
      topic: "TtsChunk",
      timestampMs: Date.now() + 6,
      payload: { audio: new ArrayBuffer(0), timestampMs: 0, isLast: true },
    },
  ];

  const { router, recorded } = createStubBridge([mockEvents]);

  await router.handle({ type: "engine/boot", payload: undefined });

  for (const topic of expectedTopics) {
    await router.handle({ type: "engine/subscribe", payload: { topic } });
  }

  const mockFrame: AudioFrame = {
    sampleRate: 16000,
    channels: 1,
    data: new Float32Array([0]),
    timestampMs: 0,
  };

  await router.handle({ type: "engine/push-audio", payload: mockFrame });

  if (recorded.length !== expectedTopics.length) {
    throw new Error(`事件数量不匹配，期望 ${expectedTopics.length} 实际 ${recorded.length}`);
  }

  recorded.forEach((event, index) => {
    if (event.topic !== expectedTopics[index]) {
      throw new Error(`事件顺序错误：期望 ${expectedTopics[index]} 实际 ${event.topic}`);
    }
  });

  await router.handle({ type: "engine/shutdown", payload: undefined });
  console.log("✅ 事件流测试通过");
}

main().catch((error) => {
  console.error("❌ engine flow 测试失败:", error);
  process.exit(1);
});

