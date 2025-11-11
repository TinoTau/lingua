export type EngineEventTopic =
  | "BoundaryDetected"
  | "AsrPartial"
  | "AsrFinal"
  | "NmtPartial"
  | "NmtFinal"
  | "EmotionTag"
  | "TtsChunk"
  | "EngineError";

export interface EngineEvent<TPayload = unknown> {
  topic: EngineEventTopic;
  timestampMs: number;
  payload: TPayload;
}

export interface AudioFrame {
  sampleRate: number;
  channels: number;
  data: Float32Array;
  timestampMs: number;
}

export interface PartialTranscript {
  text: string;
  confidence: number;
  isFinal: boolean;
}

export interface StableTranscript {
  text: string;
  speakerId?: string;
  language: string;
}

export interface AsrResultPayload {
  partial?: PartialTranscript;
  finalTranscript?: StableTranscript;
}

export interface TranslationPayload {
  translatedText: string;
  isStable: boolean;
}

export interface EmotionPayload {
  label: string;
  confidence: number;
}

export interface TtsChunkPayload {
  audio: ArrayBuffer;
  timestampMs: number;
  isLast: boolean;
}

export interface EngineBootMetricsPayload {
  mode: string;
  sourceLanguage: string;
  targetLanguage: string;
}
