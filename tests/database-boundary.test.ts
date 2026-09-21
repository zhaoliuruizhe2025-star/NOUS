/// <reference types="node" />

import { readFileSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const repositoryRoot = dirname(dirname(fileURLToPath(import.meta.url)));

function readRepositoryFile(relativePath: string): string {
  return readFileSync(join(repositoryRoot, relativePath), "utf8");
}

function frontendSource(relativeDirectory: string): string {
  const absoluteDirectory = join(repositoryRoot, relativeDirectory);

  return readdirSync(absoluteDirectory, { withFileTypes: true })
    .flatMap((entry) => {
      const relativePath = join(relativeDirectory, entry.name);
      if (entry.isDirectory()) return frontendSource(relativePath);
      if (!/\.(ts|tsx)$/.test(entry.name)) return [];
      return [readRepositoryFile(relativePath)];
    })
    .join("\n");
}

describe("frontend database security boundary", () => {
  it("contains no JavaScript SQL plugin or caller-selected SQL path", () => {
    const source = frontendSource("src");
    const databaseSource = readRepositoryFile("src/app/database.ts");

    expect(source).not.toContain("@tauri-apps/plugin-sql");
    expect(source).not.toMatch(
      /\b(?:FROM|INTO|UPDATE|JOIN)\s+["`[]?(?:app_metadata|self_subjects|person_references|situations|observations|thoughts|emotions|beliefs|belief_revisions|value_revisions|memories|decisions|outcomes|evidence_links)\b/i,
    );
    expect(databaseSource).toContain(
      'invoke<DatabaseStatusResponse>("database_status")',
    );
    expect(databaseSource).not.toMatch(/invoke[^;]*,\s*\{/);
    expect(databaseSource).toContain("status.schemaVersion !== 4");
  });

  it("grants the frontend no SQL load, select, or execute capability", () => {
    const capability = JSON.parse(
      readRepositoryFile("src-tauri/capabilities/default.json"),
    ) as { permissions: string[] };

    expect(capability.permissions).not.toContain("sql:allow-load");
    expect(capability.permissions).not.toContain("sql:allow-select");
    expect(capability.permissions).not.toContain("sql:allow-execute");
    expect(capability.permissions.some((permission) => permission.startsWith("sql:"))).toBe(
      false,
    );
  });

  it("exposes a fixed Rust readiness command with no caller-selected input", () => {
    const commandSource = readRepositoryFile("src-tauri/src/commands.rs");
    const librarySource = readRepositoryFile("src-tauri/src/lib.rs");

    expect(commandSource).toMatch(
      /async fn database_status\(\s*database: State<'_, SharedSqlitePool>,?\s*\)/,
    );
    expect(commandSource).not.toMatch(
      /async fn database_status\([^)]*\b(?:sql|query|table|url|operation)\b/i,
    );
    expect(librarySource).toContain(
      "commands::database_status",
    );
    expect(librarySource).toContain("commands::save_structured_capture");
    expect(librarySource).toContain("commands::load_structured_history");
    expect(commandSource).toMatch(
      /async fn load_structured_history\(\s*database: State<'_, SharedSqlitePool>,?\s*\)/,
    );
    expect(commandSource).not.toMatch(
      /async fn load_structured_history\([^)]*\b(?:subject|sql|query|table|filter|order)\b/i,
    );
    expect(commandSource).not.toMatch(/async fn (?:insert|update|delete|execute)_/i);
  });

  it("preserves the initializing, ready, and error UI state transitions", () => {
    const appSource = readRepositoryFile("src/app/App.tsx");

    expect(appSource).toContain('useState<DatabaseStatus>("initializing")');
    expect(appSource).toContain('setDatabaseStatus("ready")');
    expect(appSource).toContain('setDatabaseStatus("error")');
    expect(appSource).toContain("messages.database[databaseStatus]");
  });
});
