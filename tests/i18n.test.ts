import { describe, expect, it } from "vitest";

import {
  getMessages,
  resolveInitialLocale,
  supportedLocales,
} from "../src/i18n";

describe("localization", () => {
  it("exposes English and Simplified Chinese catalogs", () => {
    expect(supportedLocales).toEqual(["en", "zh-CN"]);
    expect(getMessages("en").placeholderTitle).toContain("perspective");
    expect(getMessages("zh-CN").placeholderTitle).toContain("自己");
    expect(getMessages("en").capture.enterReview).toBe("Review what to remember");
    expect(getMessages("zh-CN").capture.enterReview).toBe("整理要记住的内容");
    expect(Object.keys(getMessages("zh-CN").capture)).toEqual(
      Object.keys(getMessages("en").capture),
    );
    expect(Object.keys(getMessages("zh-CN").capture.errors)).toEqual(
      Object.keys(getMessages("en").capture.errors),
    );
  });

  it("selects Simplified Chinese for Chinese browser locales", () => {
    expect(resolveInitialLocale("zh-CN")).toBe("zh-CN");
    expect(resolveInitialLocale("zh-Hans-US")).toBe("zh-CN");
  });

  it("falls back to English for unsupported or missing locales", () => {
    expect(resolveInitialLocale("fr-FR")).toBe("en");
    expect(resolveInitialLocale()).toBe("en");
  });
});
