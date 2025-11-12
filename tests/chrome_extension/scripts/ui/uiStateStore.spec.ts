/**
 * UiStateStore reducer 单元测试示例
 *
 * 用法：
 *   npm run test:ui:store
 *
 * 说明：
 *   该文件使用 Vitest/Jest 风格的断言。实际运行前请在项目中配置测试框架。
 */

import { describe, expect, it } from "vitest"; // 或改为 Jest
import { createUiStore } from "../../../clients/chrome_extension/ui";

describe("UiStateStore", () => {
  it("应当处理 BoundaryDetected 事件并置 ready=true", () => {
    const store = createUiStore();
    store.applyEvent({
      topic: "BoundaryDetected",
      timestampMs: Date.now(),
      payload: {},
    });
    expect(store.getState().ready).toBe(true);
  });

  it("应当合并 AsrPartial 并刷新 transcript.partial", () => {
    const store = createUiStore();
    store.applyEvent({
      topic: "AsrPartial",
      timestampMs: Date.now(),
      payload: { text: "hello", confidence: 0.95, isFinal: false },
    });
    expect(store.getState().transcript.partial?.text).toBe("hello");
  });

  it("应当用 TtsChunk 更新音频播放状态", () => {
    const store = createUiStore();
    store.applyEvent({
      topic: "TtsChunk",
      timestampMs: Date.now(),
      payload: { audio: new ArrayBuffer(0), timestampMs: 123, isLast: true },
    });
    const audioState = store.getState().audio;
    expect(audioState.buffering).toBe(false);
    expect(audioState.lastChunk?.isLast).toBe(true);
  });
});

