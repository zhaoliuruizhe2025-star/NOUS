# CODEX TASK 005 — Lived-Experience Records

## Status

**PROPOSED REVISION — owner-approved semantics incorporated; final owner approval and an approved Git planning checkpoint are required.**

This draft is not implementation authorization. Task 005 becomes READY only after:

1. the owner gives final approval to this revised executable specification;
2. this specification's status is updated to record that approval;
3. the approved planning documents are committed and fast-forwarded to `main`; and
4. implementation begins from that resulting `main` checkpoint on a dedicated task branch.

Do not create migration 0003, domain types, repository operations, dependencies, or an implementation branch from this unapproved draft.

## Objective

Implement three current-user-owned Structured Experience records in Rust and local SQLite:

- `Memory` — a retrospectively meaningful past experience described by the user;
- `Decision` — a choice the user reports having made; and
- `Outcome` — what the user later reports happened after a Decision.

The implementation must preserve direct ownership, optional non-synthetic Situation context, required same-subject Outcome-to-Decision linkage, validated reconstruction, local durability, and the Task 004 database security boundary.

Task 005 stores user-authored lived-experience records. It does not infer meaning, causality, success, psychological patterns, or durable commitments.

## Planning provenance

- Canonical repository: `D:\Dev\NOUS`
- Planning branch: `planning/task-005-lived-experience-records`
- Drafting base: `0643ba29cce2868564d17bf036f31d0890ad9fe9` (`Merge NOUS Task 004 persistence foundation`)
- Owner-reviewed design: `docs/TASK_005_DESIGN.md`
- Feasibility review: `docs/TASK_005_FEASIBILITY.md`
- Future implementation branch after approval: `task/005-lived-experience-records`
- Future implementation base: the later owner-approved `main` checkpoint containing the approved Task 005 planning documents

Do not implement from the drafting base merely because its hash appears here.

## Owner decisions resolved during planning review

Owner review approved these Task 005 semantics:

1. one Decision may own zero to many Outcomes;
2. Memory has optional user-authored `user_meaning`;
3. Memory and Decision each have an optional same-subject Situation link;
4. Task 005 adds no semantic occurrence, decision, or outcome timestamps;
5. Memory, Decision, and Outcome are immutable create/load records with no revision tables or normal update/delete operations; and
6. Task 005 adds no direct PersonReference relationship; structured cross-entity relationships remain Task 006 work; and
7. Memory + Decision + Outcome remain together as one Task 005 and must not be split into 005A/005B.

These decisions are no longer open implementation questions. Any later material change requires explicit owner review and consistent planning-document revision; implementation must not improvise.

No new unresolved owner decision or feasibility blocker is known at this planning revision.

## Required reading

Before implementation, read completely:

1. `AGENTS.md`
2. `README.md`
3. `START_HERE.md`
4. `docs/PRODUCT.md`
5. `docs/PRINCIPLES.md`
6. `docs/SELF_MODEL.md`
7. `docs/ARCHITECTURE.md`
8. `docs/DECISIONS.md`
9. `docs/MASTER_PLAN.md`
10. `docs/ROADMAP.md`
11. `docs/TASK_SYSTEM.md`
12. `docs/DEVELOPMENT_PROTOCOL.md`
13. `docs/BACKLOG.md`
14. `docs/TASK_002_DESIGN.md` and `CODEX_TASK_002.md`
15. `docs/TASK_003_DESIGN.md` and `CODEX_TASK_003.md`
16. `docs/TASK_004_DESIGN.md`
17. `docs/TASK_004_FEASIBILITY.md`
18. `CODEX_TASK_004.md`
19. `docs/TASK_005_DESIGN.md`
20. `docs/TASK_005_FEASIBILITY.md`
21. this specification

Then inspect the current Rust domain code/tests, `src-tauri/src/persistence.rs`, migrations 0001/0002, migration registration, readiness command, frontend readiness check, capabilities, and security-boundary test. Repository code is authoritative where older planning status text is stale. A material semantic or architectural conflict is a STOP condition.

## Exact domain contract

Add these private-field public domain types using existing validated construction and serialization conventions.

### Typed IDs

```text
MemoryId(String)
DecisionId(String)
OutcomeId(String)
```

