# NOUS v0.2 Master Plan

## 1. Purpose

This document is the engineering roadmap from the completed desktop foundation through Private Beta. It connects product vision to architecture, epics, milestones, numbered tasks, and approved Git checkpoints.

The planning hierarchy is:

```text
Vision
  -> Architecture
    -> Epics
      -> Phases / Milestones
        -> Tasks
          -> Approved Git checkpoints
```

The hierarchy is directional. A task implements an approved slice of a milestone; it does not redefine the vision or architecture. `docs/TASK_SYSTEM.md` governs task creation and completion, while `docs/BACKLOG.md` prevents future ideas from entering the active path implicitly.

## 2. Product direction

NOUS is a local-first self-awareness system that provides a second perspective on the user.

Its core loop is:

```text
Remember -> Connect -> Reflect -> Self-awareness
```

NOUS is a mirror, not an authority. It should be useful before it tries to be insightful, and it must preserve user agency, inspectability, history, privacy, and the boundary against modeling unconsenting third parties.

## 3. Engineering principles for the roadmap

- Keep the core useful offline and without a runtime LLM/API dependency.
- Build the Self Model in small, testable slices before inference or polished UI.
- Keep content understanding separate from interaction strategy.
- Make conclusions traceable to observations, model entities, rules, and limitations.
- Treat the current UI as replaceable scaffolding until a dedicated UI/UX phase.
- Prefer the user's real history over invented examples when enough relevant history exists.
- Gate Private Beta on product usefulness, safety, privacy, and bilingual quality—not feature count alone.
- Keep post-beta features in the backlog until explicitly promoted through a planning decision and a new task.

## 4. Master roadmap

Task numbers after Task 002 are planning identifiers, not implementation instructions. Scope must be approved in a dedicated task specification before work begins.

### Phase A — Foundation

#### Task 001 — Desktop scaffold

Status: **COMPLETE**

Approved checkpoint:

```text
455e929 Complete NOUS Task 001 desktop scaffold
```

Delivered the runnable Tauri 2 + React/TypeScript shell, bilingual localization skeleton, local SQLite initialization, and quality-command baseline. This is historical completed work. Do not rewrite it. Any change to it requires a new explicit task.

#### v0.2 planning checkpoint — Master plan

Status: **IN PROGRESS (documentation only)**

Purpose:

- align all repository documentation;
- establish roadmap and task governance;
- formalize adaptive interaction behavior;
- separate active work from backlog and post-beta work;
- reconfirm the narrow boundary of Task 002.

This checkpoint does not implement application code.

#### Task 002 — Self Model foundation

Status: **NEXT; NOT STARTED**

Scope is limited to:

- `SelfSubject`;
- `PersonReference`;
- `Observation`;
- `Situation`;
- `Thought`;
- `Emotion`;
- validation needed for their basic invariants;
- focused unit tests.

It excludes persistence, migrations, UI work, inference, response routing, and all later entities.

#### Task 003 — Self Model expansion (roadmap summary only)

Candidate scope:

- `Belief` and revisions;
- `Value` and revisions;
- `Memory`;
- `Decision`;
- `Outcome`;
- `Evidence`.

The exact boundaries may be split into multiple tasks after Task 002 review. This section is not an executable task specification.

#### Task 004 — Persistence foundation (roadmap summary only)

Candidate scope:

- versioned SQLite schema for approved domain entities;
- transaction boundaries;
- repositories/application services;
- preservation of history;
- focused migration and persistence tests.

No migration is authorized by this master plan alone.

### Phase B — Remember

Milestone goal: allow the user to create, inspect, revise, export, and delete a coherent local record of their own experience.

Likely task slices:

- structured capture workflow;
- history/revision behavior;
- basic local data management;
- bilingual functional UI using temporary scaffolding;
- backup/export foundations.

Exit evidence:

- local records survive restart;
- facts, interpretations, and feelings remain distinct;
- historical revisions are retained;
- user control and third-party boundaries are testable.

### Phase C — Connect

Milestone goal: connect approved records without opaque or overconfident inference.

Likely task slices:

