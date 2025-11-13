import { AudioFrame, EngineEvent } from "../shared/coreTypes";
import { createMediaRecorderFactory } from "./mediaRecorderCapture";

interface ContentCommand {
  type: "capture/start" | "capture/stop";
  payload?: unknown;
}

type BackgroundPort = chrome.runtime.Port;

class ContentBridge {
  private port?: BackgroundPort;

  connect(): void {
    if (this.port) {
      return;
    }
    this.port = chrome.runtime.connect({ name: "content" });
    this.port.onMessage.addListener((message: { event: EngineEvent }) => {
      window.dispatchEvent(new CustomEvent("lingua-engine-event", { detail: message.event }));
    });
  }

  sendAudio(frame: AudioFrame) {
    chrome.runtime.sendMessage({
      type: "engine/push-audio",
      payload: frame,
    });
  }

  subscribe(topic: EngineEvent["topic"]) {
    chrome.runtime.sendMessage({
      type: "engine/subscribe",
      payload: { topic },
    });
  }
}

class ContentController {
  private mediaStream?: MediaStream;
  private capture?: ReturnType<typeof createMediaRecorderFactory>["create"];
  private readonly bridge = new ContentBridge();

  async start(): Promise<void> {
    if (this.mediaStream) {
      return;
    }
    this.bridge.connect();
    this.bridge.subscribe("BoundaryDetected");
    this.bridge.subscribe("AsrPartial");
    this.bridge.subscribe("AsrFinal");
    this.bridge.subscribe("NmtPartial");
    this.bridge.subscribe("NmtFinal");
    this.bridge.subscribe("EmotionTag");
    this.bridge.subscribe("TtsChunk");

    this.mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
    const factory = createMediaRecorderFactory();
    this.capture = factory.create(this.mediaStream);
    this.capture.onFrame((frame) => this.bridge.sendAudio(frame));
    await this.capture.start();
  }

  async stop(): Promise<void> {
    if (this.capture) {
      await this.capture.stop();
      this.capture = undefined;
    }
    if (this.mediaStream) {
      this.mediaStream.getTracks().forEach((track) => track.stop());
      this.mediaStream = undefined;
    }
  }
}

const controller = new ContentController();

chrome.runtime.onMessage.addListener((command: ContentCommand, _sender, sendResponse) => {
  const handle = async () => {
    if (command.type === "capture/start") {
      await controller.start();
      sendResponse({ ok: true });
    } else if (command.type === "capture/stop") {
      await controller.stop();
      sendResponse({ ok: true });
    }
  };
  void handle().catch((error) => {
    sendResponse({ ok: false, error: error instanceof Error ? error.message : String(error) });
  });
  return true;
});

