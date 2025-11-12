import { EngineEvent, EngineEventTopic } from "../shared/coreTypes";
import {
  EngineUiState,
  UiEventMap,
  UiEventReducer,
  UiStateStore,
} from "./engineState";

const initialState: EngineUiState = {
  ready: false,
  transcript: {},
  emotion: { history: [] },
  audio: { buffering: true },
  errors: [],
};

export class DefaultUiStateStore implements UiStateStore {
  private state: EngineUiState = initialState;
  private readonly listeners = new Set<(state: EngineUiState) => void>();

  constructor(private readonly reducers: UiEventMap) {}

  getState(): EngineUiState {
    return this.state;
  }

  subscribe(listener: (state: EngineUiState) => void): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => this.listeners.delete(listener);
  }

  applyEvent(event: EngineEvent): void {
    const reducer = this.reducers[event.topic];
    if (!reducer) {
      return;
    }
    this.state = reducer(this.state, event);
    this.emit();
  }

  private emit() {
    for (const listener of this.listeners) {
      listener(this.state);
    }
  }
}

export function createDefaultReducers(): UiEventMap {
  const reducers: UiEventMap = {};

  reducers["BoundaryDetected" satisfies EngineEventTopic] = (state) => ({
    ...state,
    ready: true,
  });

  reducers["AsrPartial" satisfies EngineEventTopic] = (state, event) => ({
    ...state,
    transcript: {
      ...state.transcript,
      partial: event.payload,
    },
  });

  reducers["AsrFinal" satisfies EngineEventTopic] = (state, event) => ({
    ...state,
    transcript: {
      partial: undefined,
      final: event.payload,
    },
  });

  reducers["EmotionTag" satisfies EngineEventTopic] = (state, event) => ({
    ...state,
    emotion: {
      current: event.payload,
      history: [...state.emotion.history, event.payload].slice(-20),
    },
  });

  reducers["TtsChunk" satisfies EngineEventTopic] = (state, event) => ({
    ...state,
    audio: {
      buffering: !event.payload || !event.payload.isLast,
      lastChunk: event.payload,
    },
  });

  reducers["EngineError" satisfies EngineEventTopic] = (state, event) => ({
    ...state,
    errors: [...state.errors, event.payload?.message ?? "Unknown error"],
  });

  return reducers;
}

export function createUiStore(reducers?: UiEventMap): UiStateStore {
  return new DefaultUiStateStore(reducers ?? createDefaultReducers());
}

