import { invoke } from "@tauri-apps/api/core";

import type { StructuredHistory } from "./history";

export type DeletionRequest = {
  recordType: "situation" | "observation" | "thought";
  targetId: string;
  expectedStateToken: string;
};

export async function confirmDeletion(request: DeletionRequest): Promise<StructuredHistory> {
  return invoke<StructuredHistory>("delete_structured_record", { request });
}
