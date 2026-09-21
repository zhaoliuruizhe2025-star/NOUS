import { invoke } from "@tauri-apps/api/core";

export interface StructuredHistoryObservation {
  id: string;
  content: string;
}

export interface StructuredHistoryThought {
  id: string;
  content: string;
  subjectiveConviction: number | null;
}

export interface StructuredHistoryContext {
  id: string;
  description: string;
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
