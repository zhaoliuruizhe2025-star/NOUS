# CODEX TASK 004 — Persistence Foundation

## Status

**APPROVED SPECIFICATION — Task 004 becomes READY from this planning checkpoint.**

Owner approval has occurred. Implementation is authorized only after this specification is committed and fast-forwarded to `main`; that resulting specification checkpoint becomes the implementation starting checkpoint. Implementation must occur on `task/004-persistence-foundation`, not directly on `main`.

## Objective

Implement local SQLite persistence for the already-approved Task 002 and Task 003 Self Model domain:

- `SelfSubject`;
- `PersonReference`;
- `Situation`;
- `Observation`;
- `Thought`;
- `Emotion`;
- `Belief` and `BeliefRevision`;
- `Value` and `ValueRevision`.

The outcome is a narrow Rust application/repository boundary that preserves domain validation, typed identity, subject ownership, optional Situation links, immutable revision history, correction/update provenance, migration safety, transaction atomicity, and the frontend persistence security boundary.

Task 004 stores approved domain records. It does not infer psychological meaning, decide what should become durable, or implement a product capture workflow.

## Planning provenance

- Canonical repository: `D:\Dev\NOUS`
- Specification planning branch: `planning/task-004-spec`
- Specification drafting base: `21400ba Approve NOUS Task 004 persistence feasibility`
- Approved architecture: `docs/TASK_004_DESIGN.md`
- Approved feasibility decision: `docs/TASK_004_FEASIBILITY.md`
- Future implementation branch after approval: `task/004-persistence-foundation`
- Future implementation base: the later owner-approved checkpoint containing this specification

Do not start implementation from `21400ba` unless it is also the approved specification checkpoint, which it is not at the time this file is proposed.

## Required reading

Before implementation, read:

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
13. `docs/TASK_003_DESIGN.md`
14. `CODEX_TASK_003.md`
15. `docs/TASK_004_DESIGN.md`
16. `docs/TASK_004_FEASIBILITY.md`
17. this specification

Then inspect the current domain, tests, Tauri setup, migration registration, capabilities, and frontend database-readiness path before changing them. Repository code is authoritative if filenames move after this specification checkpoint; any material semantic conflict is a STOP condition.

## Scope exclusions

Task 004 must not implement or persist:

- Raw History or conversation storage;
- onboarding persistence or onboarding runtime;
- Academic Foundation or framework storage;
- `Memory`;
- `Decision`;
- `Outcome`;
- `Evidence`;
- a generic relationship graph;
- psychological inference or derived patterns;
- Thought-to-Belief promotion;
- response routing or interaction strategy;
- Care logic;
- prediction;
- final UI/UX redesign;
- backup or export;
- sync, cloud, mobile, accounts, or multi-user behavior;
- privacy-deletion UX;
- soft delete or restore;
- semantic occurrence, observation, inference, effective, or modification timestamps.

Do not absorb Task 005 or Task 006. Do not add an ORM, `rusqlite`, a generic database framework, a generic repository, `Repository<T>`, `UnitOfWork`, a generic transaction abstraction, a cache, a background write queue, or mocks merely for architectural style.

## Current domain contract

Persistence must map the real current Rust domain, not a conceptual substitute. The implementation starts with these constructors and fields:

| Domain type | Persisted domain fields | Validated construction |
| --- | --- | --- |
| `SelfSubject` | `id`, `display_name` | `SelfSubject::new` |
| `PersonReference` | `id`, `subject_id`, `display_name`, `relationship_label`, nullable `context_notes` | `PersonReference::new` |
| `Situation` | `id`, `subject_id`, `description` | `Situation::new` |
| `Observation` | `id`, `subject_id`, nullable `situation_id`, `content` | `Observation::new` |
| `Thought` | `id`, `subject_id`, nullable `situation_id`, `content`, nullable `confidence` | `Thought::new` |
| `Emotion` | `id`, `subject_id`, nullable `situation_id`, `label`, `intensity` | `Emotion::new` |
| `Belief` | `id`, `subject_id` | `Belief::new` |
| `BeliefRevision` | `id`, `belief_id`, `revision_number`, `proposition`, nullable `endorsement`, nullable `change_note`, `origin` | `BeliefRevision::new` |
| `Value` | `id`, `subject_id` | `Value::new` |
| `ValueRevision` | `id`, `value_id`, `revision_number`, `label`, nullable `importance`, nullable `change_note`, `origin` | `ValueRevision::new` |

