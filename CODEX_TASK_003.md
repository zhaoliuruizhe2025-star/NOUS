# CODEX TASK 003 — Commitments and Revision History

## Objective

Implement the next narrow NOUS Self Model domain slice in Rust:

- `Belief`;
- `BeliefRevision`;
- `Value`;
- `ValueRevision`;
- required typed identifiers and value objects;
- validation, serialization, and focused unit tests.

This task creates user-owned commitment and revision-history types only. It does not create persistence, a reasoning engine, or product workflows.

Task 003 is **not authorized to begin** until the owner reviews and approves this specification and its starting checkpoint.

## Canonical repository

Work only in:

```text
D:\Dev\NOUS
```

## Required reading

Before changing code, read:

1. `AGENTS.md`
2. `docs/PRODUCT.md`
3. `docs/PRINCIPLES.md`
4. `docs/MASTER_PLAN.md`
5. `docs/TASK_SYSTEM.md`
6. `docs/SELF_MODEL.md`
7. `docs/TASK_003_DESIGN.md`
8. `docs/ARCHITECTURE.md`
9. `docs/DECISIONS.md`
10. `docs/DEVELOPMENT_PROTOCOL.md`
11. `docs/UI_BOUNDARY.md`
12. this specification

Inspect the existing Task 002 Rust domain code and tests before introducing any abstraction.

## Phase 0 — Access and Git preflight

Before implementation:

1. confirm the absolute repository path is `D:\Dev\NOUS`;
2. run `git status`, `git branch`, and `git log -3 --oneline`;
3. confirm the current stable branch is `main`;
4. confirm the working tree is clean;
5. print `git log -3 --oneline` and confirm that the latest owner-approved stable `main` checkpoint contains this approved `CODEX_TASK_003.md` specification and the Task 003 planning state;
6. report the actual starting commit hash;
7. confirm Task 001 and Task 002 remain historical completed work;
8. confirm this specification has received explicit owner approval;
9. stop if unexpected changes exist or if any precondition is not true.

If repository write access is blocked, request only narrowly scoped access to `D:\Dev\NOUS`.

## Phase 1 — Create task branch

Create and switch to:

```text
task/003-commitments-revision-history
```

Create it from the latest owner-approved stable `main` checkpoint that contains the approved `CODEX_TASK_003.md` specification and Task 003 planning state. Do not modify `main` directly. Do not commit or merge automatically.

## Phase 2 — Architecture placement

Inspect the existing `src-tauri/src/domain/` structure and place the model in the Rust core/domain layer. Reuse Task 002 conventions where they fit.

Keep the model independent from temporary UI components, persistence, and application services. Do not duplicate the model in TypeScript.

If the existing architecture materially conflicts with these requirements, stop and report the conflict before changing architecture. Do not add a new dependency unless it is absolutely required; if one appears necessary, stop and explain why.

## Phase 3 — Implement only the approved domain model

### 3.1 Belief and BeliefRevision

A `Belief` is a relatively durable proposition that the current `SelfSubject` endorses or historically endorsed. It is not an objective fact, a diagnosis, a system inference, or a `Thought`.

`Belief` is the identity and ownership anchor. It carries its typed identity and `SelfSubject` ownership. Its changing proposition and optional user-entered endorsement belong to append-only `BeliefRevision` records.

`BeliefRevision` must include only the identifiers and revision-specific data required by the approved design, including:

- its typed identity;
- `BeliefId`;
- `RevisionNumber`;
- non-empty proposition text;
- optional `BeliefEndorsement`;
- optional non-empty change note where supplied;
- `RevisionOrigin`.

Do not put `subject_id` on `BeliefRevision`; it inherits ownership through `BeliefId`.

### 3.2 Belief endorsement

Use an optional, user-entered `BeliefEndorsement` value in `0..=100`. `BeliefEndorsement` must be a semantically distinct public domain type. It may share internal bounded-value validation with other types where appropriate, but it must not reuse `ThoughtConfidence` or a public generic `Percentage` type.

It means how strongly the user says they hold the proposition at that revision. It is not truth probability, system confidence, evidence strength, or prediction confidence.

`None` is valid. `0` is valid historical information and must not be rejected or silently interpreted as a judgment about the user.

### 3.3 Value and ValueRevision

A `Value` is an enduring orientation or priority that matters to the current `SelfSubject`. It is not a proposition that is true or false.

`Value` is the identity and ownership anchor. It carries its typed identity and `SelfSubject` ownership. Its changing label and optional user-entered importance belong to append-only `ValueRevision` records.

`ValueRevision` must include only the identifiers and revision-specific data required by the approved design, including:

- its typed identity;
- `ValueId`;
- `RevisionNumber`;
- non-empty user-facing label;
- optional `ValueImportance`;
- optional non-empty change note where supplied;
- `RevisionOrigin`.

Do not put `subject_id` on `ValueRevision`; it inherits ownership through `ValueId`.

