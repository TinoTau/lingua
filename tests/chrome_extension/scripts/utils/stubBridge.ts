import { BackgroundCommandRouter } from "../../../../clients/chrome_extension/background/commandRouter";
import { createCoreEngineBridgeFactory } from "../../../../clients/chrome_extension/background/engineBridgeFactory";
import {
  AudioFrame,
  EngineEvent,
} from "../../../../clients/chrome_extension/shared/coreTypes";
import {
  BridgeRuntimeContext,
  CoreEngineBridgeRuntime,
} from "../../../../clients/chrome_extension/background/engineBridge";

class StubRuntime implements CoreEngineBridgeRuntime {
  private context?: BridgeRuntimeContext;
  private readonly batches: EngineEvent[][];

  constructor(batches: EngineEvent[][]) {
    this.batches = batches;
  }

  async initialize(context: BridgeRuntimeContext): Promise<void> {
    this.context = context;
  }

  async teardown(): Promise<void> {
    this.context = undefined;
  }

  async pushFrame(_frame: AudioFrame): Promise<void> {
    const batch = this.batches.shift() ?? [];
    if (!this.context) {
      throw new Error("runtime not initialized");
    }
    for (const event of batch) {
      this.context.postEvent(event);
    }
  }
}

export interface StubBridgeResult {
  router: BackgroundCommandRouter;
  recorded: EngineEvent[];
}

export function createStubBridge(eventBatches: EngineEvent[][]): StubBridgeResult {
  const runtime = new StubRuntime(eventBatches);
  const bridgeFactory = createCoreEngineBridgeFactory({
    runtimeFactory: {
      async createRuntime() {
        return runtime;
      },
    },
    logger: console,
  });

  const recorded: EngineEvent[] = [];

  const router = new BackgroundCommandRouter({
    bridgeFactory,
    bridgeOptions: { wasmModulePath: "stub.wasm" },
    logger: console,
    onEvent: (event) => recorded.push(event),
  });

  return { router, recorded };
}