Each must reject empty or whitespace-only input through the existing typed-ID pattern. Do not add ID generation.

### Memory

```text
Memory
  id: MemoryId
  subject_id: SelfSubjectId
  situation_id: Option<SituationId>
  description: RequiredText
  user_meaning: Option<RequiredText>
```

Provide one validated constructor equivalent to:

```rust
Memory::new(id, subject_id, situation_id, description, user_meaning)
```

`description` is the user's account of the remembered past experience. `user_meaning` is optional user-authored meaning, never inferred psychology. A supplied blank value is invalid.

### Decision

```text
Decision
  id: DecisionId
  subject_id: SelfSubjectId
  situation_id: Option<SituationId>
  description: RequiredText
```

Provide one validated constructor equivalent to:

```rust
Decision::new(id, subject_id, situation_id, description)
```

`description` records the choice the user reports having made. It is not an unresolved question, option-ranking result, recommendation, or prediction.

### Outcome

```text
Outcome
  id: OutcomeId
  subject_id: SelfSubjectId
  decision_id: DecisionId
  description: RequiredText
```

Provide one validated constructor equivalent to:

```rust
Outcome::new(id, subject_id, decision_id, description)
```

`description` records what the user reports happened after the Decision. It is not causal proof, a success/failure score, or a system evaluation.

### Domain constraints

- All three records belong to the current `SelfSubject`.
- A Memory or Decision may omit `situation_id`.
- A non-null Situation link must resolve to a Situation owned by the same subject at persistence time.
- Creating a Memory or Decision must not create a Situation automatically.
- An Outcome always names a Decision and must share that Decision's subject at persistence time.
- Outcome has no Situation link in Task 005.
- A Decision may have zero, one, or multiple Outcomes.
- No record contains persistence `created_at_ms`.
- No record contains semantic time, confidence, emotional weight, importance, evaluation, inferred meaning, or third-party state.
- Serialization/deserialization must not bypass typed-ID or required-text validation.

## Conceptual boundaries

Implementation and tests must preserve:

- `Memory != Situation`: context is not the retrospective record, even if linked;
- `Observation != Memory`: a noticed/reported detail is not automatically a meaningful autobiographical memory;
- `Thought != Memory`: an interpretation is not a remembered experience;
- `Emotion != Memory`: a felt state is not the remembered account;
- `Decision != Thought`: considering or predicting is not a completed choice;
- `Outcome != evaluation`: aftermath is not judgment, causality, success, or recommendation; and
- user-reported experience is not automatically an objectively verified fact.

No automatic conversion among these types is allowed.

## Third-party boundary

Task 005 models the current user's experience only.

Another person may appear in user-authored description/meaning text as context. That does not create or imply the third party's inner state. Do not add a `PersonReference` owner, a PersonReference psychological field, or a direct lived-experience relationship table in this task.

Do not infer or persist another person's Beliefs, Values, Emotions, motives, diagnosis, traits, deception likelihood, future behavior, or relationship outcome probability.

## Persistence contract

### Migration 0003

Add exactly one production migration:

```text
src-tauri/migrations/0003_create_lived_experience_records.sql
```

Do not modify migrations 0001 or 0002. Register version 3 through the existing `tauri-plugin-sql` builder. The plugin remains the sole production migration authority.

The migration must create exactly these three tables with equivalent SQL semantics.

```sql
CREATE TABLE memories (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (
        situation_id IS NULL OR length(trim(situation_id)) > 0
    ),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    user_meaning TEXT NULL CHECK (
        user_meaning IS NULL OR length(trim(user_meaning)) > 0
    ),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id)
        REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE decisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (
        situation_id IS NULL OR length(trim(situation_id)) > 0
    ),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (id, subject_id),
    FOREIGN KEY (subject_id)
        REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE outcomes (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    decision_id TEXT NOT NULL CHECK (length(trim(decision_id)) > 0),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id)
        REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (decision_id, subject_id)
        REFERENCES decisions(id, subject_id) ON DELETE RESTRICT
);
```

`decisions` must retain `UNIQUE (id, subject_id)` even though `id` is already its primary key. SQLite requires the matching composite parent key for the scoped Outcome foreign key. Outcomes also retain their direct `subject_id -> self_subjects(id)` ownership foreign key.

