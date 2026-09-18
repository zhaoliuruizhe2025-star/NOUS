# Task 004 Design — Persistence Foundation

## 1. Status, objective, and boundary

**Status: APPROVED ARCHITECTURE — design checkpoint only.** This approves the architecture design but does not authorize Task 004 implementation, migrations, dependencies, runtime behavior, or `CODEX_TASK_004.md`.

Task 004 will safely, locally, and testably persist the approved Task 002 and Task 003 Self Model domain entities in SQLite while preserving domain validation, historical integrity, migration safety, and clear architectural boundaries.

It persists only:

- `SelfSubject`, `PersonReference`, `Situation`, `Observation`, `Thought`, and `Emotion`;
- `Belief` and `BeliefRevision`;
- `Value` and `ValueRevision`.

It excludes Raw History/conversations, onboarding, Academic Foundation/frameworks, response routing, derived patterns, Memory, Decision, Outcome, Evidence, and a generic relationship graph. Persistence stores approved records; it does not decide whether a statement becomes psychologically important, infer patterns, or promote a Thought into a Belief.

## 2. Proposed architecture

```text
UI / Tauri command surface
        -> application service
        -> focused repository interface
        -> SQLite repository implementation and row mappers
        -> validated Rust domain objects
```

The frontend must not receive raw SQL write authority for Self Model data. Task 004 implementation must inspect the existing Tauri SQL plugin and capability configuration and ensure React/frontend code cannot perform arbitrary `INSERT`, `UPDATE`, or `DELETE` operations against Self Model domain tables. The current Task 001 `app_metadata` read is infrastructure-only and is not a domain-write precedent. The exact capability change remains an implementation detail, but domain persistence must be reachable only through Rust command/application operations that preserve the boundaries below.

### 2.1 Responsibilities

| Layer | Responsibility | Must not do |
| --- | --- | --- |
| Domain | Typed IDs, required text, bounded values, positive `RevisionNumber`, revision-origin semantics | Know SQLite rows, migrations, or history-wide sequencing |
| Application service | Select and orchestrate the user-intent operation, validate application inputs, and translate errors for callers | Own SQL transaction mechanics, infer psychology, or duplicate domain validation |
| Repository boundary | Express intention-revealing persistence operations and guarantee their atomic persistence outcome | Offer table-shaped generic CRUD or expose partial multi-record workflows |
| SQLite repository | Own `BEGIN` / `COMMIT` / `ROLLBACK`, execute SQL, map rows, enforce structural/history constraints, and reconstruct validated domain objects | Define psychological meaning or expose transaction mechanics to callers |
| UI/command surface | Collect/display data and call application operations | Construct SQL or bypass service invariants |

Use explicit row/mapping types (for example `BeliefRow` and `BeliefRevisionRow`) between SQLite and domain objects. Row metadata such as `created_at_ms` is not silently added to Task 002/003 domain structs. Mapping into a domain object must reconstruct its typed IDs and value objects through the existing validated constructors.

### 2.2 Repository shape

Task 004 should use a narrow repository boundary and a concrete `SqliteSelfModelRepository`; it is not a generic ORM or generic repository framework. The application service selects and orchestrates a user-intent operation, while the repository operation guarantees persistence atomicity. Whether this boundary needs a Rust trait is an implementation detail: the concrete repository is sufficient unless implementation or testing demonstrates a real need for a trait. The persistence API should be limited to operations actually needed by Task 004, such as:

- `create_subject` and create/load operations for the independent Task 002 records;
- create/load operations for `PersonReference`, `Situation`, `Observation`, `Thought`, and `Emotion`;
- `create_belief_with_initial_revision` and `append_belief_revision`;
- `load_belief_history` in ascending revision-number order;
- `create_value_with_initial_revision` and `append_value_revision`;
- `load_value_history` in ascending revision-number order.

There is no normal `insert_belief`, `insert_value`, `update_revision`, `delete_revision`, or unrestricted table CRUD API. Do not introduce generic repositories, mocks merely for architectural style, `UnitOfWork`, generic transaction frameworks, caches, or ORM abstractions.

## 3. Schema proposal

All canonical domain IDs are stored as `TEXT` and remain the IDs exposed by the domain/application layers. SQLite rowids, if SQLite creates them internally, are not domain identity. ID generation is not yet defined; implementation should accept validated typed IDs from its caller and treat collisions as explicit persistence errors rather than inventing a new ID subsystem.

Every Task 004 table receives only one persistence timestamp: `created_at_ms INTEGER NOT NULL`, a UTC Unix-millisecond creation time supplied by the application clock. It records storage creation, not occurrence, observation, inference, effective, or modification time. Revision ordering is by `revision_number`, never this timestamp.

