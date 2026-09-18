# NOUS Task System

## 1. Purpose

The task system turns the product vision into small, reviewable changes without allowing roadmap ideas to bypass architecture, safety, or user approval.

## 2. Planning hierarchy

```text
Vision -> Architecture -> Epics -> Phases/Milestones -> Tasks -> Git checkpoints
```

- **Vision** defines the product outcome and non-negotiable principles.
- **Architecture** defines boundaries and technical direction.
- **Epics** group substantial product capabilities.
- **Phases/Milestones** define coherent outcomes and exit evidence.
- **Tasks** authorize a bounded unit of work.
- **Git checkpoints** record reviewed, approved completion.

Lower levels may not silently redefine higher levels.

## 3. Task numbering and records

- Implementation tasks use sequential three-digit identifiers: `001`, `002`, `003`, and so on.
- A number is never reused, even if a task is cancelled or superseded.
- Completed task specifications are historical records and are not rewritten to describe later work.
- Changes to completed work require a new task with its own scope, review, and checkpoint.
- Planning checkpoints may be named and branched independently; they do not consume an implementation task number unless they authorize implementation.

Each implementation task should have:

- a task specification using the `CODEX_TASK_` plus three-digit-number naming convention;
- supporting design documentation when needed;
- a dedicated branch such as `task/###-short-name`;
- an explicit starting checkpoint;
- a completion report.

## 4. Idea Intake / Roadmap Change Control

A new idea from product discussion must first be classified as one of:

- **current-task refinement** — clarifies the approved outcome without materially expanding its scope;
- **active-roadmap candidate** — may belong in an already approved milestone but still requires a planning decision;
- **backlog candidate** — potentially valuable, but not part of the active path;
- **research question** — requires investigation or evidence before product commitment;
- **rejected/out-of-scope idea** — conflicts with the product, safety boundaries, or current direction, or is not worth pursuing.

A conversation, suggestion, or promising idea does not automatically create a new implementation task. Record or classify it first.

Promotion from `BACKLOG.md` into the active roadmap requires an explicit planning decision and corresponding roadmap update. Even after promotion, implementation requires a bounded task specification and approval under this task system.

## 5. Creating a task

A task specification must state:

1. objective and user-visible or architectural outcome;
2. required reading;
3. exact in-scope work;
4. explicit exclusions;
5. architectural, privacy, safety, and data-integrity constraints;
6. validation and test expectations;
7. starting branch/checkpoint assumptions;
8. review and stop conditions.

A roadmap bullet is not implementation authorization.

## 6. Scope and splitting

A task should be small enough to:

- finish in a reviewable working session or clean sequence of resumable sessions;
- produce one coherent outcome;
- test the core logic and relevant edge cases;
- avoid mixing unrelated domain, persistence, interaction, and visual-design changes.

Split a proposed task when:

- it crosses multiple architectural layers without a necessary end-to-end reason;
- parts have different safety or data-migration risks;
- review would be clearer with independent checkpoints;
- completion depends on unresolved product decisions;
- it includes both active-roadmap work and backlog work.

Do not split so narrowly that a task cannot demonstrate a meaningful, testable result.

## 7. Starting work

Before an implementation task:

1. confirm the canonical repository path;
2. run `git status`;
3. confirm `main` and the latest approved checkpoint;
4. classify any existing changes as expected or unexpected;
5. read the task and required governing documents;
6. create the dedicated task branch;
7. inspect existing code and tests before adding abstractions.

Unexpected changes require a stop. Expected planning/specification files may remain when the task explicitly identifies them.

## 8. Task states

- **PROPOSED** — appears only as a candidate or roadmap summary.
- **READY** — scope and starting checkpoint are approved.
- **IN PROGRESS** — authorized work has begun on its task branch.
- **REVIEW** — implementation and checks are complete; user approval is pending.
- **COMPLETE** — user review is complete and an approved Git checkpoint exists.
- **BLOCKED** — work cannot continue without a named decision or prerequisite.
- **DEFERRED** — removed from the active path and returned to the backlog.
- **CANCELLED/SUPERSEDED** — retained as history with the reason recorded.

## 9. Definition of Done

A task reaches **COMPLETE** only after:

1. implementation is complete within scope;
2. required tests and checks pass;
3. architecture, principles, safety, and data-integrity constraints are respected;
4. the user reviews the result;
5. the user approves a Git checkpoint and that checkpoint is created.

Do not auto-commit or auto-merge unless explicitly instructed.

## 10. Review and completion report

The report must include:

- repository path and branch;
- files added/modified;
- major decisions and assumptions;
- tests/checks and results;
- anything incomplete or deferred;
- architecture, privacy, data, or safety concerns;
- final `git status`;
- suggested next task without starting it.

## 11. Checkpoint rules

- `main` represents approved stable history.
- Feature work occurs on dedicated branches unless explicitly authorized otherwise.
- A checkpoint message should identify the task and outcome.
- Do not rewrite or squash away approved historical task checkpoints without explicit approval.
- A later task may amend completed behavior, but must reference the earlier checkpoint and explain the change.

Task 001 is complete at:

```text
455e929 Complete NOUS Task 001 desktop scaffold
```

Task 002 is complete at `e324852 Complete NOUS Task 002 self model foundation`. Task 003 has an executable specification and remains **PROPOSED** until the owner reviews and approves its scope and starting checkpoint; only then may it become **READY**.
