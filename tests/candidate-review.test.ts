/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import type { StructuredHistory } from "../src/app/history";
import {
  allCandidateSources,
  assignCandidateRole,
  canReviewCandidate,
  compareCandidateSources,
  createRefreshGate,
  findCandidateSource,
  projectCandidateSources,
  toggleCandidateSource,
  type CandidateSource,
  type CandidateSourceSnapshot,
  type SelectedCandidateSource,
} from "../src/app/candidate_review";

const repositoryRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const readSource = (path: string) => readFileSync(join(repositoryRoot, path), "utf8");

function history(): StructuredHistory {
  return {
    contexts: [{
      id: "s1",
      description: "Current situation",
      stateToken: "s-token-1",
      corrected: true,
      corrections: [{
        sequence: 1,
        beforeDescription: "Prior inaccurate situation",
        afterDescription: "Current situation",
        note: null,
      }],
      observations: [{
        id: "o1",
        content: "Current observation",
        stateToken: "o-token-1",
        corrected: true,
        corrections: [{
          sequence: 1,
          beforeContent: "Prior inaccurate observation",
          afterContent: "Current observation",
          beforeContext: null,
          afterContext: { id: "s1", description: "Current situation" },
          note: null,
        }],
      }],
      thoughts: [{
        id: "t1",
        content: "Current thought",
        subjectiveConviction: 42,
        stateToken: "t-token-1",
        corrected: true,
        corrections: [{
          sequence: 1,
          beforeContent: "Prior inaccurate thought",
          afterContent: "Current thought",
          beforeContext: null,
          afterContext: { id: "s1", description: "Current situation" },
          beforeSubjectiveConviction: null,
          afterSubjectiveConviction: 42,
          note: null,
        }],
      }],
    }],
    standaloneObservations: [{ id: "o2", content: "Standalone observation", stateToken: "o-token-2", corrected: false, corrections: [] }],
    standaloneThoughts: [{ id: "t2", content: "Standalone thought", subjectiveConviction: null, stateToken: "t-token-2", corrected: false, corrections: [] }],
  };
}

function selected(source: CandidateSource): SelectedCandidateSource {
  return { status: "current", source, role: "supporting" };
}

function source(snapshot: CandidateSourceSnapshot, type: CandidateSource["recordType"], id: string): CandidateSource {
  const found = findCandidateSource(snapshot, type, id);
  if (!found) throw new Error("fixture source missing");
  return found;
}

describe("transient Candidate source projection", () => {
  it("projects only current S/O/T values, stable IDs, tokens and linked context", () => {
    const projected = projectCandidateSources(history());
    expect(allCandidateSources(projected).map((item) => [item.recordType, item.id])).toEqual([
      ["situation", "s1"], ["observation", "o1"], ["thought", "t1"],
      ["observation", "o2"], ["thought", "t2"],
    ]);
    expect(source(projected, "situation", "s1")).toMatchObject({ content: "Current situation", stateToken: "s-token-1", linkedSituation: null });
    expect(source(projected, "observation", "o1")).toMatchObject({ content: "Current observation", stateToken: "o-token-1", linkedSituation: { id: "s1", stateToken: "s-token-1", description: "Current situation" } });
    expect(source(projected, "thought", "t1")).toMatchObject({ content: "Current thought", stateToken: "t-token-1", subjectiveConviction: 42 });
    expect(projected.standaloneObservations[0].linkedSituation).toBeNull();
    expect(projected.standaloneThoughts[0].linkedSituation).toBeNull();
    expect(JSON.stringify(projected)).not.toMatch(/Prior inaccurate|corrections|createdAt|savedAt/);
  });

  it("allows incomplete drafts but gates summary on wording, scope and current selection", () => {
    const item = selected(source(projectCandidateSources(history()), "observation", "o1"));
    expect(canReviewCandidate("", "scope", [item])).toBe(false);
    expect(canReviewCandidate("candidate", "  ", [item])).toBe(false);
    expect(canReviewCandidate("candidate", "scope", [])).toBe(false);
    expect(canReviewCandidate(" candidate ", " scope ", [{ status: "current", source: source(projectCandidateSources(history()), "observation", "o1"), role: null }])).toBe(true);
    expect(canReviewCandidate("candidate", "scope", [{ status: "stale", recordType: "observation", id: "o1", reason: "sourceChanged" }])).toBe(false);
  });

  it("supports one optional role and explicit re-review without carrying an old role", () => {
    const current = source(projectCandidateSources(history()), "thought", "t1");
    const first = toggleCandidateSource([], current);
    expect(first).toEqual([{ status: "current", source: current, role: null }]);
    const classified = assignCandidateRole(first, current, "contextualizing");
    expect(classified[0]).toMatchObject({ status: "current", role: "contextualizing" });
    expect(assignCandidateRole(classified, current, "contradicting")[0]).toMatchObject({ status: "current", role: "contradicting" });
    expect(toggleCandidateSource(classified, current)).toEqual([]);
    const stale: SelectedCandidateSource[] = [{ status: "stale", recordType: "thought", id: "t1", reason: "sourceChanged" }];
    expect(toggleCandidateSource(stale, current)).toEqual([{ status: "current", source: current, role: null }]);
  });
});

