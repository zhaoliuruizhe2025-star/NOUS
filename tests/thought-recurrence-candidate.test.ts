/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import {
  candidateRequestIsCurrent,
  changedScopeSources,
  deriveCandidate,
  dismissCandidate,
  editClassification,
  editX,
  failCandidateLoad,
  finishChecking,
  handoffFromReviewedComparison,
  isValidHandoff,
  readyForCandidateAnalysis,
  restoreCheckedResult,
  returnToGrounding,
  staleCandidate,
  startCandidateAttempt,
  startChecking,
  syncComparisonEdit,
  type CandidateHandoff,
  type CandidateState,
  type Classification,
  type ThoughtSource,
} from "../src/app/thought_recurrence_candidate";
import {
  answerComparability,
  answerExperience,
  answerMeaning,
  createComparisonRefreshGate,
  type ComparisonDraft,
} from "../src/app/semantic_comparison";
import { SystemCandidateReview } from "../src/components/SystemCandidateReview";
import { getMessages } from "../src/i18n";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const read = (path: string) => readFileSync(join(root, path), "utf8");

function thought(id: string, context = false): ThoughtSource {
  return {
    recordType: "thought", id, content: `Thought ${id}`, stateToken: `${id}-token`,
    linkedSituation: context ? { id: "s1", description: "Current situation", stateToken: "s1-token" } : null,
  };
}

function handoff(ids = ["a", "b", "c"]): CandidateHandoff {
  return {
    sourceType: "thought", comparisonFrame: "  My own frame  ",
    selectedThoughts: ids.map((id) => thought(id, id === "a")),
    anchorId: "a",
    experienceRelations: Object.fromEntries(ids.filter((id) => id !== "a").map((id) =>
      [id, id === "b" ? "differentExperiences" : "cannotTell"])),
  };
}

function marked(answers: Record<string, Classification>, h = handoff()): CandidateState {
  let state = editX(startCandidateAttempt(h, 7), "I think my manager dislikes me because of a diagnosis.");
  for (const [id, answer] of Object.entries(answers)) state = editClassification(state, id, answer);
  return state;
}

function decision(state: CandidateState, requestId = 11) {
  if (state.kind === "closed" || state.kind === "stale") throw new Error("missing active grounding");
  return deriveCandidate(state, requestId);
}

function comparison(h = handoff()): ComparisonDraft {
  return {
    sourceType: "thought", comparisonFrame: h.comparisonFrame,
    selectedRecords: h.selectedThoughts.map((source) => ({ status: "current" as const, source })),
    anchorId: h.anchorId,
    comparisons: Object.fromEntries(h.selectedThoughts.filter((source) => source.id !== h.anchorId).map((source) =>
      [source.id, { comparability: "comparable", meaningRelation: "similar", experienceRelation: h.experienceRelations[source.id] }])),
  };
}

function renderReview(state: CandidateState, locale: "en" | "zh-CN" = "en") {
  return renderToStaticMarkup(createElement(SystemCandidateReview, {
    state, messages: getMessages(locale), onXChange: () => {}, onClassificationChange: () => {},
    onAnalyze: () => {}, onDismiss: () => {}, onReturn: () => {}, onClose: () => {},
  }));
}

