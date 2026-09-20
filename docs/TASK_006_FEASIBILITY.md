# Task 006 Feasibility — Evidence and Explicit Relationships

## 1. Status and question

**Status: PROPOSED / UNDER OWNER REVIEW — feasibility planning only.** This document does not authorize implementation, migration 0004, dependency changes, runtime changes, or an implementation branch.

This review asks whether the bounded EvidenceLink design in `TASK_006_DESIGN.md` can preserve legal source/target combinations, same-subject ownership, revision history, and validated reconstruction using the current SQLite/SQLx architecture.

Task 006 intentionally covers explicit evidential relationships only. The future general-purpose relationship/graph layer and its broader vocabulary remain deferred; this feasibility review does not attempt to make them fit the EvidenceLink table.

## 2. Conclusion

**Feasible with the current architecture and no new dependency, provided the schema uses explicit typed foreign-key columns rather than unconstrained polymorphic IDs.**

The recommended single `evidence_links` table is wider than a generic edge table, but it allows SQLite to enforce:

- source existence and source type;
- target anchor/revision consistency;
- same-subject ownership;
- exactly one source and target family;
- the closed source/kind/target triplet matrix; and
- the Situation/Emotion contextual-only rule.

The current shared plugin-managed SQLx pool, exact-connection FK verification, persistence errors, runtime row mapping, and on-disk migration fixtures can extend to this design. Each create is one row insert and requires no new transaction abstraction.

Implementation must STOP if compile/migration proof shows that the proposed composite foreign keys or CHECK expressions cannot enforce these invariants cleanly. Application-only referential integrity is not an acceptable fallback.

## 3. Current repository compatibility

### 3.1 Domain patterns

The current domain already provides:

- validated string-backed typed IDs;
- private fields and validated constructors;
- nonblank required/optional text;
- closed serialized enums;
- `SelfSubjectId` ownership;
- revision-specific `BeliefRevisionId` and `ValueRevisionId`; and
- tamper-resistant deserialization patterns.

A bounded `EvidenceSource` enum and `EvidenceTarget` enum fit these patterns without introducing `AnyEntity`, a generic graph node, or a trait hierarchy. Constructor validation can enforce the source/kind matrix without changing completed entity semantics.

### 3.2 Persistence patterns

The current production path is:

```text
tauri-plugin-sql migration/preload
  -> plugin-managed SQLx SqlitePool
  -> SharedSqlitePool
  -> VerifiedSqliteConnection
  -> concrete SqliteSelfModelRepository operations
```

Task 006 can preserve this path. SQLx 0.8.6 supports parameterized queries, nullable columns, row reconstruction, database constraint inspection, partial indexes, and ordinary SQLite transactions. No ORM, relationship library, graph database, second pool, or new dependency is needed.

### 3.3 Existing ownership keys

`situations` and `decisions` already expose `UNIQUE(id, subject_id)`. Other approved source/anchor tables have a primary key and `subject_id` but not a matching composite candidate key. SQLite requires the referenced columns of a composite foreign key to be a primary key or covered by a matching unique constraint/index.

Migration 0004 can add composite unique indexes without rewriting migrations 0001–0003. This does not change entity identity; it makes existing ownership pairs referenceable.

BeliefRevision and ValueRevision inherit subject ownership through their anchors. The proposed table therefore carries both revision ID and anchor ID. Two FKs prove the chain:

```text
(target_anchor_id, subject_id) -> anchor(id, subject_id)
(target_revision_id, target_anchor_id) -> revision(id, parent_id)
```

This is stronger than trusting a repository join after insertion.

## 4. Storage alternatives

### 4.1 Generic polymorphic edge table

Shape:

```text
(source_type, source_id, target_type, target_id, relationship_type)
```

**Rejected.** SQLite cannot make `source_id` reference a table chosen by `source_type`. It also cannot structurally prove same-subject ownership or revision/anchor consistency. CHECK constraints can validate strings but not polymorphic row existence. This would turn referential integrity into application convention and make corrupt or cross-subject edges easy to insert.