Canonical IDs are validated nonblank string newtypes and serialize as strings. `ThoughtConfidence`, `EmotionIntensity`, `BeliefEndorsement`, and `ValueImportance` serialize as integers in `0..=100`. `RevisionNumber` serializes as a positive `u32`. `RevisionOrigin` serializes exactly as `InitialUserEntry`, `UserUpdate`, or `UserCorrection`.

`RequiredText` is module-private; persistence must reconstruct public entities through their existing constructors rather than bypassing it. Do not modify Task 002/003 structs merely to add persistence metadata or make row mapping convenient.

## Authorized dependency change

Task 004 authorizes exactly this new direct Rust dependency:

```toml
sqlx = { version = "=0.8.6", default-features = false, features = ["sqlite", "runtime-tokio"] }
```

SQLx 0.8.6 is already present transitively through `tauri-plugin-sql` 2.4.1. The exact pin preserves type compatibility with the plugin-managed `SqlitePool`. Cargo feature unification may still enable SQLx features required transitively by the plugin even though NOUS requests only `sqlite` and `runtime-tokio` directly.

Do not add any other dependency. In particular, do not add an ORM, `rusqlite`, a connection-pool crate, a test-only temporary-file crate, an error-derive crate, or a generic database framework. If implementation encounters a concrete need for another dependency, STOP and request owner review.

Removing the unused frontend `@tauri-apps/plugin-sql` package after replacing its only JavaScript use is permitted; that is removal of an obsolete frontend bridge, not authorization for a replacement dependency.

## Architecture and ownership

The required production path is:

```text
tauri-plugin-sql
  -> sole production migration authority
  -> preloaded sqlite:nous.db
  -> plugin-managed SQLx SqlitePool

Tauri command / future caller
  -> narrow Rust application service
  -> concrete SqliteSelfModelRepository
  -> direct SQLx API on a clone of the plugin-managed pool
  -> SQLite
```

Responsibilities are fixed:

- The application service selects and orchestrates a user-intent operation, supplies persistence creation time, and translates storage failures for its caller.
- Each repository operation guarantees its complete persistence result atomically.
- `SqliteSelfModelRepository` owns SQL, row mapping, `BEGIN` / `COMMIT` / `ROLLBACK`, constraint translation, and read reconstruction.
- The domain owns typed IDs, required text, bounded values, positive local revision numbers, and revision-origin vocabulary.
- React must not construct or submit SQL.

A concrete repository is sufficient. Do not create a Rust repository trait unless a concrete implementation or test demonstrates a real need and the implementation report explains it.

## Phase 0 — Implementation preflight

Before any implementation:

1. confirm the repository is `D:\Dev\NOUS`;
2. run `git status`, `git branch --show-current`, and `git log -7 --oneline`;
3. confirm `main` is clean and contains the owner-approved checkpoint for this specification;
4. record the full starting commit hash;
5. confirm Task 001–003 remain historical completed work;
6. confirm this specification is **READY** by explicit owner approval;
7. create `task/004-persistence-foundation` from that checkpoint; and
8. stop if any change or precondition is unexpected.

Do not implement directly on `main`. Do not commit or merge automatically.

## Phase 1 — SQLx declaration and shared-pool compile proof

This is the first implementation gate. Add only the authorized direct SQLx declaration, then make the smallest compile/runtime proof necessary to verify all of the following before writing the migration or broad repository code:

1. application Rust code can obtain Tauri-managed `tauri_plugin_sql::DbInstances`;
2. the preloaded `sqlite:nous.db` entry can be identified reliably rather than selected by map order;
3. the entry is `tauri_plugin_sql::DbPool::Sqlite` and exposes the expected SQLx 0.8.6 `SqlitePool` type;
4. that pool can be cloned into a narrow concrete Rust owner; and
5. Tauri/plugin setup ordering guarantees registered startup migrations finish successfully before repository operations or the readiness command can be served.

