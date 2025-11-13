import { EngineEventTopic } from "../shared/coreTypes";
import {
  BackgroundCommand,
  BackgroundResponse,
  BootCommand,
  PushAudioCommand,
  ShutdownCommand,
  SubscribeCommand,
  UnsubscribeCommand,
} from "../shared/messages";
import { CoreEngineBridge, CoreEngineBridgeFactory, EngineSubscription } from "./engineBridge";

interface CommandRouterDeps {
  bridgeFactory: CoreEngineBridgeFactory;
  bridgeOptions: Parameters<CoreEngineBridgeFactory["create"]>[0];
  logger?: {
    info(message: string, meta?: Record<string, unknown>): void;
    warn(message: string, meta?: Record<string, unknown>): void;
    error(message: string, meta?: Record<string, unknown>): void;
  };
  onEvent?: (event: Parameters<CoreEngineBridge["subscribe"]>[1] extends (event: infer T) => void ? T : never) => void;
}

export class BackgroundCommandRouter {
  private bridge?: CoreEngineBridge;
  private subscriptions = new Map<string, EngineSubscription>();

  constructor(private readonly deps: CommandRouterDeps) {}

  async handle(command: BackgroundCommand): Promise<BackgroundResponse> {
    switch (command.type) {
      case "engine/boot":
        return this.handleBoot(command);
      case "engine/shutdown":
        return this.handleShutdown(command);
      case "engine/push-audio":
        return this.handlePushAudio(command);
      case "engine/subscribe":
        return this.handleSubscribe(command);
      case "engine/unsubscribe":
        return this.handleUnsubscribe(command);
      default:
        return { ok: false, error: `Unknown command ${command["type"]}` };
    }
  }

  private async handleBoot(_command: BootCommand): Promise<BackgroundResponse> {
    if (!this.bridge) {
      this.bridge = await this.deps.bridgeFactory.create(this.deps.bridgeOptions);
    }
    await this.bridge.boot();
    this.deps.logger?.info("core engine booted");
    return { ok: true };
  }

  private async handleShutdown(_command: ShutdownCommand): Promise<BackgroundResponse> {
    await this.bridge?.shutdown();
    this.bridge = undefined;
    this.clearSubscriptions();
    this.deps.logger?.info("core engine shut down");
    return { ok: true };
  }

  private async handlePushAudio(command: PushAudioCommand): Promise<BackgroundResponse> {
    if (!this.bridge) {
      return { ok: false, error: "core engine not booted" };
    }
    await this.bridge.submitAudioFrame(command.payload);
    return { ok: true };
  }

  private async handleSubscribe(command: SubscribeCommand): Promise<BackgroundResponse> {
    if (!this.bridge) {
      return { ok: false, error: "core engine not booted" };
    }
    const topic = command.payload.topic as EngineEventTopic;
    if (!this.subscriptions.has(topic)) {
      const subscription = this.bridge.subscribe(topic, (event) => {
        this.deps.logger?.info("event received", { topic: event.topic });
        this.deps.onEvent?.(event);
      });
      this.subscriptions.set(topic, subscription);
    }
    return { ok: true };
  }

  private async handleUnsubscribe(command: UnsubscribeCommand): Promise<BackgroundResponse> {
    const topic = command.payload.topic as EngineEventTopic;
    const subscription = this.subscriptions.get(topic);
    if (subscription) {
      subscription.unsubscribe();
      this.subscriptions.delete(topic);
    }
    return { ok: true };
  }

  private clearSubscriptions() {
    for (const subscription of this.subscriptions.values()) {
      subscription.unsubscribe();
    }
    this.subscriptions.clear();
  }
}

