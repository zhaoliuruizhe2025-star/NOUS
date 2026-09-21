/// <reference types="node" />

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

import {
  isStructuredHistoryEmpty,
  type StructuredHistory,
} from "../src/app/history";

const repositoryRoot = dirname(dirname(fileURLToPath(import.meta.url)));

function readRepositoryFile(relativePath: string): string {
  return readFileSync(join(repositoryRoot, relativePath), "utf8");
}

function history(overrides: Partial<StructuredHistory> = {}): StructuredHistory {
  return {
    contexts: [],
    standaloneObservations: [],
    standaloneThoughts: [],
    ...overrides,
  };
}

describe("structured history inspect boundary", () => {
  it("recognizes only the bounded three-type empty state", () => {
    expect(isStructuredHistoryEmpty(history())).toBe(true);
    expect(
      isStructuredHistoryEmpty(
        history({ contexts: [{ id: "s", description: "Context", observations: [], thoughts: [] }] }),
      ),
    ).toBe(false);
    expect(
      isStructuredHistoryEmpty(
        history({ standaloneObservations: [{ id: "o", content: "A report" }] }),
      ),
    ).toBe(false);
    expect(
      isStructuredHistoryEmpty(
        history({
          standaloneThoughts: [
            { id: "t", content: "A thought", subjectiveConviction: null },
          ],
        }),
      ),
    ).toBe(false);
  });

  it("uses one fixed no-input read command and exposes no persistence controls", () => {
    const api = readRepositoryFile("src/app/history.ts");
    const component = readRepositoryFile("src/components/StructuredHistory.tsx");

    expect(api).toContain('invoke<StructuredHistory>("load_structured_history")');
    expect(api).not.toContain("subjectId");
    expect(api).not.toContain("createdAt");
    expect(api).not.toMatch(/invoke<StructuredHistory>\([^)]*,\s*\{/);
    expect(component.match(/loadStructuredHistory\(/g)).toHaveLength(1);
    expect(component).not.toMatch(/\b(?:save|edit|delete|correct|promote)\w*\s*\(/i);
  });

  it("presents only repository-assembled relationships and hides opaque metadata", () => {
    const component = readRepositoryFile("src/components/StructuredHistory.tsx");

    expect(component).toContain("context.observations.map");
    expect(component).toContain("context.thoughts.map");
    expect(component).toContain("state.history.standaloneObservations.map");
    expect(component).toContain("state.history.standaloneThoughts.map");
    expect(component).not.toContain("situationId");
    expect(component).not.toContain("createdAtMs");
    expect(component).not.toContain("subjectId");
    expect(component).not.toMatch(
      />\s*\{(?:context|observation|thought)\.id\}\s*</,
    );
  });

  it("shows subjective conviction only when the persisted value is present", () => {
    const component = readRepositoryFile("src/components/StructuredHistory.tsx");

    expect(component).toContain("thought.subjectiveConviction !== null");
    expect(component).toContain('convictionCopy.replace(');
    expect(component).not.toContain("NOUS confidence");
    expect(component).not.toContain("probability");
  });

  it("loads history only after deliberate navigation from capture", () => {
    const app = readRepositoryFile("src/app/App.tsx");

    expect(app).toContain('type AppView = "capture" | "history"');
    expect(app).toContain('useState<AppView>("capture")');
    expect(app).toContain('onClick={() => setView("history")}');
    expect(app).toContain('view === "capture" ?');
  });
});