| Table | Domain fields | Persistence-only field | Key constraints |
| --- | --- | --- | --- |
| `self_subjects` | `id`, `display_name` | `created_at_ms` | `PRIMARY KEY (id)`; nonempty text check |
| `person_references` | `id`, `subject_id`, `display_name`, `relationship_label`, `context_notes` nullable | `created_at_ms` | PK; FK subject; required-text checks |
| `situations` | `id`, `subject_id`, `description` | `created_at_ms` | PK; FK subject; `UNIQUE (id, subject_id)` for scoped links |
| `observations` | `id`, `subject_id`, `situation_id` nullable, `content` | `created_at_ms` | PK; FK subject; nullable composite FK `(situation_id, subject_id)` to `situations`; required-text check |
| `thoughts` | `id`, `subject_id`, `situation_id` nullable, `content`, `confidence` nullable | `created_at_ms` | same scoped situation FK; `confidence IS NULL OR confidence BETWEEN 0 AND 100` |
| `emotions` | `id`, `subject_id`, `situation_id` nullable, `label`, `intensity` | `created_at_ms` | same scoped situation FK; `intensity BETWEEN 0 AND 100` |
| `beliefs` | `id`, `subject_id` | `created_at_ms` | PK; FK subject |
| `belief_revisions` | `id`, `belief_id`, `revision_number`, `proposition`, `endorsement` nullable, `change_note` nullable, `origin` | `created_at_ms` | PK; FK belief; `UNIQUE (belief_id, revision_number)`; positive revision; bounded endorsement; origin check |
| `values` | `id`, `subject_id` | `created_at_ms` | PK; FK subject |
| `value_revisions` | `id`, `value_id`, `revision_number`, `label`, `importance` nullable, `change_note` nullable, `origin` | `created_at_ms` | PK; FK value; `UNIQUE (value_id, revision_number)`; positive revision; bounded importance; origin check |

Required-text checks should reject empty/whitespace-only `TEXT` values, using a SQLite expression such as `length(trim(column)) > 0` where SQLite semantics align with the Rust rule. The mapper is still authoritative for reconstructing Unicode/text validity and all domain types; SQL checks are a second safety layer, not a replacement.

`origin` is constrained to the exact persisted representation of `InitialUserEntry`, `UserUpdate`, and `UserCorrection`. Persisted history adds a Task 004 invariant: revision number `1` must use `InitialUserEntry`, while every revision number greater than `1` must use only `UserUpdate` or `UserCorrection`. A row-level SQLite `CHECK` can enforce this relationship in addition to repository/application validation. Task 003 local domain values remain unchanged, and this responsibility does not move into `RevisionNumber`. The design does not add a system-inference origin. Nullable `situation_id` preserves the existing Task 002 model; when supplied, the composite foreign key prevents a record from pointing to another subject’s Situation.

### 3.1 Deletion policy

Use `ON DELETE RESTRICT` for the relationships above, including subject ownership, revision parents, and non-nullable relationship edges. Task 004 exposes no normal deletion API. This prevents a casual deletion from silently erasing history or nulling historical links.

Privacy deletion remains a required future product concern, distinct from normal psychological revision. A later explicitly scoped deletion workflow can perform a deliberate, user-authorized transaction in dependency order. `RESTRICT` keeps that future option possible without prematurely choosing cascades, soft deletion, restoration, or granular deletion UX.

### 3.2 SQLite enforcement

Every connection must enable and verify `PRAGMA foreign_keys = ON` (or the equivalent connection configuration) before domain operations. SQLite supplies structural protection through primary keys, foreign keys, `NOT NULL`, `UNIQUE`, and bounded/enum checks. It must not encode psychological conclusions, semantic continuity, pattern detection, or Thought-to-Belief promotion.

## 4. Transaction and revision strategy

### 4.1 Creation is atomic

Persisted Beliefs and Values are never empty anchors. The repository operations `create_belief_with_initial_revision` and `create_value_with_initial_revision` guarantee atomic persistence and own the SQLite transaction mechanics:

```text
BEGIN IMMEDIATE
  insert belief/value anchor
  insert revision number 1 with InitialUserEntry
COMMIT
```

If either insert or mapping/validation step fails, the concrete SQLite repository rolls back the entire transaction. Revision number `1` must use `InitialUserEntry`; any other origin is rejected. The application service selects/orchestrates this user-intent operation and accepts the initial revision content, but it does not manage `BEGIN`, `COMMIT`, or `ROLLBACK` and does not expose a normal two-step anchor-then-revision route.

### 4.2 Appending revisions

The repository operations `append_belief_revision` and `append_value_revision` also guarantee atomic persistence through one repository-owned write transaction:

```text
BEGIN IMMEDIATE
  verify the parent anchor exists
  read MAX(revision_number) for that parent
  derive next positive number
  insert the new immutable revision
COMMIT
```

The repository persistence operation—not `RevisionNumber`, the application service, or the caller—derives the number. `RevisionNumber` continues to validate only positivity. Append operations must reject `InitialUserEntry` and accept only `UserUpdate` or `UserCorrection`. The database’s unique parent/revision constraint and origin/revision-number `CHECK` remain independent final guards.

