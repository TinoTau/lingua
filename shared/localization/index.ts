export type LocaleKey =
  | "app.title"
  | "app.subtitle"
  | "action.start"
  | "action.stop"
  | "status.ready"
  | "status.buffering";

type LocaleTable = Record<LocaleKey, string>;

const enUS: LocaleTable = {
  "app.title": "Lingua Translator",
  "app.subtitle": "Real-time multi-lingual assistant",
  "action.start": "Start",
  "action.stop": "Stop",
  "status.ready": "Ready",
  "status.buffering": "Buffering...",
};

const zhCN: LocaleTable = {
  "app.title": "Lingua 翻译助手",
  "app.subtitle": "实时多语言智能助手",
  "action.start": "开始",
  "action.stop": "停止",
  "status.ready": "就绪",
  "status.buffering": "缓冲中…",
};

const locales: Record<string, LocaleTable> = {
  "en-US": enUS,
  "zh-CN": zhCN,
};

export function t(key: LocaleKey, locale: string = "en-US"): string {
  const table = locales[locale] ?? locales["en-US"];
  return table[key] ?? key;
}

