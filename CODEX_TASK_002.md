# CODEX TASK 002 — Self Model Foundation

## Objective

Implement the first NOUS domain-model foundation:

- SelfSubject
- PersonReference
- Observation
- Situation
- Thought
- Emotion

This is a domain-code and unit-test task only.

Task 002 is the next task in `docs/MASTER_PLAN.md`, but it remains **NOT STARTED** until the v0.2 documentation checkpoint is reviewed and an approved Task 002 starting point is available.

## Canonical repository

Work only in:

```text
D:\Dev\NOUS
```

Do not use the previous C-drive workspace.

## Required reading

Before changing code, read:

1. `AGENTS.md`
2. `docs/PRODUCT.md`
3. `docs/PRINCIPLES.md`
4. `docs/SELF_MODEL.md`
5. `docs/ARCHITECTURE.md`
6. `docs/DECISIONS.md`
7. `docs/DEVELOPMENT_PROTOCOL.md`
8. `docs/MASTER_PLAN.md`
9. `docs/TASK_SYSTEM.md`
10. `docs/BACKLOG.md`
11. `docs/UI_BOUNDARY.md`
12. `docs/TASK_002_DESIGN.md`
13. `docs/RELATIONSHIP_LENS.md`

## Phase 0 — Access and Git preflight

Before implementation:

1. confirm the absolute repository path is `D:\Dev\NOUS`;
2. verify write access to the repository;
3. run `git status`;
4. confirm the stable branch is `main`;
5. confirm latest approved checkpoint includes:
   `455e929 Complete NOUS Task 001 desktop scaffold`;
6. confirm the v0.2 documentation checkpoint has been reviewed and approved;
7. stop if unexpected uncommitted changes exist.

Do not begin from the uncommitted planning branch. The eventual documentation checkpoint may have a later hash; `455e929` must remain in its history as the approved Task 001 checkpoint.

If repository sandbox permissions block writes, stop and request narrowly scoped access to this project only.

## Phase 1 — Create task branch

Create and switch to:

```text
task/002-self-model-foundation
```

Do not modify `main` directly.

Do not commit or merge automatically.

## Phase 2 — Inspect architecture

Inspect the Task 001 code structure.

Use the existing Rust/Tauri core architecture unless a serious conflict exists.

Prefer implementing these domain types in the Rust core/domain layer.

If that choice conflicts materially with the existing architecture:
- stop;
- explain the conflict;
- wait for approval.

## Phase 3 — Implement the domain model

Implement only:

- `SelfSubject`
- `PersonReference`
- `Observation`
- `Situation`
- `Thought`
- `Emotion`

Follow `docs/TASK_002_DESIGN.md`.

Do not implement:
- database persistence;
- migrations;
- Belief/Value/Memory/Decision/etc.;
- inference;
- NLP;
- prediction;
- relationship analysis;
- Care detection;
- final UI.

## Phase 4 — Tests

Add focused unit tests for the invariants defined in `docs/TASK_002_DESIGN.md`.

Tests should include Unicode/Simplified-Chinese content.

## Phase 5 — Verify

Run relevant:

- Rust format/check;
- Rust tests;
- existing frontend checks needed to ensure no regression;
- existing Tauri build/check if reasonably required.

Do not spend time polishing the temporary UI.

## Phase 6 — Stop for review

At completion:

1. run `git status`;
2. do not commit;
3. do not merge;
4. report:
   - repository path;
   - current branch;
   - files changed;
   - domain types created;
   - validation choices;
   - tests added;
   - commands run and results;
   - any warnings/limitations;
   - `git status`.

Then stop.

Do not begin Task 003.

Any Task 003 references in the master plan are roadmap summaries only, not implementation instructions.