The proof must compile under the real application and include a focused test or other repeatable evidence for key resolution and readiness ordering where practical. Do not retain a throwaway generic abstraction after the proof.

### Mandatory STOP condition

If the shared plugin-pool approach does not compile, the map key cannot be identified reliably, migration readiness cannot be established, or the public plugin surface proves materially brittle, STOP before migration 0002 or broader repository implementation. Report the blocker and the exact proof attempted.

Do not silently open another production pool. The independent application-owned SQLx pool described in `docs/TASK_004_FEASIBILITY.md` is a bounded fallback that requires explicit owner approval before use.

## Phase 2 — Migration 0002

Only after Phase 1 succeeds, add one production migration:

```text
src-tauri/migrations/0002_create_self_model.sql
```

Register it as migration version 2 through the existing `tauri-plugin-sql` builder. Do not modify `0001_initialize.sql`. The plugin's applied-migration bookkeeping remains the sole production migration authority.

### Exact table and column shape

Migration 0002 must create explicit relational tables equivalent to the following. Constraint names may be added and SQL formatting may differ, but columns, nullability, relationships, and semantics must not drift.

```sql
CREATE TABLE self_subjects (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    created_at_ms INTEGER NOT NULL
);

CREATE TABLE person_references (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    relationship_label TEXT NOT NULL CHECK (length(trim(relationship_label)) > 0),
    context_notes TEXT NULL CHECK (
        context_notes IS NULL OR length(trim(context_notes)) > 0
    ),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT
);

CREATE TABLE situations (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (id, subject_id),
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT
);

CREATE TABLE observations (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (
        situation_id IS NULL OR length(trim(situation_id)) > 0
    ),
    content TEXT NOT NULL CHECK (length(trim(content)) > 0),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE thoughts (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (
        situation_id IS NULL OR length(trim(situation_id)) > 0
    ),
    content TEXT NOT NULL CHECK (length(trim(content)) > 0),
    confidence INTEGER NULL CHECK (
        confidence IS NULL OR confidence BETWEEN 0 AND 100
    ),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE emotions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (
        situation_id IS NULL OR length(trim(situation_id)) > 0
    ),
    label TEXT NOT NULL CHECK (length(trim(label)) > 0),
    intensity INTEGER NOT NULL CHECK (intensity BETWEEN 0 AND 100),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE beliefs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT
);

CREATE TABLE belief_revisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    belief_id TEXT NOT NULL CHECK (length(trim(belief_id)) > 0),
    revision_number INTEGER NOT NULL CHECK (revision_number > 0),
    proposition TEXT NOT NULL CHECK (length(trim(proposition)) > 0),
    endorsement INTEGER NULL CHECK (
        endorsement IS NULL OR endorsement BETWEEN 0 AND 100
    ),
    change_note TEXT NULL CHECK (
        change_note IS NULL OR length(trim(change_note)) > 0
    ),
    origin TEXT NOT NULL CHECK (
        origin IN ('InitialUserEntry', 'UserUpdate', 'UserCorrection')
    ),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (belief_id, revision_number),
    CHECK (
        (revision_number = 1 AND origin = 'InitialUserEntry') OR
        (revision_number > 1 AND origin IN ('UserUpdate', 'UserCorrection'))
    ),
    FOREIGN KEY (belief_id) REFERENCES beliefs(id) ON DELETE RESTRICT
);

CREATE TABLE "values" (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT
);

CREATE TABLE value_revisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    value_id TEXT NOT NULL CHECK (length(trim(value_id)) > 0),
    revision_number INTEGER NOT NULL CHECK (revision_number > 0),
    label TEXT NOT NULL CHECK (length(trim(label)) > 0),
    importance INTEGER NULL CHECK (
        importance IS NULL OR importance BETWEEN 0 AND 100
    ),
    change_note TEXT NULL CHECK (
        change_note IS NULL OR length(trim(change_note)) > 0
    ),
    origin TEXT NOT NULL CHECK (
        origin IN ('InitialUserEntry', 'UserUpdate', 'UserCorrection')
    ),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (value_id, revision_number),
    CHECK (
        (revision_number = 1 AND origin = 'InitialUserEntry') OR
        (revision_number > 1 AND origin IN ('UserUpdate', 'UserCorrection'))
    ),
    FOREIGN KEY (value_id) REFERENCES "values"(id) ON DELETE RESTRICT
);
```

