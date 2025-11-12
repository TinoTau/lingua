import { EngineEvent } from "../shared/coreTypes";
import { EngineEventHandler, EngineSubscription } from "./engineBridge";
import { EngineEventRelay, RelayChannel, RelayRegistration } from "./eventRelay";

class ChannelRegistry {
  private handlers = new Map<RelayChannel, Set<EngineEventHandler>>();

  register(channel: RelayChannel, handler: EngineEventHandler): () => void {
    const set = this.handlers.get(channel) ?? new Set<EngineEventHandler>();
    set.add(handler);
    this.handlers.set(channel, set);
    return () => this.unregister(channel, handler);
  }

  emit(channel: RelayChannel, event: EngineEvent) {
    const set = this.handlers.get(channel);
    if (!set) {
      return;
    }
    for (const handler of set.values()) {
      handler(event);
    }
  }

  clearChannel(channel: RelayChannel) {
    this.handlers.delete(channel);
  }

  private unregister(channel: RelayChannel, handler: EngineEventHandler) {
    const set = this.handlers.get(channel);
    if (!set) {
      return;
    }
    set.delete(handler);
    if (set.size === 0) {
      this.handlers.delete(channel);
    }
  }
}

export class DefaultEngineEventRelay implements EngineEventRelay {
  private readonly channels = new ChannelRegistry();
  private readonly forwarded = new Set<EngineSubscription>();

  register(channel: RelayChannel, handler: EngineEventHandler): RelayRegistration {
    const disconnect = this.channels.register(channel, handler);
    return {
      channel,
      disconnect: () => {
        disconnect();
        this.channels.clearChannel(channel);
      },
    };
  }

  emitToChannel(channel: RelayChannel, event: EngineEvent): void {
    this.channels.emit(channel, event);
  }

  forward(subscription: EngineSubscription): void {
    this.forwarded.add(subscription);
  }

  dispose(): void {
    for (const subscription of this.forwarded.values()) {
      subscription.unsubscribe();
    }
    this.forwarded.clear();
  }
}

