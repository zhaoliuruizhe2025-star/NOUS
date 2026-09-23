/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import type { StructuredHistory } from "../src/app/history";
import {
  addSource,
  answerComparability,
  answerExperience,
  answerMeaning,
  changeComparisonFrame,
  changeSourceType,
  chooseAnchor,
  createComparisonRefreshGate,
  emptyComparison,
  mayRestoreReviewedSummary,
  projectComparisonSources,
  readyForComparison,
  reconcileComparison,
  removeSource,
  type ComparisonDraft,
  type ComparisonSource,
} from "../src/app/semantic_comparison";

const repositoryRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const readSource = (path: string) => readFileSync(join(repositoryRoot, path), "utf8");

function history(): StructuredHistory {
  return {
    contexts: [{
      id: "s1",
      description: "Current context",
      stateToken: "s1-token",
      corrected: true,
      corrections: [{ sequence: 1, beforeDescription: "Wrong old context", afterDescription: "Current context", note: null }],
      observations: [{
        id: "o1", content: "Current observed detail", stateToken: "o1-token", corrected: true,
        corrections: [{ sequence: 1, beforeContent: "Wrong old observation", afterContent: "Current observed detail", beforeContext: null, afterContext: { id: "s1", description: "Current context" }, note: null }],
      }],
      thoughts: [{
        id: "t1", content: "Current thought A", stateToken: "t1-token", subjectiveConviction: 87, corrected: true,
        corrections: [{ sequence: 1, beforeContent: "Wrong old thought", afterContent: "Current thought A", beforeContext: null, afterContext: { id: "s1", description: "Current context" }, beforeSubjectiveConviction: null, afterSubjectiveConviction: 87, note: null }],
      }],
    }],
    standaloneObservations: [
      { id: "o2", content: "Standalone observed detail", stateToken: "o2-token", corrected: false, corrections: [] },
    ],
    standaloneThoughts: [
      { id: "t2", content: "Current thought B", stateToken: "t2-token", subjectiveConviction: null, corrected: false, corrections: [] },
      { id: "t3", content: "Current thought C", stateToken: "t3-token", subjectiveConviction: null, corrected: false, corrections: [] },
    ],
  };
}

function source(sources: ComparisonSource[], id: string): ComparisonSource {
  const found = sources.find((item) => item.id === id);
  if (!found) throw new Error(`missing fixture source ${id}`);
  return found;
}

function draftWith(...ids: string[]): ComparisonDraft {
  const sources = projectComparisonSources(history());
  let draft = changeSourceType(emptyComparison(), ids[0]?.startsWith("o") ? "observation" : "thought");
  draft = changeComparisonFrame(draft, "  my exact comparison angle  ");
  for (const id of ids) draft = addSource(draft, source(sources, id));
  return draft;
}

function answered(draft: ComparisonDraft, id: string): ComparisonDraft {
  return answerExperience(answerMeaning(answerComparability(draft, id, "comparable"), id, "similar"), id, "differentExperiences");
}

describe("Task 015 current source boundary", () => {
  it("projects only current Thought and Observation values with linked context and no old correction content", () => {
    const sources = projectComparisonSources(history());
    expect(sources.map((item) => [item.recordType, item.id])).toEqual([
      ["observation", "o1"], ["thought", "t1"], ["observation", "o2"], ["thought", "t2"], ["thought", "t3"],
    ]);
    expect(source(sources, "t1")).toEqual({ recordType: "thought", id: "t1", content: "Current thought A", stateToken: "t1-token", linkedSituation: { id: "s1", description: "Current context", stateToken: "s1-token" } });
    expect(source(sources, "o2").linkedSituation).toBeNull();
    expect(JSON.stringify(sources)).not.toMatch(/Wrong old|corrections|subjectiveConviction|createdAt|savedAt/);
  });

  it("keeps modes separate and requires an explicit anchor and two current records", () => {
    const sources = projectComparisonSources(history());
    let draft = draftWith("t1");
    expect(readyForComparison(draft)).toBe(false);
    expect(addSource(draft, source(sources, "o1"))).toEqual(draft);
    draft = addSource(draft, source(sources, "t2"));
    expect(draft.anchorId).toBeNull();
    expect(readyForComparison(draft)).toBe(false);
    draft = chooseAnchor(draft, "t1");
    expect(readyForComparison(draft)).toBe(false);
    expect(chooseAnchor(draft, "o1")).toEqual(draft);
    expect(readyForComparison(answered(draft, "t2"))).toBe(true);
    expect(readyForComparison(changeComparisonFrame(answered(draft, "t2"), "   "))).toBe(false);
  });
});

