import { AudioFrame } from "../shared/coreTypes";

export interface AudioCaptureStream {
  start(): Promise<void>;
  stop(): Promise<void>;
  onFrame(callback: (frame: AudioFrame) => void): void;
}

export interface AudioCaptureFactory {
  create(source: MediaStream): AudioCaptureStream;
}
