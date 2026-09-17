import en from "./en/common.json";
import zhCN from "./zh-CN/common.json";

export const supportedLocales = ["en", "zh-CN"] as const;

export type Locale = (typeof supportedLocales)[number];
export type Messages = typeof en;

const catalogs: Record<Locale, Messages> = {
  en,
  "zh-CN": zhCN,
};

export function getMessages(locale: Locale): Messages {
  return catalogs[locale];
}

export function resolveInitialLocale(language?: string): Locale {
  return language?.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}

