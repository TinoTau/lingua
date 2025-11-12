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

export interface BridgeRuntimeContext {
  options: BridgeFactoryOptions;
  postEvent(event: EngineEvent): void;
  onLifecycleError(error: unknown): void;
}

export interface CoreEngineBridgeRuntime {
  initialize(context: BridgeRuntimeContext): Promise<void>;
  teardown(): Promise<void>;
  pushFrame(frame: AudioFrame): Promise<void>;
}

export interface BridgeRuntimeFactory {
  createRuntime(options: BridgeFactoryOptions): Promise<CoreEngineBridgeRuntime>;
}

export type EngineBridgeDependencies = {
  runtimeFactory: BridgeRuntimeFactory;
  logger?: {
    info(message: string, meta?: Record<string, unknown>): void;
    error(message: string, meta?: Record<string, unknown>): void;
  };
};