`values` remains the approved logical table name, but `VALUES` is a SQLite keyword. Every SQL statement that directly references this table must therefore quote it consistently as `"values"`, including migration DDL, repository queries, migration/schema checks, and tests. Do not rename it to `user_values` or another physical table name.

Add only narrowly useful indexes for actual ownership/history lookup and foreign-key enforcement; do not invent a search/indexing subsystem. At minimum, index child foreign-key columns and preserve the unique parent/revision indexes created by the constraints.

The composite foreign keys on `(situation_id, subject_id)` are required. When `situation_id` is non-null, they prevent an Observation, Thought, or Emotion owned by one `SelfSubject` from linking to a Situation owned by another. The `UNIQUE (id, subject_id)` parent key on `situations` is required even though `id` is already the domain primary key, because SQLite requires a matching composite parent key.

Required-text SQL checks are a second structural layer. Rust constructors remain authoritative, including for Unicode whitespace that SQLite's default `trim` may not classify the same way.

Retain `app_metadata.schema_version` as an informational marker and set it to `2` within migration 0002. It is not a second migration controller; plugin migration bookkeeping remains authoritative.

## Timestamp policy

Every Task 004 table has exactly one persistence-only timestamp:

```text
created_at_ms INTEGER NOT NULL
```

It means UTC Unix-millisecond time when storage creates that row. It is not event occurrence time, observation time, inference time, effective time, or modification time. Revision history ordering is always `revision_number`, never `created_at_ms`.

Supply this value from the application boundary using the standard library/current application clock. Keep it in persistence/application input or row types; do not add it to Task 002/003 domain structs and do not add a clock dependency or speculative clock framework.

## Phase 3 — Persistence errors and row reconstruction

Create a focused `PersistenceError` family covering at least:

- `NotFound` for an expected record or revision parent that does not exist;
- `ConstraintViolation` for known primary-key, foreign-key, unique, not-null, or check failures;
- `DomainReconstruction` for stored data rejected by a domain constructor/value object;
- `Storage` for other SQLite I/O, busy/locked, transaction, or driver failures; and
- `Migration` or `NotReady` for migration/readiness failure where that distinction is available at the application boundary.

Preserve diagnostic causes or safe local context for tests/logging. Tauri commands must translate failures into a narrow non-psychological result and must not expose raw SQL, paths, or driver internals directly to frontend users.

Use explicit private row types where they make nullability and SQL conversion clear, such as `SelfSubjectRow`, `ThoughtRow`, `BeliefRevisionRow`, and `ValueRevisionRow`. Row types do not redefine domain semantics and must not be exported as an alternate model.

Every read must reconstruct the public domain through existing validated APIs:

- every typed ID via its `new`/`TryFrom` path;
- entity text through the entity/revision constructor and therefore `RequiredText`;
- numeric values through `ThoughtConfidence`, `EmotionIntensity`, `BeliefEndorsement`, or `ValueImportance`;
- revision numbers through `RevisionNumber`;
- origin through an explicit mapping to the three `RevisionOrigin` variants; and
- final records through their existing entity/revision constructors.

An invalid row returns `DomainReconstruction` with entity/field context. Never skip it, clamp a percentage, repair text, substitute an origin, downgrade an error to `None`, or construct an invalid object.

## Phase 4 — Task 002 entity persistence

Implement intention-revealing operations for:

- `create_self_subject` / `load_self_subject`;
- `create_person_reference` / `load_person_reference`;
- `create_situation` / `load_situation`;
- `create_observation` / `load_observation`;
- `create_thought` / `load_thought`; and
- `create_emotion` / `load_emotion`.