describe("Task 017 typed handoff and readiness", () => {
  it("copies the complete reviewed Thought scope in order without generic meaning or conviction", () => {
    const from = comparison();
    const result = handoffFromReviewedComparison(from);
    expect(result).toEqual(handoff());
    expect(JSON.stringify(result)).not.toMatch(/comparability|meaningRelation|subjectiveConviction/);
    expect(handoffFromReviewedComparison({ ...from, sourceType: "observation" })).toBeNull();
    expect(handoffFromReviewedComparison({ ...from, comparisons: { ...from.comparisons, b: { comparability: null, experienceRelation: "differentExperiences" } } })).toBeNull();
  });

  it("requires exact complete classifications, valid anchor and Thought-only scope", () => {
    expect(readyForCandidateAnalysis(marked({ a: "present", b: "present" }))).toBe(false);
    expect(readyForCandidateAnalysis(marked({ a: "present", b: "present", c: "cannotTell" }))).toBe(true);
    expect(readyForCandidateAnalysis(marked({ a: "present", b: "present", c: "cannotTell" }, { ...handoff(), comparisonFrame: "  " }))).toBe(false);
    expect(isValidHandoff({ ...handoff(), anchorId: "outside" })).toBe(false);
    expect(isValidHandoff({ ...handoff(), selectedThoughts: [thought("a"), thought("a")] })).toBe(false);
    expect(isValidHandoff({ ...handoff(), experienceRelations: { b: "differentExperiences" } })).toBe(false);
    expect(isValidHandoff({ ...handoff(), experienceRelations: { b: "differentExperiences", c: "cannotTell", extra: "cannotTell" } })).toBe(false);
    expect(isValidHandoff({ ...handoff(), experienceRelations: { b: "invalid" as never, c: "cannotTell" } })).toBe(false);
    expect(isValidHandoff({ ...handoff(), selectedThoughts: [thought("a"), { ...thought("b"), recordType: "observation" } as never, thought("c")] })).toBe(false);
    const valid = marked({ a: "present", b: "present", c: "cannotTell" });
    if (valid.kind === "closed" || valid.kind === "stale") throw new Error("fixture");
    expect(readyForCandidateAnalysis({ ...valid, classifications: { a: "present", b: "present" } })).toBe(false);
    expect(readyForCandidateAnalysis({ ...valid, classifications: { ...valid.classifications, extra: "present" } })).toBe(false);
    expect(readyForCandidateAnalysis({ ...valid, x: "  " })).toBe(false);
  });
});

describe("Task 017 deterministic Candidate", () => {
  it("returns exactly one attributed structured proposal from the explicit anchor edge", () => {
    const result = decision(marked({ a: "present", b: "present", c: "absent" }));
    expect(result.kind).toBe("proposed");
    if (result.kind !== "proposed") return;
    expect(result.payload).toMatchObject({
      proposer: "system", sourceType: "thought", claimScope: "reviewedThoughtRecordsOnly",
      provenExperienceLowerBound: 2, anchorId: "a", mechanism: { id: "user-grounded-thought-recurrence", version: 1, expressionVersion: 1 },
      semanticAspect: { text: "I think my manager dislikes me because of a diagnosis.", author: "user" },
      comparisonFrame: { text: "  My own frame  ", author: "user" },
      proofEdges: [{ anchorId: "a", thoughtId: "b", relation: "differentExperiences" }],
      absentIds: ["c"], cannotTellIds: [],
    });
    expect(result.payload.analysisScope.map((item) => [item.source.id, item.classification])).toEqual([
      ["a", "present"], ["b", "present"], ["c", "absent"],
    ]);
  });

  it.each([
    [{ a: "absent", b: "present", c: "present" }, "anchorNotPresent"],
    [{ a: "cannotTell", b: "present", c: "present" }, "anchorNotPresent"],
    [{ a: "present", b: "absent", c: "cannotTell" }, "insufficientPresent"],
  ] as const)("returns a semantic zero when grounding lacks a basis", (answers, reason) => {
    expect(decision(marked(answers))).toEqual({ kind: "zeroResult", reason });
  });

  it.each(["sameOrPossiblySame", "cannotTell"] as const)("does not count a %s experience edge", (relation) => {
    const h = { ...handoff(), experienceRelations: { b: relation, c: "cannotTell" as const } };
    expect(decision(marked({ a: "present", b: "present", c: "absent" }, h))).toEqual({ kind: "zeroResult", reason: "noDifferentExperiencePair" });
  });

  it("never infers three experiences from two anchor-relative edges", () => {
    const h = { ...handoff(), experienceRelations: { b: "differentExperiences" as const, c: "differentExperiences" as const } };
    const result = decision(marked({ a: "present", b: "present", c: "present" }, h));
    expect(result.kind).toBe("proposed");
    if (result.kind !== "proposed") return;
    expect(result.payload.proofEdges).toHaveLength(2);
    expect(result.payload.provenExperienceLowerBound).toBe(2);
    expect(result.payload.proofEdges.every((edge) => edge.anchorId === "a")).toBe(true);
    expect(JSON.stringify(result.payload)).not.toContain("threeExperiences");
  });

  it("keeps explicit uncertainty and extra present records without distinct-experience proof visible", () => {
    const uncertain = decision(marked({ a: "present", b: "present", c: "cannotTell" }));
    expect(uncertain.kind === "proposed" && uncertain.payload.cannotTellIds).toEqual(["c"]);
    const extra = decision(marked({ a: "present", b: "present", c: "present" }));
    expect(extra.kind === "proposed" && extra.payload.presentWithoutDistinctExperienceIds).toEqual(["c"]);
    expect(extra.kind === "proposed" && extra.payload.presentExperienceRelations).toHaveLength(2);
    const sameExperience = decision(marked({ a: "present", b: "present", c: "present" }, {
      ...handoff(), experienceRelations: { b: "differentExperiences", c: "sameOrPossiblySame" },
    }));
    expect(sameExperience.kind === "proposed" && sameExperience.payload.presentWithoutDistinctExperienceIds).toEqual(["c"]);
  });

  it("does not convert Task 015 Similar, materially different or not comparable into X markings", () => {
    const initial = comparison();
    const different = answerMeaning(initial, "b", "materiallyDifferent");
    const notComparable = answerComparability(initial, "c", "notComparable");
    const examples = [initial, different, notComparable];
    for (const draft of examples) {
      const from = handoffFromReviewedComparison(draft);
      expect(from?.selectedThoughts.map((source) => source.id)).toEqual(["a", "b", "c"]);
      const attempt = from && startCandidateAttempt(from, 1);
      expect(attempt && attempt.kind === "draft" && attempt.classifications).toEqual({ a: null, b: null, c: null });
    }
  });
});