Add only:

```sql
CREATE INDEX memories_subject_id_idx
    ON memories(subject_id);
CREATE INDEX memories_situation_subject_idx
    ON memories(situation_id, subject_id);
CREATE INDEX decisions_subject_id_idx
    ON decisions(subject_id);
CREATE INDEX decisions_situation_subject_idx
    ON decisions(situation_id, subject_id);
CREATE INDEX outcomes_subject_id_idx
    ON outcomes(subject_id);
CREATE INDEX outcomes_decision_subject_idx
    ON outcomes(decision_id, subject_id);
```

Then update the informational mirror:

```sql
UPDATE app_metadata SET value = '3' WHERE key = 'schema_version';
```

Do not add `UNIQUE (decision_id)`; the approved cardinality is zero-to-many. Migration 0003 must add exactly the three tables above and no relationship, Evidence, revision, PersonReference junction, or temporal table. Do not add search, tags, FTS, generic relationships, soft-delete fields, or semantic timestamps.

### Persistence timestamp

Every new table has only:

```text
created_at_ms INTEGER NOT NULL
```

It is the UTC Unix-millisecond storage creation time supplied separately by the application boundary. It is not event occurrence time, decision time, outcome time, remembered time, causal order, or psychological effective time. Keep it in row/persistence inputs, not the domain structs. No other Task 005 timestamp is authorized.

### Concrete operation surface

Extend the existing concrete repository with only:

- `create_memory` / `load_memory`;
- `create_decision` / `load_decision`;
- `create_outcome` / `load_outcome`; and
- `load_outcomes_for_decision`.

Names may follow existing Rust conventions, but the semantic families must remain explicit.

Creation operations:

- accept an already validated domain object and separate `created_at_ms`;
- use parameterized SQL;
- preserve caller-supplied canonical typed IDs;
- execute one atomic SQL insert; and
- map known structural failures to `ConstraintViolation`.

Load operations:

- accept the matching typed ID;
- return `NotFound` when absent;
- reconstruct through the public typed ID/entity constructors; and
- return `DomainReconstruction` for corrupt stored IDs or text without repair, trimming, skipping, defaulting, or converting corruption to `None`.

`load_outcomes_for_decision` returns every Outcome for the supplied Decision using exactly `ORDER BY created_at_ms ASC, id ASC`. This is deterministic storage/read order, not event chronology, causal order, outcome significance, or psychological order. If timestamps match, the canonical string ID is the tie-breaker. If the Decision does not exist, return `NotFound`; an existing Decision with no Outcomes returns an empty vector.

No Task 005 operation updates, deletes, reparents, rewrites, or revises a record. Do not copy the Belief/Value revision infrastructure.

### Connection discipline

Every new persistence operation must:

1. acquire the actual connection it will use from the existing `SharedSqlitePool`;
2. verify `PRAGMA foreign_keys = 1` through the existing `VerifiedSqliteConnection` path on that exact connection;
3. retain that connection; and
4. execute the query on it.

Do not open another pool, change the database path, enable WAL, add retry behavior, or bypass foreign-key verification.

No new multi-step transaction is required. Do not add `BEGIN IMMEDIATE` merely for symmetry; preserve the approved Task 004 transaction behavior for Belief/Value operations unchanged.

## Readiness and security boundary

Registering migration 3 requires the fixed readiness path to expect schema version `3`:

- Rust `database_status` must accept no new argument and return the same small response shape with `schemaVersion: 3`;
- `src/app/database.ts` may change only its expected fixed version from 2 to 3;
- initializing/ready/error UI behavior must remain unchanged;
- the frontend must retain no raw SQL, database URL, SQL plugin dependency, or `sql:*` capability; and
- the security regression must include `memories`, `decisions`, and `outcomes` among protected domain tables.

Do not expose Memory/Decision/Outcome commands to React in Task 005. Do not create a generic database or repository command.

## Explicit exclusions

Do not implement:

