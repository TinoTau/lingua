import { EngineEvent } from "../../../clients/chrome_extension/shared/coreTypes";
import { createStubBridge } from "./utils/stubBridge";

async function main() {
  console.log("== TTS 输出烟雾测试 ==");

  const events: EngineEvent[][] = [
    [
      {
        topic: "TtsChunk",
        timestampMs: Date.now(),
        payload: { audio: new ArrayBuffer(8), timestampMs: 0, isLast: false },
      },
      {
        topic: "TtsChunk",
        timestampMs: Date.now() + 1,
        payload: { audio: new ArrayBuffer(4), timestampMs: 40, isLast: true },
      },
    ],
  ];

  const { router, recorded } = createStubBridge(events);

  await router.handle({ type: "engine/boot", payload: undefined });
  await router.handle({ type: "engine/subscribe", payload: { topic: "TtsChunk" } });

  await router.handle({
    type: "engine/push-audio",
    payload: {
      sampleRate: 16000,
      channels: 1,
      data: new Float32Array([0]),
      timestampMs: 0,
    },
  });

  await router.handle({ type: "engine/shutdown", payload: undefined });

  if (recorded.length !== 2) {
    throw new Error(`TTS 事件数量异常，期望 2 实际 ${recorded.length}`);
  }
  const lastChunk = recorded[1].payload as { isLast: boolean };
  if (!lastChunk.isLast) {
    throw new Error("未检测到最后一个 TTS Chunk");
  }

  console.log("✅ TTS 烟雾测试通过");
}

main().catch((error) => {
  console.error("❌ TTS 烟雾测试失败:", error);
  process.exit(1);
});