describe("Task 017 edit, invalidation and source freshness", () => {
  it("clears all answers on any actual X edit without resurrecting them and preserves other answers on one classification edit", () => {
    const initial = marked({ a: "present", b: "present", c: "absent" });
    const same = initial.kind === "closed" || initial.kind === "stale" ? initial : editX(initial, initial.x);
    expect(same).toBe(initial);
    const changed = editX(initial, "new X");
    expect(changed.kind === "draft" && changed.classifications).toEqual({ a: null, b: null, c: null });
    const returned = editX(changed, "I think my manager dislikes me because of a diagnosis.");
    expect(returned.kind === "draft" && returned.classifications).toEqual({ a: null, b: null, c: null });
    const one = editClassification(initial, "c", "cannotTell");
    expect(one.kind === "draft" && one.classifications).toEqual({ a: "present", b: "present", c: "cannotTell" });
    expect(one.kind === "draft" && initial.kind === "draft" && one.revision).toBe(initial.kind === "draft" ? initial.revision + 1 : false);
  });

  it("ignores generic Similar/Comparability edits but invalidates present experience changes", () => {
    const initial = marked({ a: "present", b: "present", c: "absent" });
    const before = comparison();
    const meaningOnly = answerMeaning(before, "b", "materiallyDifferent");
    const comparabilityOnly = answerComparability(before, "b", "notComparable");
    expect(syncComparisonEdit(initial, before, meaningOnly)).toBe(initial);
    expect(syncComparisonEdit(initial, before, comparabilityOnly)).toBe(initial);
    const absentRelation = answerExperience(before, "c", "sameOrPossiblySame");
    const updated = syncComparisonEdit(initial, before, absentRelation);
    expect(updated.kind).toBe("draft");
    if (updated.kind !== "draft" || initial.kind !== "draft") return;
    expect(updated.revision).toBe(initial.revision);
    expect(updated.handoff.experienceRelations.c).toBe("sameOrPossiblySame");
    const presentRelation = answerExperience(before, "b", "cannotTell");
    expect(syncComparisonEdit(initial, before, presentRelation).kind).toBe("draft");
    const proposed = finishChecking(startChecking(initial, "analyze", 1), decision(initial));
    expect(syncComparisonEdit(proposed, before, meaningOnly)).toBe(proposed);
    expect(syncComparisonEdit(proposed, before, comparabilityOnly)).toBe(proposed);
    expect(syncComparisonEdit(proposed, before, absentRelation).kind).toBe("proposed");
    expect(syncComparisonEdit(proposed, before, presentRelation).kind).toBe("draft");
    expect(syncComparisonEdit(proposed, before, { ...before, comparisonFrame: "new frame" })).toEqual({ kind: "closed" });
    expect(syncComparisonEdit(proposed, before, { ...before, anchorId: "b" })).toEqual({ kind: "closed" });
    expect(syncComparisonEdit(proposed, before, { ...before, selectedRecords: before.selectedRecords.slice(0, 2) })).toEqual({ kind: "closed" });
  });

  it("invalidates the full frozen scope for any selected source or linked situation change, including absent/unknown", () => {
    const h = handoff();
    expect(changedScopeSources(h, h.selectedThoughts)).toBeNull();
    for (const id of ["a", "b", "c"]) {
      const modified = h.selectedThoughts.map((source) => source.id === id ? { ...source, stateToken: "new-token" } : source);
      expect(changedScopeSources(h, modified)?.affectedIds).toContain(id);
      const correctedText = h.selectedThoughts.map((source) => source.id === id ? { ...source, content: "changed without token" } : source);
      expect(changedScopeSources(h, correctedText)?.affectedIds).toContain(id);
    }
    expect(changedScopeSources(h, h.selectedThoughts.filter((source) => source.id !== "c"))).toEqual({ reason: "sourceNotFound", affectedIds: ["c"] });
    const contextChanged = h.selectedThoughts.map((source) => source.id === "a" ? { ...source, linkedSituation: { ...source.linkedSituation!, description: "corrected context" } } : source);
    expect(changedScopeSources(h, contextChanged)?.affectedIds).toEqual(["a"]);
    const relinked = h.selectedThoughts.map((source) => source.id === "a" ? { ...source, linkedSituation: null } : source);
    expect(changedScopeSources(h, relinked)?.affectedIds).toEqual(["a"]);
    expect(changedScopeSources(h, [...h.selectedThoughts, thought("unselected")])).toBeNull();
    expect(JSON.stringify(staleCandidate("sourceNotFound", ["c"]))).not.toMatch(/Thought c|Current situation|manager/);
  });
});

