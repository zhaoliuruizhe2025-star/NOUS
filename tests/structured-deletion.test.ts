/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

import { invoke } from "@tauri-apps/api/core";
import { confirmDeletion } from "../src/app/deletion";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const file = (path: string) => readFileSync(join(root, path), "utf8");

describe("structured deletion boundary", () => {
  it("invokes only the typed dedicated command with target and expected state", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ contexts: [], standaloneObservations: [], standaloneThoughts: [] });
    const request = { recordType: "thought" as const, targetId: "t", expectedStateToken: "initial:thought:t" };
    await confirmDeletion(request);
    expect(invoke).toHaveBeenCalledWith("delete_structured_record", { request });
    expect(file("src/app/deletion.ts")).not.toMatch(/\b(?:subjectId|sql|cascade|detach|createdAtMs)\b/);
  });

  it("keeps opening and cancellation local; only explicit confirmation invokes deletion", () => {
    const component = file("src/components/StructuredDeletion.tsx");
    expect(component).toContain("onCancel");
    expect(component).toContain("onClick={onCancel}");
    expect(component).toContain("onClick={() => void confirm()}");
    expect(component.match(/confirmDeletion\(/g)).toHaveLength(1);
    expect(component).toContain("expectedStateToken: selected.record.stateToken");
    expect(component).toContain("onDeleted(history)");
    expect(component).not.toMatch(/\b(?:sql|INSERT|UPDATE|DELETE FROM)\b/);
  });

  it("shows delete separately from correction and never displays technical identity", () => {
    const history = file("src/components/StructuredHistory.tsx");
    expect(history).toContain("messages.deletion.open");
    expect(history).toContain("copy.open");
    expect(history).toContain("<StructuredDeletion");
    expect(history).toContain("onDeleted={(history) => { setState({ status: \"loaded\", history });");
    expect(file("src/components/StructuredDeletion.tsx")).not.toMatch(/>\s*\{selected\.record\.(?:id|stateToken)\}\s*</);
  });
});
