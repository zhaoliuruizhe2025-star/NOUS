/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { createDatabaseBackup, exportUserData } from "../src/app/data_portability";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const file = (path: string) => readFileSync(join(root, path), "utf8");

describe("data portability action boundary", () => {
  it("uses separate payload-free commands for export and backup", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ status: "cancelled" }).mockResolvedValueOnce({ status: "completed" });
    expect(await exportUserData()).toEqual({ status: "cancelled" });
    expect(await createDatabaseBackup()).toEqual({ status: "completed" });
    expect(invoke).toHaveBeenNthCalledWith(1, "export_user_data");
    expect(invoke).toHaveBeenNthCalledWith(2, "create_database_backup");
  });

  it("keeps dialog cancellation benign and lets Rust own file selection", () => {
    const component = file("src/components/DataPortability.tsx");
    expect(component).toContain('result.status === "cancelled" ? { kind: "idle" }');
    expect(component).toContain('run("export")');
    expect(component).toContain('run("backup")');
    expect(file("src/app/data_portability.ts")).not.toMatch(/destination|path|subjectId|sql|writeFile/i);
    expect(component).not.toMatch(/\b(?:SELECT|INSERT|UPDATE|DELETE FROM)\b/i);
  });

  it("registers only the Rust dialog capability, not frontend filesystem authority", () => {
    const cargo = file("src-tauri/Cargo.toml");
    const lib = file("src-tauri/src/lib.rs");
    const capability = file("src-tauri/capabilities/default.json");
    expect(cargo).toContain('tauri-plugin-dialog = "=2.7.3"');
    expect(lib).toContain("tauri_plugin_dialog::init()");
    expect(lib).toContain("commands::export_user_data");
    expect(lib).toContain("commands::create_database_backup");
    expect(capability).not.toContain("dialog:");
    expect(capability).not.toContain("fs:");
  });
});
