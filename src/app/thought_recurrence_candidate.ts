import type { ComparisonDraft, ComparisonSource, ExperienceRelation } from "./semantic_comparison";
import { readyForComparison } from "./semantic_comparison";

export const CANDIDATE_MECHANISM = {
  id: "user-grounded-thought-recurrence",
  version: 1,
  expressionVersion: 1,
} as const;

export type Classification = "present" | "absent" | "cannotTell";
export type ClassificationAnswer = Classification | null;
export type ThoughtSource = ComparisonSource & { recordType: "thought" };
export type HistoryLoadError = "subjectInvariant" | "dataInconsistent" | "storageUnavailable" | "loadFailed";
export type ZeroReason = "anchorNotPresent" | "insufficientPresent" | "noDifferentExperiencePair";

export interface CandidateHandoff {
  sourceType: "thought";
  comparisonFrame: string;
  selectedThoughts: readonly ThoughtSource[];
  anchorId: string;
  experienceRelations: Readonly<Record<string, ExperienceRelation>>;
}

export interface CandidateGrounding {
  attemptId: number;
  revision: number;
  handoff: CandidateHandoff;
  x: string;
  classifications: Readonly<Record<string, ClassificationAnswer>>;
}

export interface CandidateScopeItem {
  source: ThoughtSource;
  classification: Classification;
}

export interface PresentExperienceRelation {
  anchorId: string;
  thoughtId: string;
  relation: ExperienceRelation;
}

export interface ThoughtRecurrenceCandidate {
  proposer: "system";
  mechanism: typeof CANDIDATE_MECHANISM;
  claimKind: "userGroundedThoughtAcrossDifferentExperiences";
  claimScope: "reviewedThoughtRecordsOnly";
  provenExperienceLowerBound: 2;
  generation: { attemptId: number; dependencyRevision: number; requestId: number };
  semanticAspect: { text: string; author: "user" };
  comparisonFrame: { text: string; author: "user" };
  sourceType: "thought";
  anchorId: string;
  analysisScope: readonly CandidateScopeItem[];
  presentExperienceRelations: readonly PresentExperienceRelation[];
  proofEdges: readonly PresentExperienceRelation[];
  absentIds: readonly string[];
  cannotTellIds: readonly string[];
  presentWithoutDistinctExperienceIds: readonly string[];
  limitations: readonly [
    "selectedRecordsOnly",
    "userMarkedContentNotVerifiedTruth",
    "differentExperiencesNotIndependentEvidence",
    "noSemanticEventTime",
  ];
}

type ActiveState = CandidateGrounding & (
  | { kind: "draft" }
  | { kind: "checking"; purpose: "analyze" | "focus"; requestId: number; previous?: { kind: "zeroResult"; reason: ZeroReason } | { kind: "proposed"; payload: ThoughtRecurrenceCandidate } }
  | { kind: "zeroResult"; reason: ZeroReason }
  | { kind: "proposed"; payload: ThoughtRecurrenceCandidate }
  | { kind: "dismissed" }
  | { kind: "loadError"; code: HistoryLoadError }
);

export type CandidateState =
  | { kind: "closed" }
  | { kind: "stale"; reason: "sourceChanged" | "sourceNotFound"; affectedIds: readonly string[] }
  | ActiveState;

export function handoffFromReviewedComparison(draft: ComparisonDraft): CandidateHandoff | null {
  if (draft.sourceType !== "thought" || !readyForComparison(draft) || !draft.anchorId) return null;
  const selectedThoughts: ThoughtSource[] = [];
  const experienceRelations: Record<string, ExperienceRelation> = {};
  for (const selection of draft.selectedRecords) {
    if (selection.status !== "current" || selection.source.recordType !== "thought") return null;
    const source = selection.source;
    selectedThoughts.push({
      recordType: "thought",
      id: source.id,
      content: source.content,
      stateToken: source.stateToken,
      linkedSituation: source.linkedSituation ? { ...source.linkedSituation } : null,
    });
    if (source.id !== draft.anchorId) {
      const relation = draft.comparisons[source.id]?.experienceRelation;
      if (!relation) return null;
      experienceRelations[source.id] = relation;
    }
  }
  return {
    sourceType: "thought",
    comparisonFrame: draft.comparisonFrame,
    selectedThoughts,
    anchorId: draft.anchorId,
    experienceRelations,
  };
}

export function startCandidateAttempt(handoff: CandidateHandoff, attemptId: number): CandidateState {
  return {
    kind: "draft", attemptId, revision: 0, handoff, x: "",
    classifications: Object.fromEntries(handoff.selectedThoughts.map(({ id }) => [id, null])),
  };
}

function active(state: CandidateState): state is ActiveState {
  return state.kind !== "closed" && state.kind !== "stale";
}

export function editX(state: CandidateState, x: string): CandidateState {
  if (!active(state) || state.x === x) return state;
  return {
    kind: "draft", ...grounding(state), x, revision: state.revision + 1,
    classifications: Object.fromEntries(state.handoff.selectedThoughts.map(({ id }) => [id, null])),
  };
}

