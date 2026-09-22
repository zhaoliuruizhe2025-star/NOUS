/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

import { confirmCorrection } from "../src/app/correction";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const read = (path: string) => readFileSync(join(root, path), "utf8");

describe("structured correction frontend boundary", () => {
  beforeEach(() => invoke.mockReset());

  it("sends only a typed reviewed request through the dedicated command", async () => {
    invoke.mockResolvedValue({ contexts: [], standaloneObservations: [], standaloneThoughts: [] });
    const request = {
      recordType: "thought" as const,
      targetId: "t1",
      expectedStateToken: "initial:thought:t1",
      content: "I worried they might not trust me.",
      context: { kind: "none" as const },
      subjectiveConviction: { kind: "reported" as const, value: 60 },
      note: null,
    };
    await confirmCorrection(request);
    expect(invoke).toHaveBeenCalledExactlyOnceWith("correct_structured_record", { request });
    expect(JSON.stringify(request)).not.toMatch(/subjectId|correctionSequence|auditId|createdAtMs|sql|persistenceIntent/);
  });

  it("opens a temporary review and calls the write path only in the explicit confirm handler", () => {
    const component = read("src/components/StructuredCorrection.tsx");
    const history = read("src/components/StructuredHistory.tsx");
    const api = read("src/app/correction.ts");
    expect(history).toContain("onCorrect(selected)");
    expect(component).toContain("async function submit(");
    expect(component).toContain("await confirmCorrection(request)");
    expect(component).toContain("onClick={onCancel}");
    expect(history).toContain("onCancel={() => setSelected(null)}");
    expect(component).toContain("onSaved(history)");
    expect(component).toContain("onClick={onStale}");
    expect(api).not.toContain("@tauri-apps/plugin-sql");
    expect(component).not.toMatch(/\b(?:INSERT|UPDATE|DELETE|SELECT)\b/);
  });

  it("keeps correction provenance distinct from historical user truth", () => {
    const history = read("src/components/StructuredHistory.tsx");
    expect(history).toContain("selected.record.corrected");
    expect(history).toContain("copy.priorMeaning");
    expect(history).toContain("item.beforeContent");
    expect(history).toContain("item.afterContent");
    expect(history).not.toContain("recordedAtMs");
    expect(history).not.toMatch(/>\s*\{selected\.record\.stateToken\}\s*</);
  });
});
