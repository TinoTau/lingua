import { AudioFrame, EngineEvent } from "../../../clients/chrome_extension/shared/coreTypes";
import {
  BridgeFactoryOptions,
  BridgeRuntimeContext,
  BridgeRuntimeFactory,
  CoreEngineBridgeRuntime,
} from "../../../clients/chrome_extension/background/engineBridge";

export interface WasmEngineModule {
  initialize(options: BridgeFactoryOptions): Promise<void>;
  dispose(): Promise<void>;
  pushAudioFrame(frame: AudioFrame): Promise<void>;
  pollEvents(): Promise<EngineEvent[]>;
}

export interface WasmModuleLoader {
  load(path: string): Promise<WasmEngineModule>;
}

class PollingEngineRuntime implements CoreEngineBridgeRuntime {
  private module?: WasmEngineModule;
  private context?: BridgeRuntimeContext;
  private pollTimer?: number;

  constructor(private readonly loader: WasmModuleLoader, private readonly pollIntervalMs = 50) {}

  async initialize(context: BridgeRuntimeContext): Promise<void> {
    if (this.module) {
      return;
    }
    this.context = context;
    this.module = await this.loader.load(context.options.wasmModulePath);
    await this.module.initialize(context.options);
    this.startPolling();
  }

  async teardown(): Promise<void> {
    if (!this.module) {
      return;
    }
    this.stopPolling();
    await this.module.dispose();
    this.module = undefined;
    this.context = undefined;
  }

  async pushFrame(frame: AudioFrame): Promise<void> {
    if (!this.module) {
      throw new Error("WASM module not loaded");
    }
    await this.module.pushAudioFrame(frame);
  }

  private startPolling() {
    if (!this.module || !this.context) {
      return;
    }
    const poll = async () => {
      if (!this.module || !this.context) {
        return;
      }
      try {
        const events = await this.module.pollEvents();
        for (const event of events) {
          this.context.postEvent(event);
        }
      } catch (error) {
        this.context?.onLifecycleError(error);
      } finally {
        this.pollTimer = setTimeout(poll, this.pollIntervalMs) as unknown as number;
      }
    };
    poll();
  }

  private stopPolling() {
    if (this.pollTimer !== undefined) {
      clearTimeout(this.pollTimer);
      this.pollTimer = undefined;
    }
  }
}

export function createBridgeRuntimeFactory(loader: WasmModuleLoader): BridgeRuntimeFactory {
  return {
    async createRuntime(): Promise<CoreEngineBridgeRuntime> {
      return new PollingEngineRuntime(loader);
    },
  };
}