describe("Candidate representation invalidation", () => {
  it("invalidates a corrected source, dropping previous content and role", () => {
    const before = projectCandidateSources(history());
    const afterHistory = history();
    afterHistory.contexts[0].observations[0].content = "Corrected again";
    afterHistory.contexts[0].observations[0].stateToken = "o-token-2";
    const result = compareCandidateSources([selected(source(before, "observation", "o1"))], projectCandidateSources(afterHistory));
    expect(result).toEqual({ stale: true, selectedSources: [{ status: "stale", recordType: "observation", id: "o1", reason: "sourceChanged" }] });
    expect(JSON.stringify(result)).not.toContain("Current observation");
    expect(JSON.stringify(result)).not.toContain("supporting");
    expect(source(projectCandidateSources(afterHistory), "observation", "o1").content).toBe("Corrected again");
  });

  it("invalidates a linked child when only its Situation context changes", () => {
    const before = projectCandidateSources(history());
    const afterHistory = history();
    afterHistory.contexts[0].description = "Corrected context";
    afterHistory.contexts[0].stateToken = "s-token-2";
    const result = compareCandidateSources([
      selected(source(before, "observation", "o1")),
      selected(source(before, "thought", "t1")),
    ], projectCandidateSources(afterHistory));
    expect(result.selectedSources).toEqual([
      { status: "stale", recordType: "observation", id: "o1", reason: "sourceChanged" },
      { status: "stale", recordType: "thought", id: "t1", reason: "sourceChanged" },
    ]);
  });

  it("invalidates a changed linkage even if a child token is unchanged", () => {
    const before = projectCandidateSources(history());
    const afterHistory = history();
    const moved = afterHistory.contexts[0].observations.shift();
    if (!moved) throw new Error("fixture source missing");
    afterHistory.standaloneObservations.push(moved);
    expect(compareCandidateSources([selected(source(before, "observation", "o1"))], projectCandidateSources(afterHistory)).selectedSources[0]).toMatchObject({ status: "stale", reason: "sourceChanged" });
  });

  it("invalidates a deleted source without retaining its content", () => {
    const before = projectCandidateSources(history());
    const afterHistory = history();
    afterHistory.standaloneThoughts = [];
    const result = compareCandidateSources([selected(source(before, "thought", "t2"))], projectCandidateSources(afterHistory));
    expect(result).toEqual({ stale: true, selectedSources: [{ status: "stale", recordType: "thought", id: "t2", reason: "sourceNotFound" }] });
    expect(JSON.stringify(result)).not.toMatch(/Standalone thought|supporting/);
  });

  it("keeps valid mixed roles and never automatically revives a stale item", () => {
    const snapshot = projectCandidateSources(history());
    const items: SelectedCandidateSource[] = [
      selected(source(snapshot, "situation", "s1")),
      { status: "current", source: source(snapshot, "observation", "o1"), role: "contradicting" },
      { status: "current", source: source(snapshot, "thought", "t1"), role: "complicating" },
      { status: "current", source: source(snapshot, "observation", "o2"), role: "contextualizing" },
      { status: "current", source: source(snapshot, "thought", "t2"), role: null },
    ];
    expect(compareCandidateSources(items, snapshot)).toEqual({ selectedSources: items, stale: false });
    const stale: SelectedCandidateSource[] = [{ status: "stale", recordType: "thought", id: "t1", reason: "sourceChanged" }];
    expect(compareCandidateSources(stale, snapshot)).toEqual({ selectedSources: stale, stale: true });
  });
});

describe("bounded asynchronous and UI boundaries", () => {
  it("rejects an older response after a newer changed or deleted result and after unmount", async () => {
    const gate = createRefreshGate();
    const published: string[] = [];
    let finishOlder!: (value: string) => void;
    let finishNewer!: (value: string) => void;
    const older = new Promise<string>((resolve) => { finishOlder = resolve; });
    const newer = new Promise<string>((resolve) => { finishNewer = resolve; });
    const a = gate.start();
    const oldResult = older.then((value) => { if (gate.isCurrent(a)) published.push(value); });
    const b = gate.start();
    const newResult = newer.then((value) => { if (gate.isCurrent(b)) published.push(value); });
    finishNewer("sourceChanged");
    finishOlder("old summary");
    await Promise.all([oldResult, newResult]);
    expect(published).toEqual(["sourceChanged"]);

    let finishOldAgain!: (value: string) => void;
    const oldAgain = new Promise<string>((resolve) => { finishOldAgain = resolve; });
    const c = gate.start();
    const oldAgainResult = oldAgain.then((value) => { if (gate.isCurrent(c)) published.push(value); });
    const d = gate.start();
    const deleted = Promise.resolve("sourceNotFound").then((value) => { if (gate.isCurrent(d)) published.push(value); });
    await deleted;
    finishOldAgain("old summary");
    await oldAgainResult;
    expect(published[published.length - 1]).toBe("sourceNotFound");
    expect(published).not.toContain("old summary");
    gate.dispose();
    expect(gate.isCurrent(d)).toBe(false);
    const newMount = createRefreshGate();
    expect(newMount.isCurrent(newMount.start())).toBe(true);
    expect(gate.isCurrent(d)).toBe(false);
  });

  it("uses only the existing read command and component memory, with focus cleanup", () => {
    const component = readSource("src/components/CandidateReview.tsx");
    const helper = readSource("src/app/candidate_review.ts");
    const app = readSource("src/app/App.tsx");
    expect(component).toContain("loadStructuredHistory()");
    expect(component).toContain('window.addEventListener("focus", onFocus)');
    expect(component).toContain('window.removeEventListener("focus", onFocus)');
    expect(component).toContain("gateRef.current.dispose()");
    expect(component).toContain('refresh("review")');
    expect(component).not.toMatch(/localStorage|sessionStorage|indexedDB|invoke\(/);
    expect(helper).not.toMatch(/localStorage|sessionStorage|indexedDB|invoke\(/);
    expect(app).toContain('<CandidateReview messages={messages} onClose={() => setView("history")} />');
    expect(app).toContain('view === "portability" ?');
  });
});
