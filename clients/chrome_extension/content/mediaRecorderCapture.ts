import { AudioFrame } from "../shared/coreTypes";
import { AudioCaptureFactory, AudioCaptureStream } from "./audioCapture";

interface MediaRecorderCaptureOptions {
  sampleRate: number;
  channelCount: number;
  chunkDurationMs: number;
}

class MediaRecorderAudioCapture implements AudioCaptureStream {
  private frameCallback?: (frame: AudioFrame) => void;
  private startTimestamp?: number;
  private audioContext?: AudioContext;
  private processor?: ScriptProcessorNode;

  constructor(
    private readonly source: MediaStream,
    private readonly options: MediaRecorderCaptureOptions,
  ) {}

  async start(): Promise<void> {
    if (this.audioContext) {
      return;
    }
    const context = new AudioContext({ sampleRate: this.options.sampleRate });
    const sourceNode = context.createMediaStreamSource(this.source);
    const processor = context.createScriptProcessor(4096, this.options.channelCount, this.options.channelCount);
    sourceNode.connect(processor);
    processor.connect(context.destination);

    processor.onaudioprocess = (event) => {
      const buffer = event.inputBuffer;
      const data = buffer.getChannelData(0);
      const frame: AudioFrame = {
        sampleRate: buffer.sampleRate,
        channels: this.options.channelCount,
        data: Array.from(data),
        timestampMs: performance.now() - (this.startTimestamp ?? performance.now()),
      };
      this.frameCallback?.(frame);
    };
    this.startTimestamp = performance.now();
    this.audioContext = context;
    this.processor = processor;
  }

  async stop(): Promise<void> {
    this.source.getTracks().forEach((track) => track.stop());
    this.processor?.disconnect();
    await this.audioContext?.close();
    this.processor = undefined;
    this.audioContext = undefined;
    this.startTimestamp = undefined;
  }

  onFrame(callback: (frame: AudioFrame) => void): void {
    this.frameCallback = callback;
  }
}

export function createMediaRecorderFactory(
  options: Partial<MediaRecorderCaptureOptions> = {},
): AudioCaptureFactory {
  const captureOptions: MediaRecorderCaptureOptions = {
    sampleRate: options.sampleRate ?? 16000,
    channelCount: options.channelCount ?? 1,
    chunkDurationMs: options.chunkDurationMs ?? 100,
  };

  return {
    create(source: MediaStream): AudioCaptureStream {
      return new MediaRecorderAudioCapture(source, captureOptions);
    },
  };
}