describe("Task 015 user answer semantics", () => {
  it("distinguishes unanswered from explicit Cannot tell and asks experience independently", () => {
    let draft = chooseAnchor(draftWith("t1", "t2"), "t1");
    expect(draft.comparisons.t2).toEqual({ comparability: null, experienceRelation: null });
    draft = answerComparability(draft, "t2", "cannotTell");
    expect(readyForComparison(draft)).toBe(false);
    draft = answerExperience(draft, "t2", "cannotTell");
    expect(draft.comparisons.t2).toEqual({ comparability: "cannotTell", experienceRelation: "cannotTell" });
    expect(readyForComparison(draft)).toBe(true);
    expect(answerMeaning(draft, "t2", "similar")).toEqual(draft);
  });

  it("keeps Meaning Relation only when comparable and clears it when comparability changes", () => {
    let draft = chooseAnchor(draftWith("t1", "t2"), "t1");
    draft = answerExperience(draft, "t2", "sameOrPossiblySame");
    draft = answerComparability(draft, "t2", "comparable");
    expect(readyForComparison(draft)).toBe(false);
    draft = answerMeaning(draft, "t2", "materiallyDifferent");
    expect(readyForComparison(draft)).toBe(true);
    draft = answerComparability(draft, "t2", "notComparable");
    expect(draft.comparisons.t2).toEqual({ comparability: "notComparable", experienceRelation: "sameOrPossiblySame" });
    expect(readyForComparison(draft)).toBe(true);
    draft = answerComparability(draft, "t2", "comparable");
    expect(draft.comparisons.t2).toEqual({ comparability: "comparable", meaningRelation: null, experienceRelation: "sameOrPossiblySame" });
    expect(readyForComparison(draft)).toBe(false);
  });

  it("accepts all explicitly unknown, all not comparable, and materially different outcomes", () => {
    let draft = chooseAnchor(draftWith("t1", "t2", "t3"), "t1");
    for (const id of ["t2", "t3"]) draft = answerExperience(answerComparability(draft, id, "cannotTell"), id, "cannotTell");
    expect(readyForComparison(draft)).toBe(true);
    for (const id of ["t2", "t3"]) draft = answerComparability(draft, id, "notComparable");
    expect(readyForComparison(draft)).toBe(true);
    for (const id of ["t2", "t3"]) draft = answerMeaning(answerComparability(draft, id, "comparable"), id, "materiallyDifferent");
    expect(readyForComparison(draft)).toBe(true);
  });
});

describe("Task 015 deterministic edits", () => {
  it("preserves exact frame text, clears answers only on its first actual edit, and does not restore them", () => {
    let draft = chooseAnchor(draftWith("t1", "t2"), "t1");
    draft = answered(draft, "t2");
    draft = changeComparisonFrame(draft, "new angle ");
    expect(draft.comparisonFrame).toBe("new angle ");
    expect(draft.selectedRecords).toHaveLength(2);
    expect(draft.anchorId).toBe("t1");
    expect(draft.comparisons.t2).toEqual({ comparability: null, experienceRelation: null });
    draft = changeComparisonFrame(draft, "  my exact comparison angle  ");
    expect(draft.comparisons.t2).toEqual({ comparability: null, experienceRelation: null });
  });

  it("clears all answers on anchor change/removal and only the affected answer on non-anchor removal", () => {
    let draft = chooseAnchor(draftWith("t1", "t2", "t3"), "t1");
    draft = answered(answered(draft, "t2"), "t3");
    const removed = removeSource(draft, "t2");
    expect(removed.comparisons.t2).toBeUndefined();
    expect(removed.comparisons.t3).toEqual(draft.comparisons.t3);
    const changed = chooseAnchor(draft, "t2");
    expect(changed.comparisons.t1).toEqual({ comparability: null, experienceRelation: null });
    expect(changed.comparisons.t3).toEqual({ comparability: null, experienceRelation: null });
    expect(removeSource(changed, "t2")).toMatchObject({ anchorId: null, comparisons: {} });
  });

  it("preserves unrelated answers when adding a source and clears all on mode switch while keeping the frame", () => {
    const sources = projectComparisonSources(history());
    let draft = answered(chooseAnchor(draftWith("t1", "t2"), "t1"), "t2");
    draft = addSource(draft, source(sources, "t3"));
    expect(draft.comparisons.t2.comparability).toBe("comparable");
    expect(draft.comparisons.t3).toEqual({ comparability: null, experienceRelation: null });
    draft = changeSourceType(draft, "observation");
    expect(draft).toMatchObject({ sourceType: "observation", comparisonFrame: "  my exact comparison angle  ", selectedRecords: [], anchorId: null, comparisons: {} });
  });
});