### 4.2 One table per source-target pair

For seven sources and two targets, this produces fourteen tables before future expansion.

**Not recommended.** Referential integrity is excellent, but migration, repository, and testing surfaces multiply without a corresponding semantic benefit. Adding a relationship kind matrix across fourteen tables creates repetitive SQL and higher drift risk.

### 4.3 Registry/supertype table for every entity

Shape: add an `entity_registry` parent, then make all current entities register a generic entity ID.

**Rejected for Task 006.** Existing tables do not inherit from such a registry. Keeping it synchronized would require changes to all create paths, triggers, or multi-step transactions, and would introduce a universal node abstraction into completed Tasks 002–005. It is a broad architectural redesign motivated by graph elegance rather than current product need.

### 4.4 Parent Evidence table plus subtype/junction rows

Shape: common Evidence metadata in one table with source/target subtype tables beneath it.

**Not recommended for initial Task 006.** SQLite cannot easily guarantee that each parent has exactly one valid subtype row without triggers, deferred multi-table validation, or application-only convention. Creation becomes a multi-step transaction for no user-visible benefit.

### 4.5 Narrow tables grouped by target family

Shape: separate `belief_revision_evidence` and `value_revision_evidence` tables with explicit nullable source columns.

**Feasible but not preferred.** It provides integrity and reduces target nullability, but duplicates source checks, provenance, reconstruction, and indexes. A shared EvidenceLink ID would not be globally unique without another registry. It also encourages target-specific domain types where the semantics are otherwise identical.

### 4.6 One bounded table with explicit typed columns

**Recommended.** A single table has one nullable column per approved source type and two explicit target families. CHECK constraints enforce the active discriminator and relationship matrix; ordinary FKs enforce existence and ownership.

Trade-offs:

- the row is wide and sparse;
- adding a source type requires a migration and code change; and
- SQL mapping is more verbose than a generic edge.

These are desirable constraints for the initial model: extension is deliberate, illegal combinations remain difficult, and no arbitrary graph API emerges.

## 5. SQLite constraint feasibility

### 5.1 Exactly one source

SQLite CHECK expressions can sum boolean non-null expressions, for example conceptually:

```sql
CHECK (
    (source_observation_id IS NOT NULL) +
    (source_thought_id IS NOT NULL) +
    ... = 1
)
```

A second CHECK binds the non-null column to `source_kind`. Migration tests must execute the real SQL and prove every legal/illegal shape.

### 5.2 Exactly one target family

The target CHECK can require either:

- Belief kind with both belief anchor and revision IDs populated and all Value columns null; or
- Value kind with both value anchor and revision IDs populated and all Belief columns null.

Composite foreign keys then prove revision-parent and parent-subject consistency.

### 5.3 Relationship matrix

A row-level CHECK should enumerate the approved source/kind/target triplets rather than validate only source-to-kind compatibility. It can enforce the contextual-only restriction conceptually as:

```text
source_kind in (Situation, Emotion)
  -> relationship_kind = Contextualizes
```

All other approved sources may use any of the four kinds for either approved target family, with target-specific semantics enforced in the domain design: BeliefRevision polarity bears on a proposition, while ValueRevision polarity bears on consistency or tension with an orientation/priority and never on whether a Value is “true.” The Rust constructor must enforce the same complete triplet allowlist for early domain feedback, while SQLite remains the independent structural guard.

Encoding target kind in the CHECK is intentional even though the initial allowed sets are parallel. A later target type or relationship kind must not become legal merely because it was added to one discriminator enum.

### 5.4 Scoped source keys

Adding unique indexes on `(id, subject_id)` is valid for current tables because each `id` is already globally unique. These indexes do not accept new data that was previously invalid and do not rewrite history. They only permit a composite child FK.

### 5.5 Nullable FK behavior

SQLite does not check a composite FK if one child column is null. The design relies on CHECK constraints to ensure the selected source/target columns are fully populated and all inactive families are null. Migration tests must explicitly prove that partial target pairs and discriminator mismatches fail.