- explicit relationships and evidence links;
- cycle-safe graph traversal;
- recurring-pattern candidates;
- structured reasoning traces;
- user confirmation, rejection, and correction.

Exit evidence:

- every surfaced connection can show why it exists;
- uncertainty and source records are available;
- a casual statement does not silently become a durable self-model conclusion.

### Phase D — Reflect and interact

Milestone goal: turn model content into useful, state-aware responses.

Likely task slices:

- response-mode routing;
- adaptive interaction strategy from `docs/INTERACTION_MODEL.md`;
- optional Action -> Explanation -> Reflection strategy where appropriate;
- concrete translation of abstract reasoning;
- use of relevant user history when available;
- bilingual voice and safety-copy review.

Content understanding and interaction strategy must remain separate. A correct interpretation can still be delivered with the wrong verbosity, warmth, directness, or question frequency.

### Phase E — Mirror

Milestone goal: help the user see change over time.

Likely task slices:

- temporal comparisons;
- narrative change summaries;
- a restrained Inner Map;
- “Why did NOUS notice this?” evidence views;
- correction/rejection workflows;
- evaluation of usefulness versus generic or intrusive reflection.

### Phase F — Product Experience / UI-UX

Milestone goal: replace the Task 001 functional scaffold with a coherent product experience suitable for Private Beta.

The Task 001 UI remains temporary scaffolding until this dedicated phase. This milestone includes:

- final visual identity;
- interaction flows for the approved core workflows;
- information hierarchy across primary, explanation, and advanced surfaces;
- bilingual English/Simplified Chinese typography and layout behavior;
- replacement of temporary forms, navigation, and verification surfaces;
- usability review without coupling domain logic to presentation.

This roadmap placement does not authorize UI implementation now. The phase requires separately scoped and approved design and implementation tasks.

### Phase G — Care / Safety implementation

Milestone goal: implement and validate the Care Layer before Private Beta.

This milestone includes:

- Care routing behavior;
- distress/crisis interaction behavior;
- human-reviewed care copy in English and Simplified Chinese;
- safety-focused tests, including ambiguous and mixed-state cases;
- verified precedence of Care over ordinary reflection, philosophy, simulation, and prediction.

This roadmap placement does not implement Care behavior now. Detection, copy, tests, and integration require separately scoped and approved tasks.

### Phase H — Private Beta readiness

Milestone goal: safely test whether NOUS provides a useful second perspective for a small, informed group.

Required gates:

- core offline workflows are stable;
- security and privacy boundaries are reviewed;
- the Product Experience / UI-UX milestone is complete;
- the Care / Safety implementation milestone is complete;
- recovery/export expectations are documented and tested to the approved scope;
- interaction behavior avoids over-analysis and generic templates;
- feedback can be collected without telemetry or automatic remote collection;
- known limitations are visible to testers;
- the project owner approves the beta checkpoint.

Private Beta precedes any public or open-source release. Public release planning begins only after beta feedback, redesign where needed, and a separate safety/privacy review.

## 5. Deferred epics

These are not on the active path to the first Private Beta unless explicitly promoted:

- Relationship Lens;
- Expression Lab;
- mature Framework Library;
- Life Paths;
- Digital Self prediction;
- optional local or external language-model integrations;
- sync, mobile, and multi-user experiences.

Their canonical status is maintained in `docs/BACKLOG.md`.

Deferring Life Paths from the first Private Beta is a scope reduction, not removal from the long-term NOUS product vision. Relationship Lens, Expression Lab, and Digital Self remain future/deferred work unless explicitly promoted through roadmap change control.

## 6. Definition of Done

A task is complete only when all of the following occur in order:

```text
implementation complete
  -> tests/checks pass
    -> architecture and product boundaries respected
      -> user review
        -> approved Git checkpoint
```

“Code written” and “build passes” are not sufficient by themselves. Until the approved Git checkpoint exists, the task remains under review rather than complete.

## 7. Current handoff

- Task 001: **COMPLETE** at `455e929`.
- v0.2 master planning checkpoint: **documentation work in progress**.
- Task 002: **NEXT; NOT STARTED**.
- Task 003 and later: **roadmap summaries only; not authorized for implementation**.
