import { EngineEvent } from "../shared/coreTypes";
import { EngineEventHandler, EngineSubscription } from "./engineBridge";

export type RelayChannel = "popup" | "content" | "options";

export interface RelayRegistration {
  channel: RelayChannel;
  disconnect: () => void;
}

export interface EngineEventRelay {
  register(channel: RelayChannel, handler: EngineEventHandler): RelayRegistration;
  emitToChannel(channel: RelayChannel, event: EngineEvent): void;
  forward(subscription: EngineSubscription): void;
}