export function editClassification(state: CandidateState, id: string, answer: ClassificationAnswer): CandidateState {
  if (!active(state) || !Object.prototype.hasOwnProperty.call(state.classifications, id) ||
      (answer !== null && answer !== "present" && answer !== "absent" && answer !== "cannotTell") ||
      state.classifications[id] === answer) return state;
  return {
    kind: "draft", ...grounding(state), revision: state.revision + 1,
    classifications: { ...state.classifications, [id]: answer },
  };
}

function grounding(state: ActiveState): CandidateGrounding {
  return {
    attemptId: state.attemptId, revision: state.revision, handoff: state.handoff,
    x: state.x, classifications: state.classifications,
  };
}

export function isValidHandoff(handoff: CandidateHandoff): boolean {
  if (handoff.sourceType !== "thought" || !handoff.comparisonFrame.trim() || handoff.selectedThoughts.length < 2) return false;
  const ids = handoff.selectedThoughts.map((source) => source.id);
  if (new Set(ids).size !== ids.length || !ids.includes(handoff.anchorId) ||
      handoff.selectedThoughts.some((source) => source.recordType !== "thought" || !source.id || !source.stateToken ||
        typeof source.content !== "string" || (source.linkedSituation !== null &&
          (!source.linkedSituation.id || !source.linkedSituation.stateToken ||
            typeof source.linkedSituation.description !== "string")))) return false;
  const others = ids.filter((id) => id !== handoff.anchorId);
  if (!sameKeys(handoff.experienceRelations, others)) return false;
  return others.every((id) => isExperienceRelation(handoff.experienceRelations[id]));
}

function isExperienceRelation(value: unknown): value is ExperienceRelation {
  return value === "sameOrPossiblySame" || value === "differentExperiences" || value === "cannotTell";
}

function sameKeys(object: Readonly<Record<string, unknown>>, ids: readonly string[]): boolean {
  const keys = Object.keys(object);
  return keys.length === ids.length && keys.every((key) => ids.includes(key));
}

export function readyForCandidateAnalysis(state: CandidateState): boolean {
  if (!active(state) || state.kind === "checking" ||
      !isValidHandoff(state.handoff) || !state.x.trim()) return false;
  const ids = state.handoff.selectedThoughts.map((source) => source.id);
  return sameKeys(state.classifications, ids) && ids.every((id) =>
    state.classifications[id] === "present" || state.classifications[id] === "absent" || state.classifications[id] === "cannotTell");
}

export function syncComparisonEdit(state: CandidateState, previous: ComparisonDraft, next: ComparisonDraft): CandidateState {
  if (!active(state)) return state;
  const beforeIds = previous.selectedRecords.map((selection) => selection.status === "current" ? selection.source.id : selection.id);
  const afterIds = next.selectedRecords.map((selection) => selection.status === "current" ? selection.source.id : selection.id);
  if (previous.sourceType !== next.sourceType || previous.comparisonFrame !== next.comparisonFrame ||
      previous.anchorId !== next.anchorId || beforeIds.length !== afterIds.length ||
      beforeIds.some((id, index) => id !== afterIds[index])) return { kind: "closed" };

  const nextRelations: Record<string, ExperienceRelation> = {};
  let relevantChanged = false;
  for (const id of afterIds) {
    if (id === state.handoff.anchorId) continue;
    const relation = next.comparisons[id]?.experienceRelation;
    if (!isExperienceRelation(relation)) return { kind: "closed" };
    nextRelations[id] = relation;
    if (state.classifications[id] === "present" && state.handoff.experienceRelations[id] !== relation) relevantChanged = true;
  }
  const relationChanged = afterIds.some((id) => id !== state.handoff.anchorId &&
    state.handoff.experienceRelations[id] !== nextRelations[id]);
  if (!relationChanged) return state;
  const updated = { ...grounding(state), handoff: { ...state.handoff, experienceRelations: nextRelations } };
  if (relevantChanged) return { kind: "draft", ...updated, revision: state.revision + 1 };
  return { ...state, handoff: updated.handoff };
}

export function staleCandidate(reason: "sourceChanged" | "sourceNotFound", affectedIds: readonly string[]): CandidateState {
  // Do not retain a generated claim or any old source/context text in stale state.
  return { kind: "stale", reason, affectedIds };
}

export function changedScopeSources(handoff: CandidateHandoff, currentSources: readonly ComparisonSource[]): {
  reason: "sourceChanged" | "sourceNotFound"; affectedIds: readonly string[];
} | null {
  const current = new Map<string, ComparisonSource>();
  const duplicates = new Set<string>();
  for (const source of currentSources) {
    const key = `${source.recordType}:${source.id}`;
    if (current.has(key)) duplicates.add(key);
    current.set(key, source);
  }
  const changed: string[] = [];
  const missing: string[] = [];
  for (const source of handoff.selectedThoughts) {
    const key = `thought:${source.id}`;
    const next = current.get(key);
    if (!next) { missing.push(source.id); continue; }
    if (duplicates.has(key) || next.stateToken !== source.stateToken || next.content !== source.content ||
        next.linkedSituation?.id !== source.linkedSituation?.id ||
        next.linkedSituation?.stateToken !== source.linkedSituation?.stateToken ||
        next.linkedSituation?.description !== source.linkedSituation?.description) changed.push(source.id);
  }
  return missing.length ? { reason: "sourceNotFound", affectedIds: [...missing, ...changed] }
    : changed.length ? { reason: "sourceChanged", affectedIds: changed } : null;
}