## 6. Migration feasibility

Proposed migration 0004 can remain additive:

1. create the required composite unique parent indexes;
2. create `evidence_links` with CHECK and FK constraints;
3. create narrow child/lookup indexes; and
4. update informational `app_metadata.schema_version` to `4`.

Migrations 0001–0003 remain byte-for-byte unchanged. `tauri-plugin-sql` remains the sole production migration authority. Tests may execute the same checked-in SQL against isolated on-disk databases, as current tests do, without becoming a second production registry.

The migration contains multiple DDL statements but does not require a new application transaction model. The plugin's established migration mechanism owns migration execution. Implementation must verify failed migration behavior does not advertise schema version 4.

### 6.1 Duplicate assertion feasibility

Task 006 deliberately permits separate rows with distinct EvidenceLink IDs to repeat the same semantic assertion tuple. This is the smallest honest policy for the sparse typed-column design.

- Canonical SQLite uniqueness would require several partial UNIQUE indexes or an additional normalized identity representation.
- A generic hash, JSON key, or registry would add machinery unrelated to traceability.
- Repository-only duplicate rejection would be race-prone without a matching database constraint.

Therefore no semantic duplicate UNIQUE constraint or repository preflight query is proposed. The rows remain distinct authored records, but duplicates carry no additional evidential weight. Task 006 has no aggregation, counting, confidence, or scoring operation that could interpret repetition as strength. Future reasoning must define a separately approved deduplication policy before using links quantitatively.

## 7. Repository and transaction feasibility

`create_evidence_link` is one parameterized INSERT into one table after domain validation. It needs no `BEGIN IMMEDIATE`, multi-step transaction, retry loop, WAL mode, or UnitOfWork.

Loads can select the row and reconstruct:

1. `EvidenceLinkId` and `SelfSubjectId`;
2. closed discriminator, relationship, and provenance values;
3. the exact typed source ID;
4. the exact target anchor/revision pair;
5. optional nonblank user note; and
6. the final domain EvidenceLink through its public constructor.

Any unknown enum, inconsistent nullable shape, invalid ID/text, or illegal matrix combination must return `PersistenceError::DomainReconstruction`. No row may be repaired, skipped, defaulted, or converted to a different semantic type.

Target-specific list queries are ordinary indexed SELECTs ordered by `created_at_ms ASC, id ASC`. The order is deterministic storage/read order only.

Every operation can use the existing retained verified connection. No new pool, path, or frontend SQL access is necessary.

## 8. Provenance feasibility

The initial single-value `UserAuthored` provenance is intentionally narrow but still valuable:

- it records that the relationship is an explicit assertion by the modeled user;
- subject ownership identifies that user;
- optional user note preserves their explanation; and
- no system process can persist a canonical link through this API without falsely selecting a user-only provenance.

Creation must arise from explicit user intent to assert the exact link. Parsing ordinary conversation, co-occurrence, repetition, or system speculation is not authorization to call `create_evidence_link`.

The application layer must expose no automatic creation path in Task 006. System proposals are better modeled later as separate noncanonical artifacts than as extra enum values without a workflow.

The design does not yet answer how a future accepted proposal distinguishes “confirmed suggestion” from independently authored link. That is a deliberate deferral, not a feasibility blocker for user-authored links.

## 9. Security and privacy feasibility

The explicit schema prevents common generic-graph failures:

- source and target type strings are closed;
- legal source/kind/target triplets are closed;
- typed columns have real FKs;
- every relationship is scoped to one subject;
- PersonReference has no source/target column;
- no arbitrary table/type selector reaches SQL;
- repository SQL remains parameterized; and
- frontend raw SQL stays disabled.

A malformed database can still contain invalid enum/text combinations if constraints were bypassed. Validated reconstruction remains necessary and must fail explicitly.

Task 006 should not expose frontend Evidence commands. A future capture/inspection workflow should receive its own application/API review rather than turning this persistence foundation into a generic command surface.