describe("Task 015 source invalidation", () => {
  it("removes only a corrected non-anchor answer and its old content, without reviving it automatically", () => {
    const before = answered(answered(chooseAnchor(draftWith("t1", "t2", "t3"), "t1"), "t2"), "t3");
    const after = history();
    after.standaloneThoughts[0].content = "Corrected thought B";
    after.standaloneThoughts[0].stateToken = "t2-token-2";
    const result = reconcileComparison(before, projectComparisonSources(after));
    expect(result.changed).toBe(true);
    expect(result.draft.comparisons.t2).toBeUndefined();
    expect(result.draft.comparisons.t3).toEqual(before.comparisons.t3);
    expect(JSON.stringify(result.draft)).not.toContain("Current thought B");
    expect(result.draft.selectedRecords.find((item) => item.status === "stale")).toEqual({ status: "stale", recordType: "thought", id: "t2", reason: "sourceChanged" });
    expect(reconcileComparison(result.draft, projectComparisonSources(after)).draft.selectedRecords.some((item) => item.status === "stale")).toBe(true);
    const reselected = addSource(result.draft, source(projectComparisonSources(after), "t2"));
    expect(reselected.comparisons.t2).toEqual({ comparability: null, experienceRelation: null });
  });

  it("invalidates linked context, including an anchor context or changed linkage", () => {
    const before = answered(chooseAnchor(draftWith("o1", "o2"), "o1"), "o2");
    const after = history();
    after.contexts[0].description = "Corrected context";
    after.contexts[0].stateToken = "s1-token-2";
    const result = reconcileComparison(before, projectComparisonSources(after));
    expect(result.draft.anchorId).toBeNull();
    expect(result.draft.comparisons).toEqual({});
    expect(JSON.stringify(result.draft)).not.toContain("Current context");

    const linkedNonAnchor = answered(chooseAnchor(draftWith("o1", "o2"), "o2"), "o1");
    const nonAnchorResult = reconcileComparison(linkedNonAnchor, projectComparisonSources(after));
    expect(nonAnchorResult.draft.anchorId).toBe("o2");
    expect(nonAnchorResult.draft.comparisons.o1).toBeUndefined();

    const moved = history();
    const observation = moved.contexts[0].observations.shift();
    if (!observation) throw new Error("missing fixture observation");
    moved.standaloneObservations.push(observation);
    expect(reconcileComparison(linkedNonAnchor, projectComparisonSources(moved)).draft.comparisons.o1).toBeUndefined();
  });

  it("removes deleted non-anchor content, and clears the anchor and all answers if it is deleted", () => {
    const before = answered(answered(chooseAnchor(draftWith("t1", "t2", "t3"), "t1"), "t2"), "t3");
    const withoutT2 = history();
    withoutT2.standaloneThoughts.shift();
    const nonAnchor = reconcileComparison(before, projectComparisonSources(withoutT2));
    expect(nonAnchor.draft.selectedRecords.map((item) => item.status === "current" ? item.source.id : item.id)).toEqual(["t1", "t3"]);
    expect(nonAnchor.draft.comparisons.t2).toBeUndefined();
    expect(nonAnchor.draft.comparisons.t3).toEqual(before.comparisons.t3);
    expect(JSON.stringify(nonAnchor.draft)).not.toContain("Current thought B");

    const withoutAnchor = history();
    withoutAnchor.contexts[0].thoughts = [];
    const anchor = reconcileComparison(before, projectComparisonSources(withoutAnchor));
    expect(anchor.draft.anchorId).toBeNull();
    expect(anchor.draft.comparisons).toEqual({});
    expect(JSON.stringify(anchor.draft)).not.toContain("Current thought A");
  });
});

