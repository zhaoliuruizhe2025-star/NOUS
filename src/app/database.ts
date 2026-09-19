import { invoke } from "@tauri-apps/api/core";

interface DatabaseStatusResponse {
  schemaVersion: number;
}

export async function initializeDatabase(): Promise<DatabaseStatusResponse> {
  const status = await invoke<DatabaseStatusResponse>("database_status");

  if (status.schemaVersion !== 2) {
    throw new Error("Local database migration verification failed.");
  }

  return status;
}