## 10. Index feasibility

The recommended minimum is:

- composite unique parent indexes required by FKs;
- `evidence_links(subject_id)`;
- one child index for each source FK family, preferably partial when non-null;
- one BeliefRevision-target index; and
- one ValueRevision-target index.

This supports FK parent deletion checks and target-specific traceability reads. It does not optimize traversal, pattern discovery, transitive queries, or semantic search.

No semantic duplicate UNIQUE index is required or recommended under the approved distinct-authored-record policy.

Exact index column order should match the final FK definitions and approved SELECT predicates. Migration metadata and real behavior tests should verify them.

## 11. Testing feasibility

The existing temporary on-disk SQLx fixture can apply real migrations through version 4, close all pools, reopen the same file, and exercise the concrete repository. No `tempfile` or new test dependency is needed.

The matrix is finite:

- seven source types;
- two target families;
- four relationship kinds, with two sources restricted to Contextualizes.

Tests should cover every legal domain source/kind category at least once and every FK/source family, without multiplying every text/language case across the full Cartesian product.

Test-only direct SQL can inject corrupted rows only where constraints permit, or temporarily disable checks/foreign keys in isolated fixtures following current patterns. No production bypass is needed.

## 12. Dependencies

No new dependency is expected.

Reuse:

- exact SQLx `=0.8.6` already declared;
- `tauri-plugin-sql` migration/preload;
- current serde/domain validation;
- existing persistence error family; and
- standard-library temporary directory/file helpers.

If implementation appears to need a graph library, ORM, trigger framework, another SQLite client, code generation, or a new test dependency, STOP for owner review.

## 13. Transaction requirements

- EvidenceLink creation: one insert, no explicit transaction required.
- Loads/lists: read-only, no transaction required.
- Migration: owned by existing plugin migration machinery.
- No target mutation occurs in the same operation.
- No proposal/confirmation multi-step workflow exists.

If implementation discovers a need for multi-row Evidence creation, automatic target revision, or cross-link sequencing, that is new scope and a STOP condition.

## 14. Risks and review gates

### Material risks

1. Wide CHECK expressions are easy to write incorrectly; real migration behavior tests are mandatory.
2. Composite FK parent keys must match exact column order and collation.
3. Sparse row reconstruction must reject inconsistent discriminator/null shapes.
4. `Supports` and `Contradicts` can be overread as objective truth; domain docs and future UI must state that these are user-authored relationships to a self-model revision.
5. Future expansion could pressure the table toward a generic graph. Every new source/target/kind must remain an explicit reviewed migration.
6. Duplicate rows could be misread as corroboration by future code; the absence of scoring now and the mandatory future deduplication review must remain explicit.

### STOP conditions

Stop before implementation or architecture change if:

- SQLite cannot enforce source existence, target consistency, and same-subject ownership with the proposed additive keys;
- migrations 0001–0003 would need modification;
- a trigger/registry/second table is required merely to make a generic edge safe;
- application-only checks would become the sole referential-integrity boundary;
- a new dependency, second pool, WAL, retry policy, or generic repository is proposed;
- system proposals or confirmation workflow become necessary;
- a PersonReference psychological edge is requested; or
- any automatic inference or Belief/Value mutation is required.

## 15. Feasibility decision

The proposed bounded EvidenceLink foundation is technically feasible with the current repository and no new dependency. The recommended explicit-column design is intentionally less generic than a graph table because it preserves SQLite-enforced legality, same-subject ownership, revision history, and inspectable epistemic type.

The target ownership chain is structurally feasible without changing existing rows: migration 0004 adds candidate-key indexes on `(beliefs.id, beliefs.subject_id)` and `("values".id, "values".subject_id)`, plus revision-parent candidate keys. Each EvidenceLink then references both the anchor/subject pair and revision/anchor pair. A revision target therefore cannot belong to another subject even though revision rows do not directly store `subject_id`.

This conclusion remains a planning recommendation. Owner approval and an approved Git planning checkpoint are required before migration 0004 or implementation begins.
