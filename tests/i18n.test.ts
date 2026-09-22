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
    expect(Object.keys(getMessages("zh-CN").correction)).toEqual(
      Object.keys(getMessages("en").correction),
    );
    expect(Object.keys(getMessages("zh-CN").correction.errors)).toEqual(
      Object.keys(getMessages("en").correction.errors),
    );
    expect(getMessages("en").correction.explanation).toContain("changed later");
    expect(getMessages("zh-CN").correction.explanation).toContain("后来真的发生了变化");
    expect(getMessages("en").correction.priorMeaning).toContain("not true past user states");
    expect(getMessages("zh-CN").correction.priorMeaning).toContain("并非用户真实的过去状态");
    expect(Object.keys(getMessages("zh-CN").deletion)).toEqual(Object.keys(getMessages("en").deletion));
    expect(Object.keys(getMessages("zh-CN").deletion.errors)).toEqual(Object.keys(getMessages("en").deletion.errors));
    expect(getMessages("en").deletion.confirm).toBe("Permanently delete");
    expect(getMessages("zh-CN").deletion.confirm).toBe("永久删除");
    expect(getMessages("en").deletion.consequence).toContain("cannot be undone");
    expect(getMessages("zh-CN").deletion.consequence).toContain("无法撤销");
    expect(Object.keys(getMessages("zh-CN").portability)).toEqual(Object.keys(getMessages("en").portability));
    expect(Object.keys(getMessages("zh-CN").portability.errors)).toEqual(Object.keys(getMessages("en").portability.errors));
    expect(getMessages("en").portability.exportExplanation).toContain("JSON");
    expect(getMessages("zh-CN").portability.exportExplanation).toContain("JSON");
    expect(getMessages("en").portability.backupExplanation).toContain("Restore is not available");
    expect(getMessages("zh-CN").portability.backupExplanation).toContain("尚不支持在应用内恢复");
    expect(getMessages("en").portability.privacy).toContain("does not automatically upload");
    expect(getMessages("zh-CN").portability.privacy).toContain("不会自动上传");
    expect(getMessages("en").portability.privacy).toContain("does not automatically upload it or add NOUS-specific encryption");
    expect(getMessages("zh-CN").portability.privacy).toContain("不会为文件额外添加 NOUS 专用加密");
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