Names may follow existing Rust conventions, but the operation families and typed inputs/outputs must remain explicit. Creation accepts validated domain objects plus persistence creation time; loading accepts the matching typed ID and returns a reconstructed domain object or `NotFound`.

Do not add generic CRUD, unrestricted list/query builders, update/delete APIs, psychological search, or cross-entity inference. `PersonReference` remains contextual and cannot own beliefs, values, emotions, or a psychological profile.

## Phase 5 — Belief and Value transactional persistence

The minimum explicit operation families are:

- `create_belief_with_initial_revision`;
- `load_belief`;
- `append_belief_revision`;
- `load_belief_history` ordered by `revision_number ASC`;
- `create_value_with_initial_revision`;
- `load_value`;
- `append_value_revision`; and
- `load_value_history` ordered by `revision_number ASC`.

Exact input-struct names are an implementation detail, but normal callers supply typed anchor/revision IDs and revision content, not a revision number. The repository derives and constructs `RevisionNumber`. Initial creation fixes the origin to `InitialUserEntry`; append input may contain only `UserUpdate` or `UserCorrection`.

### Atomic creation

A persisted Belief or Value must never exist as an empty anchor. Each create operation owns one SQLite transaction:

```text
BEGIN IMMEDIATE
  validate/fix revision #1 and InitialUserEntry
  insert Belief or Value anchor
  insert its complete initial revision
COMMIT
```

If any step fails, roll back the anchor and revision together. A caller cannot separately insert an anchor or select a different number/origin for the initial revision.

### Atomic append and sequencing

Each append operation owns one SQLite transaction:

```text
BEGIN IMMEDIATE
  verify parent exists
  reject InitialUserEntry
  accept only UserUpdate or UserCorrection
  SELECT MAX(revision_number) for that parent
  derive the next positive revision number internally
  construct the validated domain revision
  insert the immutable revision
COMMIT
```

The repository—not the caller, application service, frontend, or `RevisionNumber`—owns persisted-history sequencing. `RevisionNumber` remains responsible only for local positivity. `UNIQUE (parent_id, revision_number)` is an independent final guard.

Use SQLx's explicit SQLite transaction support with `BEGIN IMMEDIATE` on the same acquired connection. Busy/locked outcomes must become explicit `Storage` errors (or a narrower busy variant if justified), not hidden semantic retries. Do not mutate or delete earlier revisions and do not expose normal revision update/delete operations.

## Foreign-key guarantee

SQLite foreign-key enforcement is connection-specific. Every connection used by a repository operation must be established or verified with foreign keys enabled.

For the shared plugin pool, do not treat `PRAGMA foreign_keys = ON` or `PRAGMA foreign_keys` on one arbitrary pooled connection as proof for the pool. Use the exact SQLx/plugin behavior plus an operation-level connection discipline that guarantees the connection actually executing the operation has `foreign_keys = 1`. One acceptable approach is:

1. explicitly acquire the connection used for the operation;
2. query `PRAGMA foreign_keys` on that same connection;
3. fail with `NotReady`/`Storage` if it is not `1`; and
4. execute the read or transaction on that same connection.

Tests must exercise multiple acquired connections and prove cross-subject and missing-parent foreign keys fail. If the plugin-managed pool cannot provide this guarantee, STOP and report it. Do not silently weaken the invariant or switch pools.

## Phase 6 — Rust readiness command and frontend SQL boundary

Preserve the current scaffold's initializing/ready/error behavior and schema-version check without redesigning the UI.

Replace `src/app/database.ts` access to `@tauri-apps/plugin-sql` with a narrow no-SQL Tauri command, conceptually `database_status`, that:

- accepts no SQL string, table name, or caller-selected database URL;
- executes the fixed `app_metadata` schema-version query in Rust through the established pool boundary;
- reports the expected schema version/readiness through a small serializable response; and
- returns a sanitized failure to the UI.

The frontend may invoke that fixed command and update the existing status display. It must not receive a database handle or a generic query/execute command.