- Raw History or conversation storage;
- automatic capture/extraction or automatic Memory creation;
- Evidence or provenance entities;
- generic relationships or graph traversal;
- links to Belief, Value, Observation, Thought, Emotion, or PersonReference beyond the exact approved fields;
- inferred relationships, recurring-pattern detection, or reasoning traces;
- psychological inference, causal attribution, or Durable Self Model promotion;
- Thought-to-Belief promotion;
- Decision options, ranking, anticipated outcomes, utility, recommendation, or Life Paths;
- Outcome evaluation, scoring, causal proof, or prediction;
- confidence, emotional-weight, salience, importance, or quality scores;
- revision/history tables or correction workflow for the new records;
- update/delete/reparent operations, soft delete, restore, or privacy-deletion UX;
- semantic occurrence/effective/modification timestamps;
- response generation/routing, adaptive interaction behavior, Care, or prediction;
- final UI, forms, CRUD screens, navigation, styling, copy, or localization changes;
- backup/export, sync/cloud/mobile, accounts, or multi-user behavior;
- third-party psychological profiles; or
- any Task 006+ behavior.

Do not add generic CRUD, `Repository<T>`, a repository framework, a trait merely for style, ORM, `rusqlite`, `UnitOfWork`, generic transaction abstraction, cache, write queue, retry framework, or mocks merely for architecture.

## Dependency policy

Add no dependency.

Reuse the existing exact direct SQLx declaration:

```toml
sqlx = { version = "=0.8.6", default-features = false, features = ["sqlite", "runtime-tokio"] }
```

Keep the Rust `tauri-plugin-sql` dependency and plugin migration/preload behavior. Do not restore the frontend SQL plugin.

If a new Rust, frontend, or test dependency appears necessary, STOP and request owner review rather than adding it.

## Phase 0 — Implementation preflight

Before any implementation:

1. confirm repository `D:\Dev\NOUS`;
2. run `git status`, `git branch --show-current`, and `git log -8 --oneline`;
3. confirm `main` is clean, synchronized with its configured upstream, and contains the owner-approved Task 005 specification checkpoint;
4. record the full starting checkpoint hash;
5. confirm Task 001–004 remain historical completed work;
6. confirm this specification says **APPROVED SPECIFICATION / READY** after explicit owner approval;
7. create `task/005-lived-experience-records` from that checkpoint; and
8. STOP if any precondition differs.

Do not implement directly on `main`. Do not commit, merge, or push automatically.

## Phase 1 — Domain model only

Add the three typed IDs and domain types with focused unit tests.

Verify:

- constructor/getter shape;
- required and optional text validation;
- typed ownership/context fields;
- English, Simplified Chinese, and mixed Unicode;
- JSON round trips;
- invalid serialized IDs/text cannot bypass validation;
- no Task 006 link or third-party state; and
- no persistence metadata in domain structs.

Do not add migration/repository code until the domain diff is internally consistent with the approved specification.

## Phase 2 — Migration 0003 and readiness version

Add/register the one approved migration and advance the fixed readiness checks to version 3.

Use the real checked-in migrations in tests. Verify:

- fresh 0001 -> 0002 -> 0003;
- existing version-2 -> version-3 upgrade;
- all 13 approved domain tables (10 Task 004 plus 3 Task 005) exist after migration, alongside the existing infrastructure tables;
- exact new columns, checks, foreign keys, unique key, and indexes;
- informational schema version 3;
- migration failure withholds readiness; and
- migrations 0001 and 0002 are byte-for-byte unchanged.

Do not create another migration registry or authority.

## Phase 3 — Persistence operations and reconstruction

Add only the approved concrete operations and private row types genuinely useful for nullability/reconstruction.

Preserve exact-connection FK verification. Test:

- create/load each type;
- nullable/non-null Situation and `user_meaning` states;
- same-subject ownership success;
- missing and cross-subject FK rejection;
- duplicate typed-ID translation;
- `NotFound` loads;
- zero/multiple Outcome loading and deterministic order;
- corrupt row reconstruction; and
- no generic/update/delete/reparent API.

Do not refactor or split the existing persistence module merely because it is large. If its size becomes an actual correctness/review blocker, STOP and request review before reorganizing it.

## Phase 4 — Durability, constraints, and security regression

Complete focused integration coverage using temporary on-disk databases:

