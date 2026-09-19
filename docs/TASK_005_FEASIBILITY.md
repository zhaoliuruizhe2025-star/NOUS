# Task 005 Feasibility — Lived-Experience Records

## 1. Status and purpose

**Status: REVISED AFTER OWNER REVIEW — feasibility planning only.** The reviewed Task 005 semantics are technically feasible. This revised artifact still requires final owner approval and an approved Git planning checkpoint before implementation, migration 0003, dependency changes, or `CODEX_TASK_005.md` execution.

This review checks the owner-reviewed Task 005 design against the repository at Task 004 merge checkpoint `0643ba29cce2868564d17bf036f31d0890ad9fe9`.

## 2. Conclusion

**Feasible with the current architecture and no new dependency.** The current typed-ID/required-text patterns can express the approved records. Migration 0003 can enforce direct subject ownership, optional same-subject Situation links, and required same-subject Outcome-to-Decision ownership with ordinary SQLite constraints. The existing shared plugin-managed SQLx pool, exact-connection foreign-key check, persistence errors, row reconstruction, and on-disk migration fixtures can be extended without redesign.

No Task 005 operation in the approved model requires a multi-step transaction. Each create is one parameterized insert. The existing transaction machinery remains available but should not be used ceremonially.

## 3. Current repository evidence

### 3.1 Domain patterns

The current Rust domain supplies reusable patterns rather than generic abstractions:

- string ID newtypes validate through `new` / `TryFrom<String>`;
- module-private `RequiredText` enforces nonblank text through public entity constructors;
- domain types use private fields, getters, `serde`, and validated deserialization;
- `SelfSubjectId` and `SituationId` already express the approved ownership/context links; and
- validation errors already cover empty text without adding a new error dependency.

Task 005 needs three new ID newtypes and three concrete entities. It does not need a public generic ID, timestamp, percentage, relationship, or lifecycle abstraction.

The approved domain policy is directly expressible with those patterns: `Memory.user_meaning` is optional but a supplied value must be nonblank and must never be normalized to `None`; Memory/Decision Situation links are optional and never auto-create a Situation; Outcome has no Situation field; all three records are immutable create/load records without revision tables; and no structured PersonReference link is added.

### 3.2 Persistence patterns

Task 004 established:

```text
tauri-plugin-sql migration/preload
  -> plugin-managed sqlite:nous.db SqlitePool
  -> SharedSqlitePool
  -> acquire_verified_connection
  -> PRAGMA foreign_keys = 1 on the retained operation connection
  -> concrete SqliteSelfModelRepository operation
  -> validated row-to-domain reconstruction
```

`PersistenceError` already provides the required `NotFound`, `ConstraintViolation`, `DomainReconstruction`, `Storage`, `Migration`, and `NotReady` families. Known SQLite primary-key, foreign-key, not-null, unique, and check failures already map to `ConstraintViolation`.

Task 005 should extend these concrete patterns. No repository trait, alternate pool, ORM, generic CRUD layer, or new error crate is needed.

### 3.3 Migration patterns

The production migration list currently registers real checked-in versions 1 and 2 through `tauri-plugin-sql`. Tests execute those same SQL files against unique temporary on-disk databases. Migration 0002 already supplies:

- `self_subjects` primary ownership;
- `situations` with `UNIQUE (id, subject_id)` for scoped composite links;
- `ON DELETE RESTRICT` conventions;
- required-text checks;
- focused child-key indexes; and
- informational `app_metadata.schema_version = 2`.

The approved migration shape can append version 3 without modifying either completed migration or creating another production migration authority.

Migration 0003 adds exactly `memories`, `decisions`, and `outcomes`. It adds no relationship, Evidence, revision, PersonReference junction, or temporal table.

## 4. Constraint feasibility

### 4.1 Optional same-subject Situation links

SQLite can reuse the exact Task 004 pattern:

```text
FOREIGN KEY (situation_id, subject_id)
  REFERENCES situations(id, subject_id)
  ON DELETE RESTRICT
```

When `situation_id` is null, the relationship is absent. When present, the composite key rejects a Situation owned by another subject. No application-only ownership convention is required.

### 4.2 Same-subject Outcome ownership

Add `UNIQUE (id, subject_id)` to `decisions`, then use:

```text
FOREIGN KEY (decision_id, subject_id)
  REFERENCES decisions(id, subject_id)
  ON DELETE RESTRICT
```

This ensures an Outcome cannot point across subjects. Keeping `subject_id` directly on Outcome makes ownership explicit and queryable while the composite foreign key proves it agrees with the parent Decision.

The composite parent key is required even though `decisions.id` is already a primary key. It follows the established Task 004 Situation ownership pattern and gives SQLite the exact parent-key shape referenced by `(decision_id, subject_id)`.

### 4.3 Approved Outcome cardinality

The approved cardinality is `Decision 1 -> 0..N Outcome`. A Decision may have no Outcome; every Outcome references exactly one Decision; multiple Outcomes for one Decision are valid.

Migration 0003 must not add a uniqueness constraint on `outcomes.decision_id`. Multiple rows do not imply causal stages or chronological phases.

### 4.4 Deterministic Outcome reading

`load_outcomes_for_decision` can use `ORDER BY created_at_ms ASC, id ASC`. SQLx/SQLite support this without another index beyond the approved relationship indexes. The order is deterministic storage/read order only. It is not event chronology, causal order, significance, or psychological order; `id` breaks equal-timestamp ties.