Inspect `src-tauri/capabilities/default.json` and remove `sql:allow-select` and `sql:allow-load` once the JavaScript bridge is no longer used. Retain only plugin permissions genuinely required after replacement. Plugin preload and Rust migration registration remain; the infrastructure-only `app_metadata` read is not precedent for frontend domain SQL.

Remove the frontend `@tauri-apps/plugin-sql` dependency and resulting lockfile entry if no JavaScript import remains. Do not remove the Rust `tauri-plugin-sql` dependency or the configured `sqlite:nous.db` preload.

Add a regression check appropriate to the repository proving:

- frontend source has no import/use of the SQL plugin;
- frontend capability configuration grants no unscoped SQL load/select/execute permission;
- the exposed readiness command has no caller-selected SQL argument; and
- React has no invoke path that accepts caller-selected SQL against Self Model tables.

Do not add Task 004 CRUD screens or change the existing visual design, text, layout, localization, or navigation.

## Test migration helper

Production migrations remain registered and applied only by `tauri-plugin-sql`.

Tests must execute the real checked-in `0001_initialize.sql` and `0002_create_self_model.sql` against unique temporary on-disk SQLite files. Prefer reusing the production migration descriptor list or `include_str!` sources so test schema definitions cannot drift. The migration test must execute the SQL end to end, not merely inspect or parse it, so reserved-word and other SQL syntax failures are caught. A small test helper that applies those scripts is not a second production migration authority; do not create a second production registry or duplicate the DDL in test code.

Use the standard library to create unique temporary directories/files and clean them only after pools are closed. Do not add a temporary-file dependency. `:memory:` may support a focused unit case but cannot replace the on-disk suite.

## Phase 7 — Required test matrix

Add focused persistence, migration, concurrency, durability, and security tests.

### A. Migration

- fresh empty database -> migration 0001 then 0002 -> all expected tables, columns, keys, and constraints;
- version-1 database -> migration 0002 -> expected schema and informational schema version 2;
- `0001_initialize.sql` remains unchanged;
- migration failure is surfaced and readiness is withheld; and
- the test helper executes the real checked-in migration SQL end to end rather than inspecting it or using duplicated schema text, so reserved-word and syntax failures are caught.

### B. Round trips

- every in-scope entity round-trips through SQLite and its real domain constructor;
- English, Simplified Chinese, and mixed Unicode content;
- nullable `context_notes`, `situation_id`, `confidence`, `endorsement`, `importance`, and `change_note`; and
- boundary numeric values `0` and `100`.

### C. Structural invariants

- primary-key and typed-ID collision handling;
- foreign-key enforcement on repository-used connections;
- subject ownership;
- nullable Situation links;
- cross-subject Situation links rejected through the composite foreign key;
- required-text checks;
- bounded numeric checks;
- positive revision numbers;
- exact origin values and origin/revision relationship;
- `ON DELETE RESTRICT`; and
- no normal deletion API.

### D. Belief/Value creation

- anchor plus initial revision commits atomically;
- revision number is exactly 1 and origin is `InitialUserEntry`;
- normal input cannot choose another revision number or origin; and
- a forced second-step failure rolls back the newly inserted anchor.

### E. Append

- next revision numbers are derived internally and sequentially;
- `InitialUserEntry` is rejected after creation;
- `UserUpdate` is accepted;
- `UserCorrection` is accepted and remains distinguishable;
- missing parent returns `NotFound`;
- duplicate sequence is rejected independently by the database; and
- history is returned in `revision_number ASC` order.

### F. Immutability

- appending a later revision leaves every prior row byte-for-field unchanged at the persistence/domain level; and
- no repository operation exposes update/delete of a historical revision.

### G. Concurrency

- competing append operations cannot commit duplicate canonical revision numbers;
- successful competing operations serialize to distinct numbers where locking permits; and
- `BEGIN IMMEDIATE` / `SQLITE_BUSY` behavior is explicit and asserted rather than hidden behind semantic retry.

### H. Durability

- close all pool handles;
- reopen the same on-disk database file; and
- verify every tested record and ordered revision history remains present.

