import { invoke } from "@tauri-apps/api/core";

export interface StructuredHistoryObservation {
  id: string;
  content: string;
  stateToken: string;
  corrected: boolean;
  corrections: ObservationCorrection[];
}

export interface ContextReference {
  id: string;
  description: string;
}

export interface SituationCorrection {
  sequence: number;
  beforeDescription: string;
  afterDescription: string;
  note: string | null;
}

export interface ObservationCorrection {
  sequence: number;
  beforeContent: string;
  afterContent: string;
  beforeContext: ContextReference | null;
  afterContext: ContextReference | null;
  note: string | null;
}

export interface ThoughtCorrection extends ObservationCorrection {
  beforeSubjectiveConviction: number | null;
  afterSubjectiveConviction: number | null;
}

export interface StructuredHistoryThought {
  id: string;
  content: string;
  subjectiveConviction: number | null;
  stateToken: string;
  corrected: boolean;
  corrections: ThoughtCorrection[];
}

export interface StructuredHistoryContext {
  id: string;
  description: string;
  stateToken: string;
  corrected: boolean;
  corrections: SituationCorrection[];
  observations: StructuredHistoryObservation[];
  thoughts: StructuredHistoryThought[];
}

export interface StructuredHistory {
  contexts: StructuredHistoryContext[];
  standaloneObservations: StructuredHistoryObservation[];
  standaloneThoughts: StructuredHistoryThought[];
}

export interface StructuredHistoryCommandError {
  code: string;
}

export function isStructuredHistoryEmpty(history: StructuredHistory): boolean {
  return (
    history.contexts.length === 0 &&
    history.standaloneObservations.length === 0 &&
    history.standaloneThoughts.length === 0
  );
}

export async function loadStructuredHistory(): Promise<StructuredHistory> {
  return invoke<StructuredHistory>("load_structured_history");
}