### 4.5 Required text and corruption

SQLite checks such as `length(trim(description)) > 0` provide a structural second layer. Rust constructors remain authoritative for the full Unicode rule. Test-only direct SQL can inject some corrupt rows with constraints disabled or schema-compatible bad values to prove reconstruction fails explicitly rather than repairing content.

## 5. Transaction review

The approved operations are single-record inserts:

- Memory insert;
- Decision insert; and
- Outcome insert with an enforced Decision foreign key.

SQLite already makes each statement atomic. An explicit `BEGIN IMMEDIATE` would not add a Task 005 invariant and would unnecessarily broaden lock scope. No operation performs revision allocation, anchor-plus-child creation, or multi-row state transition.

If owner review later requires atomic creation of a Decision plus one or more other records, that would be a material scope change and must be designed explicitly rather than hidden inside the repository.

## 6. Row reconstruction feasibility

Private persistence rows can carry `created_at_ms` while reconstructing:

- `MemoryId`, `DecisionId`, `OutcomeId` through validated ID constructors;
- `SelfSubjectId` and optional `SituationId` through existing constructors;
- required and optional text through `Memory::new`, `Decision::new`, and `Outcome::new`; and
- `DecisionId` on Outcome through its typed constructor.

Invalid stored IDs or text map naturally to `DomainReconstruction`. No numeric conversion, clamping, inferred default, or alternate public model is needed.

The only timestamp remains persistence `created_at_ms`. It cannot be used as event, decision, outcome, remembered, causal, or psychological-effective time. No richer temporal model is needed for the approved Task 005 scope.

## 7. Readiness and frontend boundary

Registering migration 0003 changes the expected schema version from 2 to 3. The existing fixed Rust `database_status` command and the narrow TypeScript readiness check must be updated to expect 3. This is a version accommodation, not a new domain command or UI feature.

The frontend remains unable to submit SQL. Capabilities remain free of `sql:*` permissions. The security regression should expand its forbidden table list to include `memories`, `decisions`, and `outcomes` and continue proving that `database_status` accepts no caller-selected SQL.

No Memory/Decision/Outcome command surface is required for this domain/persistence task. Product capture and UI workflows remain later work.

## 8. Dependency review

Task 005 needs no new Rust or frontend dependency.

Reuse:

- `serde` for validated serialization;
- exact direct `sqlx = 0.8.6` already approved and present;
- `tauri-plugin-sql` as the sole production migration authority;
- standard library temporary-directory/file helpers in tests; and
- the existing Vitest security-boundary test.

Do not add `rusqlite`, an ORM, `tempfile`, an ID generator, a time/date crate, an error-derive crate, a relationship library, or a test framework.

## 9. Test feasibility

The existing test architecture can cover Task 005 without a second schema or new framework:

- include and execute real migration 0003 after the real 0001/0002 SQL;
- verify fresh migration and version-2 upgrade on disk;
- inspect `sqlite_master`, `PRAGMA table_info`, `foreign_key_list`, and index metadata;
- use the concrete repository for round trips and constraint translation;
- use test-only direct SQL for structural and reconstruction corruption cases;
- close all pools before reopening the exact same database file; and
- reuse the multiple-connection foreign-key fixture.

The constraint suite can directly prove that same-subject Decision-to-Outcome creation succeeds, missing and cross-subject Decisions fail, and multiple Outcomes for one Decision succeed. Ordering tests can give Outcomes equal `created_at_ms` values and assert the ID tie-break without treating it as semantic chronology.

No concurrency test is required because Task 005 adds no read-modify-write sequence or allocated ordering. Existing Task 004 concurrency coverage must continue to pass.

## 10. Scope and architecture risks

The design remains feasible only if implementation resists these expansions:

- turning Memory into Raw History or an automatically extracted psychological record;
- adding option analysis, scoring, causal attribution, or evaluation to Decision/Outcome;
- adding generic relationships before Task 006;
- treating storage `created_at_ms` as event time;
- adding update/delete operations without correction/history semantics;
- attaching psychological state to PersonReference;
- exposing a generic frontend database or repository command;
- splitting or redesigning the Task 004 pool/persistence boundary merely for cleanliness.

The current large `persistence.rs` is maintainability debt, not a Task 005 architecture blocker. A module split should occur only if implementation review proves it necessary for correctness or manageable compilation, not as opportunistic cleanup.

## 11. Approved task structure

The owner approved Memory + Decision + Outcome as one coherent Task 005:

- three small domain entities;
- three new tables;
- six create/load operations plus one scoped Outcome query;
- migration/readiness version advancement; and
- focused domain/persistence tests.

Decision and Outcome remain together because their required relationship is the principal structural invariant. Memory shares the bounded lived-experience, ownership, migration, and reconstruction boundary. Do not split the task into 005A/005B.

## 12. Owner decisions resolved during planning review

Owner review approved:

1. zero-to-many Outcome cardinality;
2. optional `Memory.user_meaning`;
3. optional Memory/Decision Situation links;
4. omission of semantic event timestamps;
5. omission of revision tables and normal update/delete operations; and
6. deferral of structured PersonReference links to Task 006.

No new owner decision, repository conflict, or dependency blocker was found. Final approval of the revised planning artifacts and their Git checkpoint remains required before implementation.
