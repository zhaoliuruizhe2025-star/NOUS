import type { StructuredHistory } from "./history";

export type SourceType = "thought" | "observation";
export type Comparability = "comparable" | "notComparable" | "cannotTell";
export type MeaningRelation = "similar" | "materiallyDifferent" | "cannotTell";
export type ExperienceRelation =
  | "sameOrPossiblySame"
  | "differentExperiences"
  | "cannotTell";

export interface ComparisonSource {
  recordType: SourceType;
  id: string;
  content: string;
  stateToken: string;
  linkedSituation: { id: string; description: string; stateToken: string } | null;
}

export type Selection =
  | { status: "current"; source: ComparisonSource }
  | { status: "stale"; recordType: SourceType; id: string; reason: "sourceChanged" };

export type Judgment =
  | { comparability: null; experienceRelation: ExperienceRelation | null }
  | {
      comparability: "comparable";
      meaningRelation: MeaningRelation | null;
      experienceRelation: ExperienceRelation | null;
    }
  | {
      comparability: "notComparable" | "cannotTell";
      experienceRelation: ExperienceRelation | null;
    };

export interface ComparisonDraft {
  sourceType: SourceType | null;
  comparisonFrame: string;
  selectedRecords: Selection[];
  anchorId: string | null;
  comparisons: Record<string, Judgment>;
}

const unanswered = (): Judgment => ({ comparability: null, experienceRelation: null });

export function emptyComparison(): ComparisonDraft {
  return {
    sourceType: null,
    comparisonFrame: "",
    selectedRecords: [],
    anchorId: null,
    comparisons: {},
  };
}

export function projectComparisonSources(history: StructuredHistory): ComparisonSource[] {
  return [
    ...history.contexts.flatMap((context): ComparisonSource[] => {
      const linkedSituation = {
        id: context.id,
        description: context.description,
        stateToken: context.stateToken,
      };
      return [
        ...context.observations.map((record) => ({
          recordType: "observation" as const,
          id: record.id,
          content: record.content,
          stateToken: record.stateToken,
          linkedSituation,
        })),
        ...context.thoughts.map((record) => ({
          recordType: "thought" as const,
          id: record.id,
          content: record.content,
          stateToken: record.stateToken,
          linkedSituation,
        })),
      ];
    }),
    ...history.standaloneObservations.map((record) => ({
      recordType: "observation" as const,
      id: record.id,
      content: record.content,
      stateToken: record.stateToken,
      linkedSituation: null,
    })),
    ...history.standaloneThoughts.map((record) => ({
      recordType: "thought" as const,
      id: record.id,
      content: record.content,
      stateToken: record.stateToken,
      linkedSituation: null,
    })),
  ];
}

export function changeSourceType(draft: ComparisonDraft, sourceType: SourceType): ComparisonDraft {
  if (draft.sourceType === sourceType) return draft;
  return { ...emptyComparison(), sourceType, comparisonFrame: draft.comparisonFrame };
}

export function changeComparisonFrame(draft: ComparisonDraft, comparisonFrame: string): ComparisonDraft {
  if (draft.comparisonFrame === comparisonFrame) return draft;
  const hasAnswer = Object.values(draft.comparisons).some((judgment) =>
    judgment.comparability !== null || judgment.experienceRelation !== null,
  );
  return {
    ...draft,
    comparisonFrame,
    comparisons: hasAnswer ? resetJudgments(draft) : draft.comparisons,
  };
}

function resetJudgments(draft: ComparisonDraft): Record<string, Judgment> {
  if (draft.anchorId === null) return {};
  return Object.fromEntries(
    draft.selectedRecords
      .flatMap((selected) => selected.status === "current" && selected.source.id !== draft.anchorId
        ? [[selected.source.id, unanswered()]]
        : []),
  );
}

export function addSource(draft: ComparisonDraft, source: ComparisonSource): ComparisonDraft {
  if (draft.sourceType !== source.recordType) return draft;
  const existing = draft.selectedRecords.find((selected) => selectionId(selected) === source.id);
  if (existing?.status === "current") return draft;
  const selectedRecords = [
    ...draft.selectedRecords.filter((selected) => selectionId(selected) !== source.id),
    { status: "current" as const, source },
  ];
  return {
    ...draft,
    selectedRecords,
    comparisons: draft.anchorId === null || draft.anchorId === source.id
      ? draft.comparisons
      : { ...draft.comparisons, [source.id]: unanswered() },
  };
}

export function removeSource(draft: ComparisonDraft, id: string): ComparisonDraft {
  if (!draft.selectedRecords.some((selected) => selectionId(selected) === id)) return draft;
  const selectedRecords = draft.selectedRecords.filter((selected) => selectionId(selected) !== id);
  if (draft.anchorId === id) {
    return { ...draft, selectedRecords, anchorId: null, comparisons: {} };
  }
  const comparisons = { ...draft.comparisons };
  delete comparisons[id];
  return { ...draft, selectedRecords, comparisons };
}