describe("Task 015 async, summary and lifecycle boundary", () => {
  it("does not restore Reviewed after a user edit during the read or an invalidated source", () => {
    const draft = answered(chooseAnchor(draftWith("t1", "t2"), "t1"), "t2");
    expect(mayRestoreReviewedSummary(draft, true, false, 4, 4)).toBe(true);
    expect(mayRestoreReviewedSummary(draft, true, false, 4, 5)).toBe(false);
    expect(mayRestoreReviewedSummary(draft, true, true, 4, 4)).toBe(false);
    expect(mayRestoreReviewedSummary(draft, false, false, 4, 4)).toBe(false);
  });

  it("rejects an older result after a newer correction/deletion and all responses after unmount", async () => {
    const gate = createComparisonRefreshGate();
    const published: string[] = [];
    let finishOld!: (value: string) => void;
    const oldRead = new Promise<string>((resolve) => { finishOld = resolve; });
    const first = gate.start();
    const oldResult = oldRead.then((value) => { if (gate.isCurrent(first)) published.push(value); });
    const second = gate.start();
    const newResult = Promise.resolve("sourceChanged").then((value) => { if (gate.isCurrent(second)) published.push(value); });
    await newResult;
    finishOld("old reviewed summary");
    await oldResult;
    expect(published).toEqual(["sourceChanged"]);
    expect(gate.isCurrent(first)).toBe(false);
    expect(gate.isCurrent(second)).toBe(true);

    const third = gate.start();
    const deleted = Promise.resolve("sourceNotFound").then((value) => { if (gate.isCurrent(third)) published.push(value); });
    await deleted;
    expect(published[published.length - 1]).toBe("sourceNotFound");
    gate.dispose();
    expect(gate.isCurrent(third)).toBe(false);
  });

  it("has only anchor-relative user judgments and the existing read command in the view", () => {
    const component = readSource("src/components/SemanticComparison.tsx");
    const helper = readSource("src/app/semantic_comparison.ts");
    const app = readSource("src/app/App.tsx");
    expect(component).toContain("loadStructuredHistory()");
    expect(component).toContain('window.addEventListener("focus", onFocus)');
    expect(component).toContain('window.removeEventListener("focus", onFocus)');
    expect(component).toContain("gateRef.current.dispose()");
    expect(component).toContain("mayRestoreReviewedSummary(");
    expect(component).toContain("reviewStatus === \"reviewed\"");
    expect(component).toContain("draft.comparisons[source.id]");
    expect(component).toContain("copy.limitations");
    expect(component).toContain("source.linkedSituation.description");
    expect(component).toContain("copy.noLinkedSituation");
    expect(component).not.toMatch(/localStorage|sessionStorage|indexedDB|invoke\(|subjectiveConviction/);
    expect(helper).not.toMatch(/localStorage|sessionStorage|indexedDB|invoke\(|subjectiveConviction/);
    expect(app).toContain('<SemanticComparison messages={messages} onClose={() => setView("history")} />');
    expect(app).not.toContain("semanticComparisonState");
  });

  it("starts empty again after Clear or remount and never stores user judgments in history", () => {
    const draft = answered(chooseAnchor(draftWith("t1", "t2"), "t1"), "t2");
    expect(readyForComparison(draft)).toBe(true);
    expect(emptyComparison()).toEqual({ sourceType: null, comparisonFrame: "", selectedRecords: [], anchorId: null, comparisons: {} });
    expect(readSource("src/app/history.ts")).toContain('invoke<StructuredHistory>("load_structured_history")');
  });
});