describe("Task 017 request and review lifecycle", () => {
  it("renders user text inert and separately attributed in both languages", () => {
    const dangerous = '<img src=x onerror="alert(1)"> My manager caused my illness.';
    let state = editX(startCandidateAttempt({ ...handoff(), comparisonFrame: "<script>my frame</script>" }, 4), dangerous);
    for (const [id, answer] of Object.entries({ a: "present", b: "present", c: "cannotTell" }) as [string, Classification][]) {
      state = editClassification(state, id, answer);
    }
    const proposed = finishChecking(startChecking(state, "analyze", 8), decision(state, 8));
    for (const locale of ["en", "zh-CN"] as const) {
      const html = renderReview(proposed, locale);
      const copy = getMessages(locale).systemCandidate;
      expect(html).toContain(copy.xLabel);
      expect(html).toContain(copy.frameLabel);
      expect(html).toContain(copy.proposalTitle);
      expect(html).toContain(copy.conclusion);
      expect(html).toContain("&lt;script&gt;my frame&lt;/script&gt;");
      expect(html).toContain("&lt;img src=x onerror=&quot;alert(1)&quot;&gt;");
      expect(html).not.toContain("<script>");
      expect(html).not.toContain("<img src=x");
      expect(html).toContain(copy.cannotTellSummary.replace("{count}", "1"));
    }
    expect(renderReview(startChecking(proposed, "focus", 9))).not.toContain(getMessages("en").systemCandidate.conclusion);
    expect(renderReview(dismissCandidate(proposed))).not.toContain(getMessages("en").systemCandidate.conclusion);
  });

  it("separates zero, load error, dismissal and rechecking without durable rejection", () => {
    const initial = marked({ a: "present", b: "present", c: "absent" });
    const checking = startChecking(initial, "analyze", 21);
    const proposed = finishChecking(checking, decision(initial, 21));
    expect(proposed.kind).toBe("proposed");
    expect(startChecking(proposed, "focus", 22).kind).toBe("checking");
    expect(restoreCheckedResult(startChecking(proposed, "focus", 22))).toEqual(proposed);
    const dismissed = dismissCandidate(proposed);
    expect(dismissed.kind).toBe("dismissed");
    expect(readyForCandidateAnalysis(dismissed)).toBe(true);
    expect(returnToGrounding(dismissed).kind).toBe("draft");
    expect(failCandidateLoad(checking, "storageUnavailable").kind).toBe("loadError");
    const zero = finishChecking(checking, { kind: "zeroResult", reason: "insufficientPresent" });
    expect(zero.kind).toBe("zeroResult");
  });

  it("rejects late analysis/focus publication after edit, clear, dismissal, newer request or unmount", async () => {
    const gate = createComparisonRefreshGate();
    let state = startChecking(marked({ a: "present", b: "present", c: "absent" }), "analyze", gate.start());
    if (state.kind !== "checking") throw new Error("fixture");
    const original = { request: state.requestId, attempt: state.attemptId, revision: state.revision };
    expect(candidateRequestIsCurrent(state, original.request, original.attempt, original.revision)).toBe(true);
    state = editX(state, "new X");
    expect(candidateRequestIsCurrent(state, original.request, original.attempt, original.revision)).toBe(false);
    state = editClassification(startChecking(marked({ a: "present", b: "present", c: "absent" }), "analyze", original.request), "c", "cannotTell");
    expect(candidateRequestIsCurrent(state, original.request, original.attempt, original.revision)).toBe(false);
    state = startChecking(marked({ a: "present", b: "present", c: "absent" }), "focus", gate.start());
    expect(gate.isCurrent(original.request)).toBe(false);
    expect(candidateRequestIsCurrent(state, original.request, original.attempt, original.revision)).toBe(false);
    state = { kind: "closed" };
    expect(candidateRequestIsCurrent(state, original.request, original.attempt, original.revision)).toBe(false);
    const proposed = finishChecking(startChecking(marked({ a: "present", b: "present", c: "absent" }), "analyze", 44), decision(marked({ a: "present", b: "present", c: "absent" }), 44));
    expect(candidateRequestIsCurrent(dismissCandidate(proposed), 44, 7, 4)).toBe(false);
    gate.dispose();
    expect(gate.isCurrent(2)).toBe(false);
    await Promise.resolve();
  });

  it("keeps the UI local, explicitly triggered and separate from Task 013", () => {
    const view = read("src/components/SystemCandidateReview.tsx");
    const parent = read("src/components/SemanticComparison.tsx");
    expect(parent).toContain('refresh("candidateAnalyze")');
    expect(parent).toContain("handoffFromReviewedComparison(draftRef.current)");
    expect(parent).toContain("changedScopeSources(currentCandidate.handoff, currentSources)");
    expect(view).toContain("readyForCandidateAnalysis(state)");
    expect(view).toContain("copy.conclusion");
    expect(view).not.toMatch(/dangerouslySetInnerHTML|localStorage|sessionStorage|indexedDB|invoke\(|candidate_review|supporting|contradicting/);
    expect(parent).not.toMatch(/localStorage|sessionStorage|indexedDB|invoke\(/);
  });
});
