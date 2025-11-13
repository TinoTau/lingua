import { EngineEvent } from "../../../clients/chrome_extension/shared/coreTypes";
import { createStubBridge } from "./utils/stubBridge";

async function main() {
  console.log("== Persona & Emotion 事件测试 ==");

  const events: EngineEvent[][] = [
    [
      {
        topic: "NmtFinal",
        timestampMs: Date.now(),
        payload: { translatedText: "您好，李先生。", isStable: true, persona: "formal" },
      },
      {
        topic: "EmotionTag",
        timestampMs: Date.now() + 1,
        payload: { label: "calm", confidence: 0.85 },
      },
    ],
  ];

  const { router, recorded } = createStubBridge(events);

  await router.handle({ type: "engine/boot", payload: undefined });
  await router.handle({ type: "engine/subscribe", payload: { topic: "NmtFinal" } });
  await router.handle({ type: "engine/subscribe", payload: { topic: "EmotionTag" } });

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

  const personaEvent = recorded.find((event) => event.topic === "NmtFinal");
  const emotionEvent = recorded.find((event) => event.topic === "EmotionTag");

  if (!personaEvent) {
    throw new Error("未捕获 NmtFinal 事件");
  }
  if (!emotionEvent) {
    throw new Error("未捕获 EmotionTag 事件");
  }

  const personaPayload = personaEvent.payload as { translatedText: string; persona?: string };
  if (personaPayload.persona !== "formal") {
    throw new Error(`Persona 标记错误，期望 formal 实际 ${personaPayload.persona}`);
  }

  const emotionPayload = emotionEvent.payload as { label: string; confidence: number };
  if (emotionPayload.label !== "calm") {
    throw new Error(`情绪标签不匹配，期望 calm 实际 ${emotionPayload.label}`);
  }

  console.log("✅ Persona & Emotion 测试通过");
}

main().catch((error) => {
  console.error("❌ Persona & Emotion 测试失败:", error);
  process.exit(1);
});

