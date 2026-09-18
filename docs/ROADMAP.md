# NOUS Roadmap

This is the concise phase view. `MASTER_PLAN.md` is the authoritative engineering sequence, `TASK_SYSTEM.md` governs execution, and `BACKLOG.md` prevents future ideas from silently entering active work.

## Phase 0 — Specification

Status: **complete**. Task 001 delivered the approved scaffold at checkpoint `455e929`.

Goals:
- product definition;
- safety/care principles;
- Self Model;
- architecture;
- bilingual voice;
- first Codex task.

Exit criteria:
- documents reviewed;
- first scaffold task approved.

## Current planning checkpoint

Status: **complete** at `8d9770d Establish NOUS v0.2 master plan`.

The v0.2 master planning checkpoint aligned product, architecture, interaction, roadmap, task governance, and backlog boundaries.

## Phase 1 — v0.1 "Constructing a Self"

Goal: represent a changing person locally and transparently.

### 1A. Scaffold
- Tauri 2 app launches on Windows.
- React + TypeScript UI works.
- localization skeleton works.
- SQLite initializes locally.
- tests/lint/typecheck/build commands defined.
- no runtime network dependency.

Status: **COMPLETE** at `455e929`. This is historical completed work. Future changes require a new explicit task.

### 1B. Domain model
Task 002 is **COMPLETE** at `e324852 Complete NOUS Task 002 self model foundation`:
- SelfSubject / PersonReference privacy boundary;
- Observation
- Situation
- Thought
- Emotion

It includes validation, serialization, and focused tests. It does not include persistence, migrations, or UI.

The next proposed split is:

- Task 003: Belief, BeliefRevision, Value, and ValueRevision;
- Task 004: persistence foundation for approved Task 002/003 types;
- Task 005: Memory, Decision, and Outcome;
- Task 006: generic Evidence foundation and explicit relationships.

Task 003 is **COMPLETE** at `e970138 Complete NOUS Task 003 commitments and revision history`. Evidence is deferred until the lived-experience source model is broader; a Thought never automatically becomes a Belief.

### 1C. Persistence
- schema + migrations;
- transactions;
- CRUD/application services;
- history preserved.

### 1D. Minimal UI
- Today;
- create/edit basic records;
- English/Chinese switch;
- initial Inner Map;
- inspect one entity and its relationships.

### 1E. Integrity
- tests;
- backup/export design;
- error handling;
- privacy/system-access settings surface.

v0.1 explicitly excludes advanced prediction.

The current UI remains temporary functional scaffolding. Final visual identity is frozen until a dedicated UI/UX phase.

## Phase 2 — Remember, Connect, and Interact

- reliable structured capture and revision history;
- explicit evidence links and inspectable reasoning traces;
- response routing and the separate adaptive interaction strategy in `INTERACTION_MODEL.md`;
- useful-before-insightful behavior and optional `Action -> Explanation -> Reflection` strategy where appropriate;
- concrete grounding in the user's real situations and history when available.

## Phase 3 — Mirror

- temporal comparison;
- recurring patterns;
- narrative change summaries;
- user correction/rejection of interpretations;
- "Why?" evidence view.

## Phase 4 — Reasoning

- Belief Graph;
- explicit relationship rules;
- contradiction/tension detection;
- cycle-safe graph traversal;
- structured reasoning traces;
- first small, curated framework rules.

## Phase 5 — Product Experience / UI-UX

- replace the Task 001 temporary scaffold;
- establish final visual identity;
- design approved interaction flows and information hierarchy;
- establish bilingual typography and layout behavior;
- validate usability while keeping domain logic independent of presentation.

The future approved direction is a minimal conversational experience with default remembering, layered durable-model safeguards, short Bootstrap onboarding, no first-run personality report, and later experimental Self/Mirror inspection. These decisions do not authorize implementation.

This dedicated phase must be completed before Private Beta and requires separately approved tasks.

## Phase 6 — Care / Safety implementation

- Care routing behavior;
- distress/crisis interaction behavior;
- human-reviewed English and Simplified Chinese care copy;
- safety-focused tests;
- verified Care precedence over ordinary reflection, philosophy, simulation, and prediction.

This phase must be completed before Private Beta and requires separately approved tasks.

## Phase 7 — Private Beta

- small, informed, trusted tester group;
- stable local workflows;
- privacy and safety review;
- bilingual care/voice review;
- feedback without required telemetry or automatic remote collection;
- known limitations presented clearly;
- project-owner approval of the beta checkpoint.

Private Beta precedes public/open-source release.

## Long-term capability — Life Paths

- decision scenarios;
- value trade-offs;
- uncertainty;
- alternative-path generation from structured inputs;
- false-dichotomy detection where possible;
- no "best path" output.

Life Paths is deferred from the first Private Beta. This is scope reduction, not removal from the long-term NOUS product vision.

## Future research — Digital Self

- prediction from historical decisions;
- hold-out/unseen decision tests;
- calibration and confidence;
- explainability;
- prediction-vs-actual history;
- careful evaluation for bias and overconfidence.

Digital Self remains future/deferred work unless explicitly promoted. Do not start it until earlier models and tests are stable.

## Phase 8 — Public release planning

- polished bilingual UI;
- installer/release pipeline;
- Windows stable;
- macOS/Linux validation;
- import/export/backups;
- public documentation;
- privacy/security review;
- open-source license decision;
- demo/screenshots;
- GitHub public release.

## Future possibilities, not commitments

- optional local language model;
- optional external AI provider;
- encrypted sync;
- mobile companion;
- richer framework library;
- research export tools.
- Relationship Lens;
- Expression Lab;

None of these should be implemented merely because they are listed here. Their canonical status is in `BACKLOG.md`.
