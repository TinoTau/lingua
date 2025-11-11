import {
  EngineEvent,
  EngineEventTopic,
  PartialTranscript,
  StableTranscript,
  EmotionPayload,
  TtsChunkPayload,
} from "../shared/coreTypes";

export interface TranscriptState {
  partial?: PartialTranscript;
  final?: StableTranscript;
}

export interface EmotionState {
  current?: EmotionPayload;
  history: EmotionPayload[];
}

export interface AudioPlaybackState {
  buffering: boolean;
  lastChunk?: TtsChunkPayload;
}

export interface EngineUiState {
  ready: boolean;
  transcript: TranscriptState;
  emotion: EmotionState;
  audio: AudioPlaybackState;
  errors: string[];
}

export type UiEventReducer = (state: EngineUiState, event: EngineEvent) => EngineUiState;

export type UiEventMap = Partial<Record<EngineEventTopic, UiEventReducer>>;

export interface UiStateStore {
  getState(): EngineUiState;
  subscribe(listener: (state: EngineUiState) => void): () => void;
  applyEvent(event: EngineEvent): void;
}
