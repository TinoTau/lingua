import { AudioFrame, EngineEvent } from "./coreTypes";

export type BackgroundCommandType =
  | "engine/boot"
  | "engine/shutdown"
  | "engine/push-audio"
  | "engine/subscribe"
  | "engine/unsubscribe";

export interface BackgroundCommandBase<TType extends BackgroundCommandType, TPayload> {
  type: TType;
  payload: TPayload;
}

export type BootCommand = BackgroundCommandBase<"engine/boot", undefined>;
export type ShutdownCommand = BackgroundCommandBase<"engine/shutdown", undefined>;
export type PushAudioCommand = BackgroundCommandBase<"engine/push-audio", AudioFrame>;
export type SubscribeCommand = BackgroundCommandBase<"engine/subscribe", { topic: string }>;
export type UnsubscribeCommand = BackgroundCommandBase<"engine/unsubscribe", { topic: string }>;

export type BackgroundCommand =
  | BootCommand
  | ShutdownCommand
  | PushAudioCommand
  | SubscribeCommand
  | UnsubscribeCommand;

export interface BackgroundResponse<TPayload = unknown> {
  ok: boolean;
  payload?: TPayload;
  error?: string;
}

export interface EngineEventEnvelope {
  channel: string;
  event: EngineEvent;
}
