# CODEX TASK 001 — Scaffold NOUS Safely

## North Star / privacy boundary

The product exists to help the current user observe themselves from a second perspective.

Do not add any feature that profiles or predicts unconsenting third parties. Other people may only be contextual references to the user's own experience.

## Objective

Create the first runnable NOUS development scaffold on the current Windows development machine.

This task is intentionally limited to infrastructure. Do **not** implement the Self Model, reasoning engine, philosophy framework logic, prediction, or Care detection logic yet.

## Required reading

Before changing anything, read:

1. `AGENTS.md`
2. `docs/PRODUCT.md`
3. `docs/PRINCIPLES.md`
4. `docs/ARCHITECTURE.md`
5. `docs/DECISIONS.md`
6. `docs/VOICE_AND_I18N.md`
7. `docs/RESPONSE_POLICY.md`
8. `docs/ROADMAP.md`

## Step 1 — Inspect the environment

Check whether the following are available:

- Git
- Node.js LTS / npm
- Rust / Cargo
- Rust MSVC toolchain suitable for Tauri on Windows
- Microsoft C++ Build Tools needed by Tauri
- WebView2 availability where practical to verify

Do not silently install system-wide dependencies.

If a required prerequisite is missing:
- stop before making system-level changes;
- report exactly what is missing;
- give the user the next installation/configuration step;
- do not use administrator elevation without explicit approval.

If the environment is ready, continue.

## Step 2 — Initialize the application

Create a Tauri 2 desktop application using:

- React
- TypeScript
- npm
- Rust/Tauri
- a structure consistent with `docs/ARCHITECTURE.md`

Do not add unnecessary UI frameworks or large dependencies.

## Step 3 — Minimal UI

Create a minimal, calm NOUS shell that proves the desktop application works.

It should contain only:
- app name: `NOUS`;
- a simple placeholder landing screen;
- language selector for English and Simplified Chinese;
- localized placeholder text.

Do not design the final visual identity yet.

## Step 4 — Localization skeleton

Set up clean localization separation.

At minimum, support:
- `en`
- `zh-CN`

Do not hard-code duplicate English/Chinese strings throughout components.

## Step 5 — Local SQLite initialization

Set up local SQLite capability using the appropriate Tauri 2 mechanism/plugin.

Requirements:
- database location should be in NOUS-managed application data/config storage, not an arbitrary repo working directory;
- no user self-model tables are required yet;
- create only the minimal database initialization/migration mechanism needed to prove local persistence works;
- do not create speculative schema for future entities in this task.

If a plugin permission/capability must be configured, use the narrowest permissions necessary.

## Step 6 — Runtime security

The scaffold must not add:
- runtime network calls;
- telemetry;
- analytics;
- cloud services;
- auto-start;
- background services;
- arbitrary shell execution;
- administrator requirements.

## Step 7 — Quality baseline

Set up or confirm commands for:
- development run;
- TypeScript type checking;
- linting if practical;
- Rust/Tauri checking/build;
- tests (a minimal smoke/unit test is sufficient at this stage).

Do not add a complicated testing stack merely for this task.

## Step 8 — Verify

Run the relevant checks and verify:

1. the desktop app launches;
2. English/Chinese switching works;
3. SQLite can initialize locally;
4. there are no intentional runtime network dependencies;
5. the project builds/checks successfully.

## Step 9 — Report, do not continue

At the end, report:

- environment versions found;
- files/directories created;
- dependencies added;
- commands used to run the app;
- checks/tests and results;
- database location/initialization approach;
- any issue or warning;
- the proposed next task.

Do **not** start implementing the Self Model after finishing this task.

## Success criteria

Task 001 is complete when NOUS is a runnable, bilingual, local-first desktop shell with verified local SQLite initialization and no core domain behavior yet.
