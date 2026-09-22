import { invoke } from "@tauri-apps/api/core";

export type PortabilityResult = { status: "completed" | "cancelled" };

export function exportUserData(): Promise<PortabilityResult> {
  return invoke<PortabilityResult>("export_user_data");
}

export function createDatabaseBackup(): Promise<PortabilityResult> {
  return invoke<PortabilityResult>("create_database_backup");
}
