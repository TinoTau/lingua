import { EngineEvent } from "../shared/coreTypes";
import { EngineEventEnvelope } from "../shared/messages";

export type ChannelName = "content" | "popup" | "options";

interface PortRegistryEntry {
  port: chrome.runtime.Port;
  disconnect: () => void;
}

export class ChannelHub {
  private readonly ports = new Map<ChannelName, Set<PortRegistryEntry>>();

  register(channel: ChannelName, port: chrome.runtime.Port): void {
    const entries = this.ports.get(channel) ?? new Set<PortRegistryEntry>();
    const entry: PortRegistryEntry = {
      port,
      disconnect: () => {
        entries.delete(entry);
        port.disconnect();
      },
    };
    port.onDisconnect.addListener(() => entries.delete(entry));
    entries.add(entry);
    this.ports.set(channel, entries);
  }

  broadcast(channel: ChannelName, event: EngineEvent): void {
    const entries = this.ports.get(channel);
    if (!entries) {
      return;
    }
    const envelope: EngineEventEnvelope = { channel, event };
    for (const entry of entries) {
      entry.port.postMessage(envelope);
    }
  }

  broadcastAll(event: EngineEvent): void {
    for (const channel of this.ports.keys()) {
      this.broadcast(channel, event);
    }
  }

  close(): void {
    for (const entries of this.ports.values()) {
      for (const entry of entries) {
        entry.disconnect();
      }
      entries.clear();
    }
    this.ports.clear();
  }
}

