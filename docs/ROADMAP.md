# NOUS Roadmap

The roadmap intentionally separates a stable foundation from ambitious prediction features.

## Phase 0 — Specification

Status: current.

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

## Phase 1 — v0.1 "Constructing a Self"

Goal: represent a changing person locally and transparently.

### 1A. Scaffold
- Tauri 2 app launches on Windows.
- React + TypeScript UI works.
- localization skeleton works.
- SQLite initializes locally.
- tests/lint/typecheck/build commands defined.
- no runtime network dependency.

### 1B. Domain model
Implement:
- SelfSubject / PersonReference privacy boundary;
- Observation
- Situation
- Thought
- Emotion
- Belief + revisions
- Value + revisions
- Memory
- Decision
- Outcome
- Evidence

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

## Phase 2 — v0.2 "Mirror"

- temporal comparison;
- recurring patterns;
- narrative change summaries;
- user correction/rejection of interpretations;
- "Why?" evidence view.

## Phase 3 — v0.3 "Reasoning"

- Belief Graph;
- explicit relationship rules;
- contradiction/tension detection;
- cycle-safe graph traversal;
- structured reasoning traces;
- first small, curated framework rules.

## Phase 4 — v0.4 "Paths"

- decision scenarios;
- value trade-offs;
- uncertainty;
- alternative-path generation from structured inputs;
- false-dichotomy detection where possible;
- no "best path" output.

## Phase 5 — v0.5 "Digital Self"

Research/experimental stage.

- prediction from historical decisions;
- hold-out/unseen decision tests;
- calibration and confidence;
- explainability;
- prediction-vs-actual history;
- careful evaluation for bias and overconfidence.

Do not start this phase until earlier models and tests are stable.

## Phase 6 — v1.0

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

None of these should be implemented merely because they are listed here.
