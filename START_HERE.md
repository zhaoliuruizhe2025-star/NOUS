# START HERE

## What this package is

This folder contains the initial product and engineering specification for NOUS.

Do not ask Codex to "build the whole app" yet. The first task is deliberately limited to creating a safe, runnable development scaffold.

## Step 1 — Create the local repository folder

Create a folder for the project, for example:

```text
C:\Users\<you>\Projects\NOUS
```

Copy the contents of this specification package into that folder so that `AGENTS.md`, `README.md`, `CODEX_TASK_001.md`, and `docs/` are at the repository root.

## Step 2 — Open the folder in your coding environment

Open the NOUS folder in the Codex/Work coding environment you plan to use.

## Step 3 — Give Codex this instruction

```text
Read AGENTS.md and every document required by CODEX_TASK_001.md before making any changes.

Then execute CODEX_TASK_001.md exactly as written.

Important:
- NOUS's North Star is to help the current user gain a second perspective on themselves.
- NOUS models the current user, not unconsenting third parties.
- Do not redesign the product, Self Model, privacy boundary, or safety principles.
- Do not implement anything beyond Task 001.
- Do not add cloud services, telemetry, runtime LLM/API dependencies, or runtime network calls.
- Do not silently install system-wide dependencies or change Windows configuration.

If a required Windows development prerequisite is missing, stop before making system-level changes and tell me exactly what I need to install or configure.

When finished, run the checks required by Task 001 and give me a concise completion report. Then stop.
```

## Step 4 — Expected outcome

If your environment is already ready, Codex should:
- scaffold a Tauri 2 + React + TypeScript desktop app;
- create English / Simplified Chinese localization skeletons;
- initialize local SQLite capability;
- run/check the project;
- report what it changed.

If a prerequisite is missing, Codex should stop and tell you what is missing instead of silently changing Windows.

## Step 5 — Do not continue automatically

After Task 001, stop.

Run NOUS yourself and review the result before starting Task 002.

Bring the Codex completion report, screenshots, errors, or unfamiliar files back to the normal Chat conversation for explanation/review.

## Important

The `.md` files are the project's written specification and standing instructions. They strongly constrain Codex, but they are not a substitute for:
- tests;
- code review;
- Git history;
- task-specific prompts.

After the scaffold is verified, initialize Git (if Codex has not already done so) and make a clean first commit before implementing the Self Model.
