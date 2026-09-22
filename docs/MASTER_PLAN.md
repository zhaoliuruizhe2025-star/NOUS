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

Task numbers after Task 003 are planning identifiers, not implementation instructions. Scope must be approved in a dedicated task specification before work begins.

### Phase A — Foundation

#### Task 001 — Desktop scaffold

Status: **COMPLETE**

Approved checkpoint:

```text
455e929 Complete NOUS Task 001 desktop scaffold
```

Delivered the runnable Tauri 2 + React/TypeScript shell, bilingual localization skeleton, local SQLite initialization, and quality-command baseline. This is historical completed work. Do not rewrite it. Any change to it requires a new explicit task.

#### v0.2 planning checkpoint — Master plan

Status: **COMPLETE** at `8d9770d Establish NOUS v0.2 master plan`

Purpose:

- align all repository documentation;
- establish roadmap and task governance;
- formalize adaptive interaction behavior;
- separate active work from backlog and post-beta work;
- reconfirm the narrow boundary of Task 002.

This checkpoint does not implement application code.

#### Task 002 — Self Model foundation

Status: **COMPLETE** at `e324852 Complete NOUS Task 002 self model foundation`

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

#### Task 003 — Commitments and revision history

Status: **COMPLETE** at `e970138 Complete NOUS Task 003 commitments and revision history`

Scope:

- `Belief`;
- `BeliefRevision`;
- `Value`;
- `ValueRevision`;
- required identifiers, bounded user-entered endorsement/importance values, positive revision-number validation, concrete user-authored revision origins, validation, serialization, and focused tests.

It excluded Evidence, Memory, Decision, Outcome, persistence, migrations, reasoning, UI, and all inference. Cross-revision uniqueness and strict sequencing remain Task 004 history/persistence/application responsibilities unless a separately approved aggregate abstraction is introduced. See `TASK_003_DESIGN.md` and `CODEX_TASK_003.md`.

#### Task 004 — Persistence foundation (roadmap summary only)

Candidate scope:

- versioned SQLite schema for approved domain entities;
- transaction boundaries;
- repositories/application services;
- preservation of history;
- focused migration and persistence tests.

No migration is authorized by this master plan alone.

#### Task 005 — Lived-experience records (roadmap summary only)

Candidate scope:

- `Memory`;
- `Decision`;
- `Outcome`;
- their direct, non-synthetic links to the current user and to each other.

Decision and Outcome belong together: a Decision may have no Outcome yet, but an Outcome must reference a Decision. Memory remains distinct from Situation. This section is not an executable task specification.

#### Task 006 — Evidence and explicit relationships (roadmap summary only)

Candidate scope:

- generic Evidence foundation;
- explicit source/target relationship rules across approved self-model entities;
- provenance and user-confirmation boundaries;
- focused traceability tests.

This task must not become a reasoning or inference engine merely by adding links.

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

Future design must follow [REASONING_MODEL.md](REASONING_MODEL.md) for evidence-grounded Pattern/Candidate reasoning, model validation, and reasoning integrity. These requirements do not start Phase C or authorize new numbered tasks.

The Owner-approved [Task 012 Connect architecture](TASK_012_DESIGN.md) fixes Candidate as a reviewable possible recurring structure, Hypothesis as a reasoning role rather than a durable entity, and Pattern as a validated, explicitly user-accepted, context- and time-bounded durable interpretation. Its first recommended implementation slice, Task 013 Transient Candidate Review Foundation, is user-initiated: the user proposes wording and selects current Situation, Observation, or Thought records, can mark evidence roles and adjust claim/scope, and retains no Candidate, Pattern, or Hypothesis after the review. Task 013 still needs its own bounded scope and approval before implementation. Automatic detection and durable derived persistence are not authorized by Task 012.

Likely task slices:

- explicit relationships and evidence links;
- cycle-safe graph traversal;
- evidence-grounded Pattern/Candidate interpretations with model validation, material counterevidence, and plausible alternatives;
- bounded context and temporal scope, including later evidence weakening or replacing an interpretation;
- faithful reasoning provenance/traces that connect conclusions to the actual evidence and major reasoning factors;
- user confirmation, rejection, and correction.

Exit evidence:

- every surfaced connection can show why it exists;
- uncertainty and source records are available;
- important interpretations and reasoning steps preserve premise types and exact revision context without treating duplicate records as independent corroboration, under `REASONING_MODEL.md`;
- a casual statement does not silently become a durable self-model conclusion.

### Phase D — Reflect and interact

Milestone goal: turn model content into useful, state-aware responses.

Likely task slices:

- response-mode routing;
- adaptive interaction strategy from `docs/INTERACTION_MODEL.md`;
- optional Action -> Explanation -> Reflection strategy where appropriate;
- concrete translation of abstract reasoning;
- Response Contract and question/goal alignment, including dependencies among explicit sub-questions;
- selection of Self Model history through the relevance gate;
- requested advice based on validated reasoning, with faithful user-facing explanations and preserved agency;
- bilingual semantic consistency of conclusions, uncertainty, advice, scope, and safety priority, alongside voice and safety-copy review.

Content understanding and interaction strategy must remain separate. A correct interpretation can still be delivered with the wrong verbosity, warmth, directness, or question frequency.

Exit evidence must show that responses answer the Primary Ask and explicit sub-questions, omit unrelated personal history, preserve ordinary task needs when emotion is present, and retain equivalent reasoning semantics in English and Simplified Chinese. Apply [REASONING_MODEL.md](REASONING_MODEL.md) with existing Care/Safety precedence; implementation still requires separately approved tasks.

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

The future core flow includes minimal conversational capture, default remembering with layered durable-model safeguards, short conversational Bootstrap Self Model onboarding, and no first-run personality report. These are approved design decisions, not authorization to implement UI, onboarding, or response behavior.

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
- v0.2 master planning checkpoint: **COMPLETE** at `8d9770d`.
- Task 002: **COMPLETE** at `e324852`.
- Task 003: **COMPLETE** at `e970138`.
- Task 004 and later: **roadmap summaries only; not authorized for implementation**.