- write all three records;
- close every pool handle;
- reopen the exact same file;
- load the records and ordered Outcomes;
- verify required text, scoped FKs, cardinality, and `ON DELETE RESTRICT` through real SQL behavior; and
- confirm frontend raw-SQL protections still cover all domain tables and readiness remains fixed-purpose.

Do not add concurrency testing solely for Task 005; there is no read-modify-write sequencing. All Task 004 tests, including concurrency/history tests, must continue to pass.

## Phase 5 — Full verification and review

Run from the repository root:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run typecheck
npm run lint
npm test
npm run build
npm run tauri:check
npm run tauri:build
git diff --check
git status
```

Inspect the complete Task 005 diff against the approved design/feasibility/specification. Stop in REVIEW without committing or merging.

## Required test matrix

### A. Domain

- all three typed IDs accept valid and reject blank values;
- all three entities construct with valid required fields;
- blank required and supplied optional text is rejected;
- nullable links/meaning are preserved;
- English, Simplified Chinese, and mixed Unicode round-trip;
- deserialization cannot bypass validation; and
- no type can be owned by PersonReference.

### B. Migration

- real fresh migration sequence reaches version 3;
- real version-2 upgrade reaches version 3;
- three and only three Task 005 tables are added;
- exact columns/nullability/checks/FKs/indexes exist, including `decisions UNIQUE (id, subject_id)` and the scoped Outcome composite foreign key;
- migration failure is surfaced and readiness is withheld;
- migrations 0001/0002 remain unchanged; and
- no duplicated test-only DDL or production migration registry exists.

### C. Round trips

- Memory with and without Situation;
- Memory with and without user meaning;
- Decision with and without Situation;
- Outcome linked to Decision;
- zero, one, and multiple Outcomes for a Decision;
- Unicode content across the three types; and
- deterministic Outcome query order, including `id ASC` for equal `created_at_ms`, without treating the result as semantic chronology.

### D. Structural invariants

- primary-key collisions map to `ConstraintViolation`;
- missing subject is rejected for every type;
- same-subject Situation succeeds;
- cross-subject Situation fails for Memory and Decision;
- missing Decision fails for Outcome;
- cross-subject Decision fails for Outcome;
- same-subject Decision succeeds for Outcome;
- multiple Outcomes for one Decision succeed without a unique `decision_id` constraint;
- required-text SQL checks hold;
- `ON DELETE RESTRICT` holds where practical; and
- FK enforcement is verified on each repository-used connection, including multiple acquired connections.

### E. Reconstruction

- corrupt Memory ID, subject ID, optional Situation ID, description, and optional meaning fail explicitly where injectable;
- corrupt Decision ID, subject ID, optional Situation ID, and description fail explicitly;
- corrupt Outcome ID, subject ID, Decision ID, and description fail explicitly; and
- no corrupt value is repaired, trimmed into validity, skipped, defaulted, or converted to `None`.

### F. Durability and regression

- on-disk close/reopen preserves every new type and multiple Outcomes;
- Task 004 Belief/Value history, transaction, busy, and reconstruction tests still pass;
- `database_status` expects 3 and sanitizes failure;
- frontend contains no raw SQL/database URL/SQL plugin use;
- capabilities contain no `sql:*` permission; and
- existing initializing/ready/error behavior remains structurally unchanged.

## Expected file boundary

Exact organization may follow current module ergonomics, but do not create placeholder modules.

### Expected new files

- `src-tauri/migrations/0003_create_lived_experience_records.sql`
- `src-tauri/src/domain/lived_experience.rs`
- `src-tauri/src/domain/lived_experience_tests.rs` or tests colocated in the new module

### Expected modified files

- `src-tauri/src/domain/primitives.rs` — three typed IDs only
- `src-tauri/src/domain/mod.rs` — module/re-exports/tests
- `src-tauri/src/lib.rs` — migration 3 registration and focused registration test
- `src-tauri/src/commands.rs` — fixed expected schema version only and readiness tests
- `src-tauri/src/persistence.rs` — concrete row mapping, operation families, and focused tests
- `src/app/database.ts` — fixed expected schema version 3 only
- `tests/database-boundary.test.ts` — protect the three new table names/version boundary as needed

### Files not to modify

- `src-tauri/migrations/0001_initialize.sql`
- `src-tauri/migrations/0002_create_self_model.sql`
- Task 002/003 domain semantics or existing type fields
- Task 004 persistence semantics, transaction behavior, pool ownership, or error meanings
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`
- `package.json` and `package-lock.json`
- `src-tauri/capabilities/default.json`
- `src-tauri/tauri.conf.json`
- `src/app/App.tsx` and all UI/components/styles/localization/navigation
- approved Task 001–004 specifications or historical documents
- `docs/TASK_005_DESIGN.md`, `docs/TASK_005_FEASIBILITY.md`, and this file during implementation
- roadmap, product, principles, Self Model, decisions, backlog, response, interaction, safety, or academic documents

