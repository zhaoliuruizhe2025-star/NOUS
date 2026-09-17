import Database from "@tauri-apps/plugin-sql";

export const DATABASE_URL = "sqlite:nous.db";

interface AppMetadataRow {
  value: string;
}

export async function initializeDatabase(): Promise<Database> {
  const database = await Database.load(DATABASE_URL);
  const rows = await database.select<AppMetadataRow[]>(
    "SELECT value FROM app_metadata WHERE key = 'schema_version' LIMIT 1",
  );

  if (rows[0]?.value !== "1") {
    throw new Error("Local database migration verification failed.");
  }

  return database;
}

