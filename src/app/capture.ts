import { invoke } from "@tauri-apps/api/core";

export interface CaptureDraftItem {
  key: string;
  text: string;
}

export interface CaptureDraft {
  context: string;
  observations: CaptureDraftItem[];
  thoughts: CaptureDraftItem[];
}

interface SituationCaptureInput {
  id: string;
  description: string;
}

interface ObservationCaptureInput {
  id: string;
  content: string;
}

interface ThoughtCaptureInput {
  id: string;
  content: string;
}

export interface SaveStructuredCaptureRequest {
  situation: SituationCaptureInput | null;
  observations: ObservationCaptureInput[];
  thoughts: ThoughtCaptureInput[];
}

export interface SavedStructuredCapture {
  situation: { id: string; description: string } | null;
  observations: Array<{ id: string; situationId: string | null; content: string }>;
  thoughts: Array<{
    id: string;
    situationId: string | null;
    content: string;
    confidence: null;
  }>;
}

export interface StructuredCaptureCommandError {
  code: string;
  itemIndex: number | null;
}

type IdFactory = () => string;

export function hasSubstantiveCapture(draft: CaptureDraft): boolean {
  return (
    draft.context.trim().length > 0 ||
    draft.observations.some((item) => item.text.trim().length > 0) ||
    draft.thoughts.some((item) => item.text.trim().length > 0)
  );
}

export function hasBlankAddedItem(draft: CaptureDraft): boolean {
  return (
    draft.observations.some((item) => item.text.trim().length === 0) ||
    draft.thoughts.some((item) => item.text.trim().length === 0)
  );
}

export function buildSaveRequest(
  draft: CaptureDraft,
  createId: IdFactory = () => crypto.randomUUID(),
): SaveStructuredCaptureRequest {
  return {
    situation:
      draft.context.trim().length > 0
        ? { id: createId(), description: draft.context }
        : null,
    observations: draft.observations.map((item) => ({
      id: createId(),
      content: item.text,
    })),
    thoughts: draft.thoughts.map((item) => ({
      id: createId(),
      content: item.text,
    })),
  };
}

export async function saveStructuredCapture(
  request: SaveStructuredCaptureRequest,
): Promise<SavedStructuredCapture> {
  return invoke<SavedStructuredCapture>("save_structured_capture", { request });
}
