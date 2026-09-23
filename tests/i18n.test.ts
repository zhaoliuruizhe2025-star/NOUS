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
    expect(Object.keys(getMessages("zh-CN").candidateReview)).toEqual(
      Object.keys(getMessages("en").candidateReview),
    );
    expect(Object.keys(getMessages("zh-CN").candidateReview.roles)).toEqual(
      Object.keys(getMessages("en").candidateReview.roles),
    );
    expect(Object.keys(getMessages("zh-CN").candidateReview.errors)).toEqual(
      Object.keys(getMessages("en").candidateReview.errors),
    );
    expect(getMessages("en").candidateReview.title).toBe("Candidate review");
    expect(getMessages("zh-CN").candidateReview.title).toBe("候选模式审阅");
    expect(getMessages("en").candidateReview.roles.contradicting).toBe("Contradicting");
    expect(getMessages("zh-CN").candidateReview.roles.contradicting).toBe("与候选说法相矛盾");
    expect(getMessages("en").candidateReview.unclassified).toBe("Unclassified");
    expect(getMessages("zh-CN").candidateReview.unclassified).toBe("尚未分类");
    expect(getMessages("en").candidateReview.notSaved).toContain("not saved");
    expect(getMessages("zh-CN").candidateReview.notSaved).toContain("不会保存");
    expect(getMessages("en").candidateReview.sourceChanged).toContain("review again");
    expect(getMessages("zh-CN").candidateReview.sourceChanged).toContain("重新审阅");
    expect(getMessages("en").candidateReview.sourceNotFound).toContain("no longer exists");
    expect(getMessages("zh-CN").candidateReview.sourceNotFound).toContain("不存在");
    expect(getMessages("en").candidateReview.dependence).toContain("does not establish independent support");
    expect(getMessages("zh-CN").candidateReview.dependence).toContain("不证明证据独立");
    expect(getMessages("en").candidateReview.noChronology).toContain("Save order does not establish");
    expect(getMessages("zh-CN").candidateReview.noChronology).toContain("保存顺序不能说明");
    expect(Object.keys(getMessages("zh-CN").semanticComparison)).toEqual(
      Object.keys(getMessages("en").semanticComparison),
    );
    for (const key of ["sourceTypes", "comparability", "meaning", "experience", "errors"] as const) {
      expect(Object.keys(getMessages("zh-CN").semanticComparison[key])).toEqual(
        Object.keys(getMessages("en").semanticComparison[key]),
      );
    }
    expect(getMessages("en").semanticComparison.anchorLabel).toBe("Record to compare others with");
    expect(getMessages("zh-CN").semanticComparison.anchorLabel).toBe("作为比较起点的记录");
    expect(getMessages("en").semanticComparison.unanswered).toBe("Not answered");
    expect(getMessages("zh-CN").semanticComparison.unanswered).toBe("尚未回答");
    expect(getMessages("en").semanticComparison.comparability.cannotTell).toBe("Cannot tell");
    expect(getMessages("zh-CN").semanticComparison.comparability.cannotTell).toBe("无法判断");
    expect(getMessages("en").semanticComparison.limitations).toContain("no relation among the other records");
    expect(getMessages("zh-CN").semanticComparison.limitations).toContain("未推断其他记录之间的关系");
    expect(getMessages("en").semanticComparison.limitations).toContain("do not establish independent evidence");
    expect(getMessages("zh-CN").semanticComparison.limitations).toContain("不证明证据独立");
    expect(Object.keys(getMessages("zh-CN").systemCandidate)).toEqual(Object.keys(getMessages("en").systemCandidate));
    for (const key of ["classifications", "experience", "errors", "zeroReasons"] as const) {
      expect(Object.keys(getMessages("zh-CN").systemCandidate[key])).toEqual(
        Object.keys(getMessages("en").systemCandidate[key]),
      );
    }
    expect(getMessages("en").systemCandidate.xLabel).toBe("User-defined Thought content");
    expect(getMessages("zh-CN").systemCandidate.xLabel).toBe("用户定义的想法内容");
    expect(getMessages("en").systemCandidate.conclusion).toContain("at least two experiences you identified as different");
    expect(getMessages("zh-CN").systemCandidate.conclusion).toContain("至少两段你标为不同的经历");
    expect(getMessages("en").systemCandidate.conclusion).toContain("within the Thought records reviewed here");
    expect(getMessages("zh-CN").systemCandidate.conclusion).toContain("在本次审阅的想法记录中");
    expect(getMessages("en").systemCandidate.cannotTellSummary).toContain("could not tell");
    expect(getMessages("zh-CN").systemCandidate.cannotTellSummary).toContain("无法判断");
    expect(JSON.stringify(getMessages("en").systemCandidate)).not.toMatch(/\bPattern\b|\btendency\b|\balways\b|\bconfidence\b/i);
    expect(JSON.stringify(getMessages("zh-CN").systemCandidate)).not.toMatch(/模式|倾向|总是|置信/);
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
