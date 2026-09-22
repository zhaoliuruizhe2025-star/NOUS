import { invoke } from "@tauri-apps/api/core";

import type { StructuredHistory } from "./history";

type ContextInput = { kind: "none" } | { kind: "existing"; situationId: string };
type ConvictionInput = { kind: "notReported" } | { kind: "reported"; value: number };

export type CorrectionRequest =
  | {
      recordType: "situation";
      targetId: string;
      expectedStateToken: string;
      description: string;
      note: string | null;
    }
  | {
      recordType: "observation";
      targetId: string;
      expectedStateToken: string;
      content: string;
      context: ContextInput;
      note: string | null;
    }
  | {
      recordType: "thought";
      targetId: string;
      expectedStateToken: string;
      content: string;
      context: ContextInput;
      subjectiveConviction: ConvictionInput;
      note: string | null;
    };

export async function confirmCorrection(request: CorrectionRequest): Promise<StructuredHistory> {
  return invoke<StructuredHistory>("correct_structured_record", { request });
}