### I. Corrupt-row reconstruction

- inject controlled invalid stored values using test-only direct SQL/fixtures without adding a production bypass;
- each invalid typed ID, text, percentage, revision number, or origin returns `DomainReconstruction` or an equivalent explicit mapping error; and
- no invalid row is silently repaired, clamped, skipped, or converted to `None`.

### J. Security boundary

- frontend code cannot call the SQL plugin;
- effective capabilities contain no unscoped frontend SQL command permission;
- the database-status path accepts no caller-selected SQL; and
- existing initializing/ready/error scaffold behavior remains functional.

## Busy timeout and journal mode

Do not enable WAL merely because SQLite supports it. Do not add speculative concurrency tuning or hidden retry loops.

Observe actual contention behavior in the concurrency tests. A narrowly justified `busy_timeout` may be used only if needed for predictable local correctness/test behavior, applies consistently to repository connections, and is documented in the implementation report. If testing reveals that WAL, a material retry policy, a second pool, or another architectural choice is needed, STOP for owner review.

## Recommended implementation order

### Phase 0 — Preflight

Confirm approval/checkpoint, clean `main`, and create the task branch.

### Phase 1 — Direct SQLx declaration and shared-pool compile proof

Add only the authorized dependency and prove `DbInstances` -> exact `sqlite:nous.db` entry -> `DbPool::Sqlite` -> SQLx 0.8.6 `SqlitePool`, including migration-before-readiness ordering.

**STOP if this gate fails. Do not use the fallback pool without approval.**

### Phase 2 — Migration 0002

Add/register the one approved migration, keep 0001 unchanged, and add early migration/schema tests.

### Phase 3 — Persistence foundation

Add focused errors, connection/foreign-key verification, explicit row types, and validated row-to-domain mapping.

### Phase 4 — Task 002 entity persistence

Implement and test explicit create/load operations for the six Task 002 entity types.

### Phase 5 — Belief/Value transactional persistence

Implement atomic initial creation, `BEGIN IMMEDIATE` append sequencing, ordered history, immutability, and concurrency behavior.

### Phase 6 — Rust command and frontend SQL-boundary replacement

Replace the raw frontend schema query, remove unnecessary SQL capabilities/frontend dependency, and preserve the status UI behavior.

### Phase 7 — Complete persistence/migration/security tests

Finish the required on-disk, upgrade, corrupt-row, close/reopen, concurrent append, foreign-key, and security coverage.

### Phase 8 — Full verification and review report

Run all required checks, inspect the complete diff for scope drift, and stop in review without committing or merging.

## Expected implementation file boundary

The exact split may be adjusted to match Rust module ergonomics, but unexpected files or materially broader modules require review.

### Expected new files

- `src-tauri/migrations/0002_create_self_model.sql`
- `src-tauri/src/application/mod.rs` — narrow orchestration/readiness boundary
- `src-tauri/src/commands.rs` — fixed database-status command only, if not kept narrowly in `lib.rs`
- `src-tauri/src/persistence/mod.rs`
- `src-tauri/src/persistence/error.rs`
- `src-tauri/src/persistence/rows.rs`
- `src-tauri/src/persistence/sqlite_self_model_repository.rs`
- `src-tauri/src/persistence/tests.rs` and/or narrowly organized Rust integration-test files
- `tests/database-boundary.test.ts` or one equivalently focused frontend security regression test

Do not create placeholder modules merely to match this list. Combining a very small `commands.rs` or `rows.rs` into its owning module is acceptable and must be reported.

### Expected modified files

- `src-tauri/Cargo.toml` — exact direct SQLx declaration only
- `src-tauri/Cargo.lock` — Cargo's corresponding resolved root dependency change
- `src-tauri/src/lib.rs` — module registration, migration 2 registration, shared-pool integration, and fixed invoke handler
- `src-tauri/capabilities/default.json` — remove unscoped frontend SQL permissions
- `src/app/database.ts` — replace plugin SQL with fixed Rust invoke
- `src/app/App.tsx` — only if a minimal type/call adjustment is required; rendered behavior must not change
- `package.json` and `package-lock.json` — remove `@tauri-apps/plugin-sql` if unused after replacement

