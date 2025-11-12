export interface EngineFeatureFlags {
  enableCloudEnhancements: boolean;
  enableEmotionAdapter: boolean;
  enablePersonaAdapter: boolean;
}

export interface EngineRuntimeConfig {
  mode: "fast" | "balanced" | "quality";
  sourceLanguage: string;
  targetLanguage: string;
  featureFlags: EngineFeatureFlags;
}

export const defaultEngineConfig: EngineRuntimeConfig = {
  mode: "balanced",
  sourceLanguage: "en",
  targetLanguage: "zh",
  featureFlags: {
    enableCloudEnhancements: false,
    enableEmotionAdapter: true,
    enablePersonaAdapter: true,
  },
};

