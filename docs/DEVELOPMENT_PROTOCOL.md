# NOUS Development Protocol

This document defines the development workflow from the v0.2 planning checkpoint onward. `TASK_SYSTEM.md` is the authoritative task-governance document.

## 1. Stable checkpoints

A task is not considered safely completed until:

1. implementation is finished;
2. required tests/checks pass;
3. architecture, product, safety, and data boundaries are respected;
4. the user reviews the result;
5. an approved Git checkpoint is created.

Codex must not automatically commit at the end of an implementation task unless explicitly instructed.

## 2. Before every implementation task

Codex must:

1. confirm the absolute repository path;
2. run `git status`;
3. confirm the current branch;
4. confirm the latest stable commit;
5. stop if unexpected uncommitted changes exist.

Changes explicitly identified by the active task as intentional are not unexpected. Classify them before proceeding and do not overwrite them.

Do not overwrite unrelated work.

## 3. Branch policy

`main` is the stable branch.

Feature work should use a dedicated task branch, for example:

```text
task/002-self-model-foundation
```

Do not implement feature work directly on `main` unless explicitly authorized.

Planning work may use a named planning branch. A roadmap entry or backlog item does not authorize implementation.

## 4. Destructive Git commands

Do not use these without explicit approval:

- `git reset --hard`
- `git clean -fd`
- force push
- history rewriting / rebase of approved history
- deleting a branch containing unmerged work

## 5. End-of-task report

Before stopping, report:

- repository path;
- branch;
- files created/modified;
- major implementation decisions;
- tests/checks run;
- results;
- `git status`;
- known limitations;
- anything intentionally deferred.

Do not merge or commit unless explicitly requested.

## 6. Codex quota resilience

Tasks should be small enough to reach a clean stopping point within one working session.

If a quota/session ends mid-task:
- preserve current files;
- do not discard partial work;
- on resume, inspect Git and unfinished changes before continuing.

## 7. Repository location

The long-term local repository is:

```text
D:\Dev\NOUS
```

The previous C-drive Codex workspace is not the canonical repository.

## 8. Planning hierarchy

```text
Vision -> Architecture -> Epics -> Phases/Milestones -> Tasks -> Git checkpoints
```

Task 001 is complete at `455e929` and remains historical. Any future change to its delivered behavior requires a new explicit task.

Task 002 is next but not started. Tasks 003 and later may appear as roadmap summaries; they are not executable instructions until separately scoped and approved.