`BEGIN IMMEDIATE` is the proposed SQLite write mode owned by the concrete repository: it acquires the write reservation before reading the maximum, so concurrent local writers cannot both calculate the same next number. This is appropriate correctness-first behavior for a modest single-user desktop store. `SQLITE_BUSY`, uniqueness violations, missing parents, and transaction failures surface as explicit persistence errors; no hidden best-effort overwrite or arbitrary caller retry is allowed. No `UnitOfWork` or generic transaction abstraction is needed.

History loading orders by `revision_number ASC`. Updates and corrections append revisions; no normal operation updates or deletes a canonical historical revision in place.

## 5. Reading, errors, and reconstruction

SQLite rows are untrusted input on read. The mapper must build typed IDs, `RequiredText`-backed entities, percentages, and `RevisionNumber` through validated domain constructors/value objects. Invalid, corrupt, or incompatible rows return an explicit `PersistenceError::DomainReconstruction` (with entity/field context safe for local diagnostics), not an invalid domain object or a silently skipped record.

The implementation should define a focused error family covering at least:

- `NotFound` for expected parent/record absence;
- `ConstraintViolation` for structural conflicts such as duplicate revisions;
- `DomainReconstruction` for invalid stored values;
- `Storage` for SQLite I/O/transaction failures;
- `Migration` for startup upgrade failures.

Errors should preserve a diagnostic cause for logs/tests without turning raw storage details into user-facing psychological claims.

## 6. Migration strategy

Task 001 currently registers migration version 1 (`0001_initialize.sql`) through `tauri-plugin-sql`, creating `app_metadata` and its `schema_version` marker. Task 004 must preserve that migration unchanged and append a next numbered migration (expected `0002_*`) through the existing migration registration mechanism.

The plugin’s applied-migration bookkeeping is the execution authority. `app_metadata.schema_version` is legacy/application metadata, not an independent migration controller; Task 004 may update it within the new migration only to keep it an accurate informational mirror. The design must not create competing migration histories.

On a fresh database, the registered migration sequence runs version 1 and then version 2 to reach the current schema. On an existing version-1 database, only the new migration runs. Migration SQL is append-only once approved/released: later schema changes add later migrations rather than edit an applied file. Startup must surface a migration failure and withhold database readiness; it must not silently continue with a partial schema or attempt destructive recovery.

## 7. Testing strategy for implementation

Task 004 implementation must add focused persistence tests; this design adds none. Prefer temporary, on-disk SQLite databases as the primary integration fixture because they verify migration application, close/reopen durability, and connection-level foreign-key configuration. In-memory databases may support isolated mapper/repository tests, but must not replace the on-disk migration/reopen suite.

The required test matrix includes:

- empty database -> migrations -> expected schema; version-1 upgrade -> current schema; migration failure propagation;
- create and reload every in-scope entity, including Unicode and nullable values;
- valid domain reconstruction and explicit failure for injected corrupt/incompatible rows;
- atomic Belief + initial revision and Value + initial revision creation, including rollback after a forced second-step failure;
- append sequencing, ordered history loading, duplicate revision rejection, and missing-parent rejection;
- later revision leaves prior revision unchanged, for both `UserUpdate` and `UserCorrection`;
- database close/reopen persistence;
- FK, required-text, bounded-percentage, positive revision, and origin constraints;
- concurrent/competing append behavior sufficient to show no duplicate canonical revision number is committed.

## 8. Dependency and implementation questions

The present dependency set includes `tauri-plugin-sql`, which supplies the Task 001 migration registration and frontend SQL bridge, but no repository implementation or test database interface exists in the Rust code. Its sufficiency for transactional Rust repositories and isolated persistence tests must be verified during Task 004 implementation. The existing capabilities currently permit database loading and selection for the infrastructure check; implementation must inspect the effective generated permissions and ensure no frontend capability grants arbitrary Self Model domain writes.

If the plugin does not expose an appropriate Rust-side transactional SQLite API, an additional narrowly scoped Rust SQLite dependency may be needed. Do not add it in this design task. That choice must be justified in the executable Task 004 specification after a small implementation feasibility check; no new ORM, cache, background queue, sync, or cloud abstraction is approved.

## 9. Explicit deferrals

- Raw History/conversation storage, onboarding sessions, and response-routing data;
- Memory, Decision, Outcome, Evidence, graph relationships, and derived patterns;
- Self Model visibility and Academic Foundation persistence;
- full privacy-deletion product design, soft deletion, restoration, and granular deletion UX;
- domain timestamp additions or semantic occurrence/effective-time modeling;
- ID generation policy beyond accepting typed IDs and rejecting collisions;
- performance optimization beyond correct indexes implied by keys/foreign keys;
- any psychological inference, recommendation, or theory selection.

## 10. Implementation acceptance boundaries

An eventual `CODEX_TASK_004.md` may be created only after owner approval and an approved Git planning checkpoint, with no changes to completed Task 002/003 history. It must preserve typed domain identity, validate on read, avoid frontend raw SQL domain writes and generic CRUD, make initial revisions atomic at the repository boundary, enforce persisted revision-origin history rules, append rather than overwrite history, and keep schema migration history append-only.
