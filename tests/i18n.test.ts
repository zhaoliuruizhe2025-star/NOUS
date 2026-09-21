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
    expect(Object.keys(getMessages("zh-CN").navigation)).toEqual(
      Object.keys(getMessages("en").navigation),
    );
    expect(Object.keys(getMessages("zh-CN").history)).toEqual(
      Object.keys(getMessages("en").history),
    );
    expect(Object.keys(getMessages("zh-CN").history.errors)).toEqual(
      Object.keys(getMessages("en").history.errors),
    );
    expect(getMessages("en").history.scope).toContain("not a complete view");
    expect(getMessages("zh-CN").history.scope).toContain("不代表");
    expect(getMessages("en").history.order).toContain("does not represent when events occurred");
    expect(getMessages("zh-CN").history.order).toContain("不代表事情实际发生");
    expect(getMessages("en").history.conviction).toContain("you believed");
    expect(getMessages("zh-CN").history.conviction).toContain("你当时");
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