If another file is materially necessary, STOP and explain before broadening scope.

## STOP conditions

Stop and request owner review before changing architecture if:

- the final approved implementation checkpoint does not contain these resolved semantics;
- current domain constructors cannot express the model without semantic changes to completed types;
- schema constraints cannot enforce the approved same-subject links;
- correct persistence appears to require a second pool, WAL, retries, or a new transaction architecture;
- a new dependency appears necessary;
- migration 0001 or 0002 would need modification;
- correction requirements appear to require revision tables;
- semantic event time becomes required without an approved temporal model;
- a direct PersonReference or other cross-entity relationship becomes necessary;
- frontend raw SQL or generic commands appear necessary;
- Task 006 Evidence/relationship behavior is needed to complete Task 005;
- implementation would require inference, scoring, causal claims, or third-party state; or
- implementation evidence reveals a material scope blocker; do not split Task 005 into 005A/005B without new owner approval.

Do not silently work around a STOP condition.

## Acceptance criteria

Task 005 is ready for owner implementation review only when:

- the approved exact domain types and fields exist with validated construction;
- Memory remains distinct from Situation/Observation/Thought/Emotion;
- Decision remains distinct from Thought/recommendation;
- Outcome remains a user-reported aftermath, not evaluation or causal inference;
- every record belongs to SelfSubject and no third-party profile is created;
- optional Situation and required Outcome-to-Decision ownership are enforced by scoped FKs;
- zero-to-many Outcome cardinality works without a synthetic Outcome requirement;
- no revision/history/update/delete semantics were invented;
- migration 0003 adds exactly three tables and prior migrations remain unchanged;
- `tauri-plugin-sql` remains the sole production migration authority;
- no dependency is added;
- every repository operation uses the existing verified retained connection;
- every read reconstructs validated domain objects and rejects corruption explicitly;
- on-disk close/reopen durability passes;
- readiness advances narrowly to version 3 without reopening frontend SQL access;
- no excluded Task 006+, inference, response, Care, prediction, UI, export, sync, or account work enters the diff; and
- every required verification command passes.

## Architecture-drift audit

Before reporting completion, explicitly check for:

- another production migration authority;
- another SQLite pool or database path;
- changes to Task 004 foreign-key/transaction semantics;
- generic CRUD/repository/transaction abstractions;
- revision tables copied without approved semantics;
- semantic time inferred from `created_at_ms`;
- automatic links to Beliefs/Values or Durable Self Model promotion;
- generic relationships or Evidence;
- psychological/causal inference in domain or SQL;
- PersonReference ownership or third-party state;
- frontend raw SQL or generic persistence commands;
- dependency drift;
- Task 006+ scope; or
- UI redesign.

Any unapproved drift is a STOP condition.

## Implementation completion report

The future implementation report must include:

- repository path, task branch, and full starting checkpoint;
- files added/modified;
- exact domain types/fields and validation;
- migration 0003 registration, tables, FKs, indexes, and schema-version behavior;
- 0001/0002 unchanged hashes/evidence;
- final concrete operation surface;
- same-connection FK guarantee;
- Outcome cardinality and ordering behavior;
- row reconstruction and corruption results;
- close/reopen durability result;
- readiness/security-boundary result;
- dependency conclusion;
- Rust/frontend test counts and every verification result;
- architecture-drift audit;
- incomplete/deferred concerns;
- `git diff --stat`, untracked files, and final `git status`; and
- any architecture, privacy, or safety concern.

After implementation, stop in **REVIEW**. Do not commit, merge, begin Task 006, or continue into UI/product workflows without explicit owner instruction.