### Files explicitly not to modify

- `src-tauri/migrations/0001_initialize.sql`
- `src-tauri/src/domain/primitives.rs`
- `src-tauri/src/domain/entities.rs`
- `src-tauri/src/domain/commitments.rs`
- completed Task 002/003 domain tests except for a strictly necessary non-semantic compile accommodation, which is a review item
- `src-tauri/tauri.conf.json` unless the Phase 1 gate proves a concrete configuration conflict and the owner approves the change
- UI styles, localization copy, visual components, and navigation
- `docs/TASK_004_DESIGN.md` and `docs/TASK_004_FEASIBILITY.md`
- completed task specifications/history
- roadmap, product, safety, interaction, and backlog documents

No other migration, dependency, feature area, or documentation rewrite is expected.

## Required verification commands

Run from the repository root unless noted:

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

The Rust test command must include the migration, on-disk persistence, close/reopen, foreign-key, rollback, reconstruction, and concurrent append tests. The npm test command must include the frontend SQL-boundary regression check.

If a required command cannot run because an existing prerequisite is missing, report the exact prerequisite and stop before any system-level installation or configuration change. Do not weaken or omit a failing check without owner review.

## Acceptance criteria

Task 004 is ready for owner review only when all of the following are true:

- Phase 1 proved the preferred shared plugin pool; no fallback architecture was silently used.
- The exact direct SQLx dependency is the only added dependency.
- Migration 0002 creates only the ten approved tables and preserves migration 0001.
- The plugin remains the sole production migration authority.
- All persisted reads reconstruct validated Task 002/003 domain objects.
- Every repository-used connection has foreign keys established or verified.
- Task 002 records round-trip with ownership and optional Situation links intact.
- Belief/Value anchors can be created only atomically with initial revision #1.
- Initial and later origin rules are enforced by repository validation and database checks.
- Revision numbers are repository-generated under `BEGIN IMMEDIATE` and unique per parent.
- Historical revisions cannot be mutated through the normal API.
- Busy/locked outcomes are explicit.
- On-disk close/reopen durability passes.
- React no longer has caller-selected SQL access to the domain database.
- The existing readiness display works through the narrow Rust command.
- No excluded Task 005/006, inference, response, Care, prediction, UI redesign, export, sync, or deletion scope entered the implementation.
- Every required command passes.

## Architecture-drift review

Before reporting completion, compare the implementation against `docs/TASK_004_DESIGN.md` and `docs/TASK_004_FEASIBILITY.md` and explicitly check for:

- a second production migration authority;
- the independent fallback pool becoming the default;
- frontend raw SQL remaining reachable;
- generic CRUD or a generic repository framework;
- a trait introduced only for style;
- a `UnitOfWork`, ORM, cache, or write queue;
- empty persisted Belief/Value anchors;
- caller-selected revision numbers;
- mutation/deletion of historical revisions;
- timestamps leaking into domain or psychological semantics;
- psychological interpretation encoded in SQL;
- Task 005/006 scope creep; or
- UI redesign.

Any such drift is a STOP condition unless it was separately approved.

## Implementation completion report

The implementation report must include:

- repository path;
- task branch;
- full starting checkpoint;
- files added;
- files modified;
- dependency changes;
- migration added and registration details;
- whether the shared plugin-pool compile proof succeeded and its evidence;
- final application/repository/persistence structure;
- foreign-key guarantee used for repository connections;
- frontend SQL permissions before and after;
- test counts and results;
- fresh and version-1 migration results;
- atomic rollback result;
- close/reopen result;
- concurrent append result;
- observed busy/locking behavior and any timeout used;
- security-boundary regression result;
- any incomplete or deferred concern;
- `git diff --stat` summary;
- final `git status`; and
- architecture/safety concerns discovered.

After implementation, run `git diff` and `git status`, then stop in **REVIEW**. Do not commit, merge, begin Task 005, or continue into another feature without explicit owner instruction.
