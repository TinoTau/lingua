import {
  AudioFrame,
  EngineEvent,
  EngineEventTopic,
} from "../shared/coreTypes";
import {
  BridgeFactoryOptions,
  BridgeRuntimeContext,
  CoreEngineBridge,
  CoreEngineBridgeFactory,
  CoreEngineBridgeRuntime,
  EngineBridgeDependencies,
  EngineEventHandler,
  EngineSubscription,
} from "./engineBridge";

class SubscriptionRegistry {
  private handlers = new Map<EngineEventTopic, Set<EngineEventHandler>>();

  add(topic: EngineEventTopic, handler: EngineEventHandler): () => void {
    const set = this.handlers.get(topic) ?? new Set<EngineEventHandler>();
    set.add(handler);
    this.handlers.set(topic, set);
    return () => this.remove(topic, handler);
  }

  dispatch(event: EngineEvent) {
    const set = this.handlers.get(event.topic);
    if (!set) {
      return;
    }
    for (const handler of set.values()) {
      handler(event);
    }
  }

  clear() {
    this.handlers.clear();
  }

  private remove(topic: EngineEventTopic, handler: EngineEventHandler) {
    const set = this.handlers.get(topic);
    if (!set) {
      return;
    }
    set.delete(handler);
    if (set.size === 0) {
      this.handlers.delete(topic);
    }
  }
}

class DefaultCoreEngineBridge implements CoreEngineBridge {
  private runtime?: CoreEngineBridgeRuntime;
  private readonly subscriptions = new SubscriptionRegistry();

  constructor(
    private readonly deps: EngineBridgeDependencies,
    private readonly options: BridgeFactoryOptions,
  ) {}

  async boot(): Promise<void> {
    if (this.runtime) {
      return;
    }
    const runtime = await this.deps.runtimeFactory.createRuntime(this.options);
    const context: BridgeRuntimeContext = {
      options: this.options,
      postEvent: (event) => this.subscriptions.dispatch(event),
      onLifecycleError: (error) => {
        this.deps.logger?.error("core-engine runtime error", { error });
      },
    };
    await runtime.initialize(context);
    this.runtime = runtime;
    this.deps.logger?.info("core-engine bridge booted");
  }

  async shutdown(): Promise<void> {
    if (!this.runtime) {
      return;
    }
    try {
      await this.runtime.teardown();
    } finally {
      this.runtime = undefined;
      this.subscriptions.clear();
    }
    this.deps.logger?.info("core-engine bridge shut down");
  }

  async submitAudioFrame(frame: AudioFrame): Promise<void> {
    if (!this.runtime) {
      throw new Error("core engine not booted");
    }
    await this.runtime.pushFrame(frame);
  }

  subscribe(topic: EngineEventTopic, handler: EngineEventHandler): EngineSubscription {
    const unsubscribe = this.subscriptions.add(topic, handler);
    return { topic, unsubscribe };
  }
}

export function createCoreEngineBridgeFactory(
  deps: EngineBridgeDependencies,
): CoreEngineBridgeFactory {
  return {
    async create(options: BridgeFactoryOptions): Promise<CoreEngineBridge> {
      return new DefaultCoreEngineBridge(deps, options);
    },
  };
}

