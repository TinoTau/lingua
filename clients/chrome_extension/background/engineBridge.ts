import {
  AudioFrame,
  EngineEvent,
  EngineEventTopic,
} from "../shared/coreTypes";

export type EngineEventHandler = (event: EngineEvent) => void;

export interface EngineSubscription {
  topic: EngineEventTopic;
  unsubscribe: () => void;
}

export interface CoreEngineBridge {
  boot(): Promise<void>;
  shutdown(): Promise<void>;
  submitAudioFrame(frame: AudioFrame): Promise<void>;
  subscribe(topic: EngineEventTopic, handler: EngineEventHandler): EngineSubscription;
}

export interface BridgeFactoryOptions {
  wasmModulePath: string;
  telemetryEndpoint?: string;
  configEndpoint?: string;
}

export interface CoreEngineBridgeFactory {
  create(options: BridgeFactoryOptions): Promise<CoreEngineBridge>;
}