export type CandidateDecision =
  | { kind: "zeroResult"; reason: ZeroReason }
  | { kind: "proposed"; payload: ThoughtRecurrenceCandidate };

export function deriveCandidate(ground: CandidateGrounding, requestId: number): CandidateDecision {
  const state: CandidateState = { kind: "draft", ...ground };
  if (!readyForCandidateAnalysis(state)) throw new Error("Candidate grounding is incomplete or invalid");
  const scope = ground.handoff.selectedThoughts.map((source): CandidateScopeItem => ({
    source, classification: ground.classifications[source.id] as Classification,
  }));
  const present = scope.filter((item) => item.classification === "present");
  if (ground.classifications[ground.handoff.anchorId] !== "present") return { kind: "zeroResult", reason: "anchorNotPresent" };
  if (present.length < 2) return { kind: "zeroResult", reason: "insufficientPresent" };
  const relations = present.filter((item) => item.source.id !== ground.handoff.anchorId).map((item): PresentExperienceRelation => ({
    anchorId: ground.handoff.anchorId,
    thoughtId: item.source.id,
    relation: ground.handoff.experienceRelations[item.source.id],
  }));
  const proofEdges = relations.filter((edge) => edge.relation === "differentExperiences");
  if (proofEdges.length === 0) return { kind: "zeroResult", reason: "noDifferentExperiencePair" };
  return {
    kind: "proposed",
    payload: {
      proposer: "system",
      mechanism: CANDIDATE_MECHANISM,
      claimKind: "userGroundedThoughtAcrossDifferentExperiences",
      claimScope: "reviewedThoughtRecordsOnly",
      provenExperienceLowerBound: 2,
      generation: { attemptId: ground.attemptId, dependencyRevision: ground.revision, requestId },
      semanticAspect: { text: ground.x, author: "user" },
      comparisonFrame: { text: ground.handoff.comparisonFrame, author: "user" },
      sourceType: "thought",
      anchorId: ground.handoff.anchorId,
      analysisScope: scope,
      presentExperienceRelations: relations,
      proofEdges,
      absentIds: scope.filter((item) => item.classification === "absent").map((item) => item.source.id),
      cannotTellIds: scope.filter((item) => item.classification === "cannotTell").map((item) => item.source.id),
      presentWithoutDistinctExperienceIds: relations.filter((edge) => edge.relation !== "differentExperiences").map((edge) => edge.thoughtId),
      limitations: ["selectedRecordsOnly", "userMarkedContentNotVerifiedTruth", "differentExperiencesNotIndependentEvidence", "noSemanticEventTime"],
    },
  };
}

export function startChecking(state: CandidateState, purpose: "analyze" | "focus", requestId: number): CandidateState {
  if (!active(state)) return state;
  const previous = state.kind === "proposed" ? { kind: "proposed" as const, payload: state.payload }
    : state.kind === "zeroResult" ? { kind: "zeroResult" as const, reason: state.reason }
      : state.kind === "checking" ? state.previous : undefined;
  return { kind: "checking", ...grounding(state), purpose, requestId, previous };
}

export function candidateRequestIsCurrent(
  state: CandidateState, requestId: number, attemptId: number, revision: number,
): state is Extract<ActiveState, { kind: "checking" }> {
  return state.kind === "checking" && state.requestId === requestId &&
    state.attemptId === attemptId && state.revision === revision;
}

export function finishChecking(state: CandidateState, decision: CandidateDecision): CandidateState {
  if (!active(state) || state.kind !== "checking") return state;
  return { ...grounding(state), ...decision };
}

export function restoreCheckedResult(state: CandidateState): CandidateState {
  if (!active(state) || state.kind !== "checking") return state;
  return state.previous ? { ...grounding(state), ...state.previous } : { kind: "draft", ...grounding(state) };
}

export function failCandidateLoad(state: CandidateState, code: HistoryLoadError): CandidateState {
  if (!active(state) || state.kind !== "checking") return state;
  return { kind: "loadError", ...grounding(state), code };
}

export function dismissCandidate(state: CandidateState): CandidateState {
  if (!active(state) || state.kind !== "proposed") return state;
  return { kind: "dismissed", ...grounding(state) };
}

export function returnToGrounding(state: CandidateState): CandidateState {
  if (!active(state) || state.kind === "draft") return state;
  return { kind: "draft", ...grounding(state), revision: state.revision + 1 };
}