export function chooseAnchor(draft: ComparisonDraft, id: string): ComparisonDraft {
  if (draft.anchorId === id) return draft;
  if (!draft.selectedRecords.some((selected) => selected.status === "current" && selected.source.id === id)) {
    return draft;
  }
  const next = { ...draft, anchorId: id };
  return { ...next, comparisons: resetJudgments(next) };
}

export function answerComparability(
  draft: ComparisonDraft,
  id: string,
  comparability: Comparability,
): ComparisonDraft {
  const previous = draft.comparisons[id];
  if (!previous) return draft;
  const judgment: Judgment = comparability === "comparable"
    ? {
        comparability,
        meaningRelation: previous.comparability === "comparable" ? previous.meaningRelation : null,
        experienceRelation: previous.experienceRelation,
      }
    : { comparability, experienceRelation: previous.experienceRelation };
  return { ...draft, comparisons: { ...draft.comparisons, [id]: judgment } };
}

export function answerMeaning(
  draft: ComparisonDraft,
  id: string,
  meaningRelation: MeaningRelation,
): ComparisonDraft {
  const previous = draft.comparisons[id];
  if (!previous || previous.comparability !== "comparable") return draft;
  return {
    ...draft,
    comparisons: { ...draft.comparisons, [id]: { ...previous, meaningRelation } },
  };
}

export function answerExperience(
  draft: ComparisonDraft,
  id: string,
  experienceRelation: ExperienceRelation,
): ComparisonDraft {
  const previous = draft.comparisons[id];
  if (!previous) return draft;
  return {
    ...draft,
    comparisons: { ...draft.comparisons, [id]: { ...previous, experienceRelation } },
  };
}

export function selectionId(selection: Selection): string {
  return selection.status === "current" ? selection.source.id : selection.id;
}

export function readyForComparison(draft: ComparisonDraft): boolean {
  if (!draft.comparisonFrame.trim() || draft.selectedRecords.length < 2 || draft.anchorId === null) {
    return false;
  }
  if (draft.sourceType !== "thought" && draft.sourceType !== "observation") return false;
  const selected = draft.selectedRecords;
  if (selected.some((item) => item.status !== "current" || item.source.recordType !== draft.sourceType)) {
    return false;
  }
  const ids = selected.map(selectionId);
  if (new Set(ids).size !== ids.length || !ids.includes(draft.anchorId)) return false;
  const otherIds = ids.filter((id) => id !== draft.anchorId);
  if (Object.keys(draft.comparisons).length !== otherIds.length) return false;
  return otherIds.every((id) => {
    const judgment = draft.comparisons[id];
    if (!judgment || judgment.comparability === null || judgment.experienceRelation === null) return false;
    return judgment.comparability !== "comparable" || judgment.meaningRelation !== null;
  });
}

export function mayRestoreReviewedSummary(
  draft: ComparisonDraft,
  requested: boolean,
  sourcesChanged: boolean,
  requestEditVersion: number,
  currentEditVersion: number,
): boolean {
  return requested
    && !sourcesChanged
    && requestEditVersion === currentEditVersion
    && readyForComparison(draft);
}

function sourceChanged(previous: ComparisonSource, next: ComparisonSource): boolean {
  return previous.stateToken !== next.stateToken
    || previous.linkedSituation?.id !== next.linkedSituation?.id
    || previous.linkedSituation?.stateToken !== next.linkedSituation?.stateToken;
}

export function reconcileComparison(
  draft: ComparisonDraft,
  currentSources: ComparisonSource[],
): { draft: ComparisonDraft; changed: boolean } {
  const current = new Map(currentSources.map((source) => [`${source.recordType}:${source.id}`, source]));
  let changed = false;
  let anchorInvalid = false;
  const comparisons = { ...draft.comparisons };
  const selectedRecords: Selection[] = [];
  for (const selected of draft.selectedRecords) {
    const id = selectionId(selected);
    const type = selected.status === "current" ? selected.source.recordType : selected.recordType;
    const next = current.get(`${type}:${id}`);
    if (!next) {
      changed = true;
      if (id === draft.anchorId) anchorInvalid = true;
      delete comparisons[id];
      continue;
    }
    if (selected.status === "stale") {
      selectedRecords.push(selected);
      changed = true;
      continue;
    }
    if (sourceChanged(selected.source, next)) {
      changed = true;
      if (id === draft.anchorId) anchorInvalid = true;
      delete comparisons[id];
      selectedRecords.push({ status: "stale", recordType: type, id, reason: "sourceChanged" });
      continue;
    }
    selectedRecords.push({ status: "current", source: next });
  }
  return {
    draft: {
      ...draft,
      selectedRecords,
      anchorId: anchorInvalid ? null : draft.anchorId,
      comparisons: anchorInvalid ? {} : comparisons,
    },
    changed,
  };
}

// A late read may publish only while this view and its latest request are active.
export function createComparisonRefreshGate() {
  let generation = 0;
  let active = true;
  return {
    start(): number { return ++generation; },
    isCurrent(request: number): boolean { return active && request === generation; },
    dispose(): void { active = false; generation += 1; },
  };
}