### 3.4 Value importance

Use an optional, user-entered `ValueImportance` value in `0..=100`. `ValueImportance` must be a semantically distinct public domain type. It may share internal bounded-value validation with other types where appropriate, but it must not reuse `ThoughtConfidence` or a public generic `Percentage` type.

It means user-reported salience or priority. It is not truth confidence, moral worth, system ranking, or recommendation strength. `None`, `0`, and `100` are valid.

### 3.5 RevisionNumber and RevisionOrigin

`RevisionNumber` must reject `0` and accept only positive values.

Task 003 defines local domain representations and invariants only. It must not claim to enforce that every Belief or Value already has an initial revision in a repository, that a revision parent ID refers to an existing persisted parent, history-wide revision-number uniqueness, or strict global sequencing. The product rule remains that every canonical initial state is represented by an initial revision. Enforcement of initial-revision existence, referential integrity, uniqueness, and persisted-history sequencing belongs to Task 004 application/persistence logic. Do not introduce an aggregate or persistence layer in this task.

`RevisionOrigin` must contain exactly these variants:

- `InitialUserEntry` — the first canonical user-authored state;
- `UserUpdate` — the user's actual Belief or Value changed over time;
- `UserCorrection` — the prior stored representation was inaccurate, mistyped, or otherwise incorrectly recorded.

System inference must not be a `RevisionOrigin` variant. Future system proposals are separate traceable derived artifacts and require user confirmation before becoming canonical revisions.

### 3.6 History, identity continuity, and boundaries

The product rule is that every canonical initial state is represented by an initial revision. Later canonical changes create new revisions; old revisions are never mutated or overwritten. Task 003 represents these records locally but does not enforce the existence of a complete repository history or a persisted parent relationship.

A revision may capture rewording, refinement, changed endorsement/importance, a user correction, or evolution of the same underlying commitment. A materially different semantic commitment must be represented as a new `Belief` or `Value`. Do not implement automatic semantic identity detection; ambiguous continuity requires future user confirmation.

`Observation` is not `Evidence`. Thoughts, Observations, Memories, Decisions, and Outcomes must not automatically create or update Beliefs or Values. No inference engine exists in this task.

All Task 003 entities belong only to the current `SelfSubject`. Do not attach Beliefs, Values, revisions, endorsement, importance, motives, diagnoses, psychological state, or predictions to `PersonReference`.

## Explicit exclusions

Do not implement:

- SQLite schema, migrations, repositories, persistence, or application services;
- `Evidence`, `Memory`, `Decision`, or `Outcome`;
- graph relationships or cross-entity links;
- inference, contradiction detection, automatic Thought-to-Belief promotion, NLP extraction, or prediction;
- response routing, Interaction Engine, Care detection, Relationship Lens, or Expression Lab;
- UI work, TypeScript domain models, runtime LLM/API calls, telemetry, or runtime network dependencies;
- a revision-history aggregate, persistence layer, or any mechanism that claims initial-revision existence, persisted referential integrity, history-wide revision ordering/uniqueness, or strict global sequencing.

Do not modify the temporary UI, database migrations, or completed Task 001/002 history.

## Phase 4 — Focused tests

Add focused Rust unit tests covering at least:

- valid Belief plus initial revision;
- valid Value plus initial revision;
- empty/whitespace rejection for required text;
- identifier validation;
- `RevisionNumber` rejecting `0`;
- endorsement accepting `0` and `100`, and rejecting values above `100`;
- importance accepting `0` and `100`, and rejecting values above `100`;
- optional endorsement and importance accepting `None`;
- `BeliefEndorsement` and `ValueImportance` remaining distinct public domain types rather than aliases of `ThoughtConfidence` or a public generic percentage type;
- every `RevisionOrigin` variant serializing and deserializing correctly;
- Chinese, English, and mixed-language content;
- multiple revisions coexisting without mutation;
- `UserUpdate` and `UserCorrection` remaining distinguishable;
- JSON round trips;
- invalid serialized values being unable to bypass validation;
- revision records not containing `subject_id`;
- the structural third-party boundary: Task 003 entities cannot belong to `PersonReference`.

Keep tests focused on the domain model. Do not add persistence, UI, or inference tests.

## Phase 5 — Verify

Run the relevant existing checks, including:

- `cargo fmt --check`;
- `cargo check`;
- `cargo test`;
- `cargo clippy --all-targets -- -D warnings`;
- relevant existing frontend/build regression checks needed to ensure Task 001 has not regressed;
- `git diff --check`.

## Phase 6 — Stop for review

After implementation and verification:

1. run `git diff` and `git status`;
2. do not commit;
3. do not merge;
4. do not start Task 004;
5. report repository path, branch, files changed, domain structure, validation/invariant decisions, tests, commands/results, architectural decisions, warnings or unresolved questions, and final Git status.

Stop in **REVIEW** state and wait for user approval.
