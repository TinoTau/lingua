import { performance } from "perf_hooks";
import { EngineEvent } from "../../../clients/chrome_extension/shared/coreTypes";
import { createStubBridge } from "./utils/stubBridge";

function percentile(samples: number[], p: number): number {
  if (samples.length === 0) {
    return 0;
  }
  const sorted = [...samples].sort((a, b) => a - b);
  const index = Math.ceil((p / 100) * sorted.length) - 1;
  return sorted[Math.max(0, Math.min(sorted.length - 1, index))];
}

async function main() {
  console.log("== 核心引擎性能探针 ==");

  const iterations = Number(process.argv[2] ?? 5);
  const durations: number[] = [];

  for (let i = 0; i < iterations; i += 1) {
    const start = performance.now();
    const events: EngineEvent[][] = [
      [
        { topic: "BoundaryDetected", timestampMs: Date.now(), payload: {} },
        { topic: "AsrFinal", timestampMs: Date.now(), payload: { text: "hello", language: "en" } },
        { topic: "NmtFinal", timestampMs: Date.now(), payload: { translatedText: "你好", isStable: true } },
        { topic: "TtsChunk", timestampMs: Date.now(), payload: { audio: new ArrayBuffer(2), timestampMs: 0, isLast: true } },
      ],
    ];

    const { router } = createStubBridge(events);
    await router.handle({ type: "engine/boot", payload: undefined });
    await router.handle({ type: "engine/subscribe", payload: { topic: "BoundaryDetected" } });
    await router.handle({ type: "engine/subscribe", payload: { topic: "AsrFinal" } });
    await router.handle({ type: "engine/subscribe", payload: { topic: "NmtFinal" } });
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

    durations.push(performance.now() - start);
  }

  console.log(`运行 ${iterations} 次，耗时 (ms)：`, durations.map((d) => d.toFixed(2)).join(", "));
  console.log(`P50=${percentile(durations, 50).toFixed(2)}ms  P95=${percentile(durations, 95).toFixed(2)}ms`);
}

main().catch((error) => {
  console.error("❌ 性能探针执行失败:", error);
  process.exit(1);
});

