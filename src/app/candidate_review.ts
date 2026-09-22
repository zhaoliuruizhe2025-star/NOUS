import type { StructuredHistory } from "./history";

export type CandidateRecordType = "situation" | "observation" | "thought";
export type EvidenceRole =
  | "supporting"
  | "contradicting"
  | "complicating"
  | "contextualizing";

export interface CandidateSituationContext {
  id: string;
  stateToken: string;
  description: string;
}

export interface CandidateSource {
  recordType: CandidateRecordType;
  id: string;
  stateToken: string;
  content: string;
  linkedSituation: CandidateSituationContext | null;
  subjectiveConviction: number | null;
}

export interface CandidateSourceGroup {
  situation: CandidateSource;
  observations: CandidateSource[];
  thoughts: CandidateSource[];
}

export interface CandidateSourceSnapshot {
  contexts: CandidateSourceGroup[];
  standaloneObservations: CandidateSource[];
  standaloneThoughts: CandidateSource[];
}

export type SelectedCandidateSource =
  | { status: "current"; source: CandidateSource; role: EvidenceRole | null }
  | {
      status: "stale";
      recordType: CandidateRecordType;
      id: string;
      reason: "sourceChanged" | "sourceNotFound";
    };

export function sourceKey(recordType: CandidateRecordType, id: string): string {
  return `${recordType}:${id}`;
}

export function selectedSourceKey(selected: SelectedCandidateSource): string {
  return selected.status === "current"
    ? sourceKey(selected.source.recordType, selected.source.id)
    : sourceKey(selected.recordType, selected.id);
}

export function toggleCandidateSource(
  selectedSources: SelectedCandidateSource[],
  source: CandidateSource,
): SelectedCandidateSource[] {
  const key = sourceKey(source.recordType, source.id);
  const existing = selectedSources.find((selected) => selectedSourceKey(selected) === key);
  const others = selectedSources.filter((selected) => selectedSourceKey(selected) !== key);
  return existing?.status === "current"
    ? others
    : [...others, { status: "current", source, role: null }];
}

export function assignCandidateRole(
  selectedSources: SelectedCandidateSource[],
  source: CandidateSource,
  role: EvidenceRole | null,
): SelectedCandidateSource[] {
  const key = sourceKey(source.recordType, source.id);
  return selectedSources.map((selected) =>
    selected.status === "current" && selectedSourceKey(selected) === key
      ? { ...selected, role }
      : selected,
  );
}

export function projectCandidateSources(history: StructuredHistory): CandidateSourceSnapshot {
  return {
    contexts: history.contexts.map((context) => {
      const linkedSituation = {
        id: context.id,
        stateToken: context.stateToken,
        description: context.description,
      };
      return {
        situation: {
          recordType: "situation" as const,
          id: context.id,
          stateToken: context.stateToken,
          content: context.description,
          linkedSituation: null,
          subjectiveConviction: null,
        },
        observations: context.observations.map((observation) => ({
          recordType: "observation" as const,
          id: observation.id,
          stateToken: observation.stateToken,
          content: observation.content,
          linkedSituation,
          subjectiveConviction: null,
        })),
        thoughts: context.thoughts.map((thought) => ({
          recordType: "thought" as const,
          id: thought.id,
          stateToken: thought.stateToken,
          content: thought.content,
          linkedSituation,
          subjectiveConviction: thought.subjectiveConviction,
        })),
      };
    }),
    standaloneObservations: history.standaloneObservations.map((observation) => ({
      recordType: "observation",
      id: observation.id,
      stateToken: observation.stateToken,
      content: observation.content,
      linkedSituation: null,
      subjectiveConviction: null,
    })),
    standaloneThoughts: history.standaloneThoughts.map((thought) => ({
      recordType: "thought",
      id: thought.id,
      stateToken: thought.stateToken,
      content: thought.content,
      linkedSituation: null,
      subjectiveConviction: thought.subjectiveConviction,
    })),
  };
}

export function allCandidateSources(snapshot: CandidateSourceSnapshot): CandidateSource[] {
  return [
    ...snapshot.contexts.flatMap((group) => [
      group.situation,
      ...group.observations,
      ...group.thoughts,
    ]),
    ...snapshot.standaloneObservations,
    ...snapshot.standaloneThoughts,
  ];
}

export function findCandidateSource(
  snapshot: CandidateSourceSnapshot,
  recordType: CandidateRecordType,
  id: string,
): CandidateSource | undefined {
  return allCandidateSources(snapshot).find(
    (source) => source.recordType === recordType && source.id === id,
  );
}

export function canReviewCandidate(
  candidateText: string,
  claimScope: string,
  selectedSources: SelectedCandidateSource[],
): boolean {
  return (
    candidateText.trim().length > 0 &&
    claimScope.trim().length > 0 &&
    selectedSources.length > 0 &&
    selectedSources.every((selected) => selected.status === "current")
  );
}

export function compareCandidateSources(
  selectedSources: SelectedCandidateSource[],
  snapshot: CandidateSourceSnapshot,
): { selectedSources: SelectedCandidateSource[]; stale: boolean } {
  const current = new Map(
    allCandidateSources(snapshot).map((source) => [sourceKey(source.recordType, source.id), source]),
  );
  let stale = false;
  const compared = selectedSources.map((selected): SelectedCandidateSource => {
    if (selected.status === "stale") {
      stale = true;
      return selected;
    }
    const previous = selected.source;
    const next = current.get(sourceKey(previous.recordType, previous.id));
    if (!next) {
      stale = true;
      return {
        status: "stale",
        recordType: previous.recordType,
        id: previous.id,
        reason: "sourceNotFound",
      };
    }
    const beforeContext = previous.linkedSituation;
    const afterContext = next.linkedSituation;
    if (
      next.stateToken !== previous.stateToken ||
      beforeContext?.id !== afterContext?.id ||
      beforeContext?.stateToken !== afterContext?.stateToken
    ) {
      stale = true;
      return {
        status: "stale",
        recordType: previous.recordType,
        id: previous.id,
        reason: "sourceChanged",
      };
    }
    return selected;
  });
  return { selectedSources: compared, stale };
}

// Only the latest refresh may publish a result; unmount permanently closes the gate.
export function createRefreshGate() {
  let generation = 0;
  let active = true;
  return {
    start(): number {
      generation += 1;
      return generation;
    },
    isCurrent(request: number): boolean {
      return active && request === generation;
    },
    dispose(): void {
      active = false;
      generation += 1;
    },
  };
}
