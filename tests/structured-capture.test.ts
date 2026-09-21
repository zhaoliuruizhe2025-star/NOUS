/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import {
  buildSaveRequest,
  hasBlankAddedItem,
  hasSubstantiveCapture,
  type CaptureDraft,
} from "../src/app/capture";

const repositoryRoot = dirname(dirname(fileURLToPath(import.meta.url)));

function draft(overrides: Partial<CaptureDraft> = {}): CaptureDraft {
  return {
    context: "",
    observations: [],
    thoughts: [],
    ...overrides,
  };
}

describe("structured capture frontend boundary", () => {
  it("builds every approved variable shape without raw input or authority fields", () => {
    let nextId = 0;
    const id = () => `id-${++nextId}`;

    const situationOnly = buildSaveRequest(draft({ context: "A meeting" }), id);
    expect(situationOnly.situation?.description).toBe("A meeting");
    expect(situationOnly.observations).toEqual([]);
    expect(situationOnly.thoughts).toEqual([]);

    const observationOnly = buildSaveRequest(
      draft({ observations: [{ key: "local-1", text: "The meeting ended." }] }),
      id,
    );
    expect(observationOnly.situation).toBeNull();
    expect(observationOnly.observations).toHaveLength(1);

    const thoughtOnly = buildSaveRequest(
      draft({ thoughts: [{ key: "local-2", text: "Maybe I should wait." }] }),
      id,
    );
    expect(thoughtOnly.situation).toBeNull();
    expect(thoughtOnly.thoughts).toHaveLength(1);

    const mixed = buildSaveRequest(
      draft({
        context: "Working with my team",
        observations: [
          { key: "local-3", text: "They finished the draft." },
          { key: "local-4", text: "I took over a section." },
        ],
        thoughts: [{ key: "local-5", text: "They may not trust me." }],
      }),
      id,
    );
    expect(mixed.observations).toHaveLength(2);
    expect(mixed.thoughts).toHaveLength(1);

    const serialized = JSON.stringify(mixed);
    expect(serialized).not.toContain("rawInput");
    expect(serialized).not.toContain("subjectId");
    expect(serialized).not.toContain("persistenceIntent");
    expect(serialized).not.toContain("confidence");
  });

  it("requires substantive content and rejects blank added items", () => {
    expect(hasSubstantiveCapture(draft())).toBe(false);
    expect(hasSubstantiveCapture(draft({ context: "  " }))).toBe(false);
    expect(
      hasSubstantiveCapture(
        draft({ thoughts: [{ key: "thought", text: "A real thought" }] }),
      ),
    ).toBe(true);
    expect(
      hasBlankAddedItem(
        draft({ observations: [{ key: "observation", text: "  " }] }),
      ),
    ).toBe(true);
  });

  it("routes only the Save handler to the dedicated persistence command", () => {
    const component = readFileSync(
      join(repositoryRoot, "src/components/StructuredCapture.tsx"),
      "utf8",
    );
    const api = readFileSync(join(repositoryRoot, "src/app/capture.ts"), "utf8");

    expect(component.match(/saveStructuredCapture\(/g)).toHaveLength(1);
    expect(component).toContain('onClick={handleSave}');
    expect(component).toContain('onClick={reset}');
    expect(component).toContain('setRawInput("")');
    expect(api).toContain('invoke<SavedStructuredCapture>("save_structured_capture", { request })');
    expect(api).not.toContain("rawInput");
    expect(api).not.toContain("persistenceIntent");
  });
});
