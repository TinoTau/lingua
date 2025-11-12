export interface EngineEventEnvelope<TPayload = unknown> {
  schemaVersion: number;
  event: string;
  timestampMs: number;
  payload: TPayload;
  meta?: Record<string, unknown>;
}

export const ENGINE_EVENT_SCHEMA_VERSION = 1;

