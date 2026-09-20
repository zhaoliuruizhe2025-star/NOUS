# CODEX Task 006 — Evidence and Explicit Relationships

## Status

**PROPOSED SPECIFICATION — owner review and an approved Git planning checkpoint are required.**

**THIS DOCUMENT DOES NOT AUTHORIZE IMPLEMENTATION UNTIL OWNER REVIEW AND APPROVAL.** Do not create an implementation branch, migration 0004, domain types, repository operations, tests, dependency changes, or runtime changes from this draft alone.

## Planning provenance

- Repository: `D:\Dev\NOUS`
- Planning starting checkpoint: `ad983c6a0046ec9cb01d35a85ac5ad73fc2119b1`
- Planning branch: `planning/task-006-evidence-relationships`
- Governing design: `docs/TASK_006_DESIGN.md`
- Governing feasibility review: `docs/TASK_006_FEASIBILITY.md`

The eventual implementation starting checkpoint must be the owner-approved Task 006 planning merge on `main`, recorded by full commit hash before implementation begins.

## 1. Objective

Implement the approved bounded EvidenceLink foundation:

- one explicit user-authored relationship from one typed source record;
- to one exact BeliefRevision or ValueRevision;
- with one approved relationship kind and inspectable provenance;
- enforced by validated Rust domain types and real SQLite foreign keys/CHECK constraints.

Task 006 provides traceability foundations only. It is not a reasoning, inference, graph traversal, capture UI, or automatic Self Model update task.

Its relationship scope is intentionally limited to explicit evidential relationships represented by EvidenceLink. The future general-purpose relationship/graph layer and broader kinds such as `DEPENDS_ON`, `IMPLIES`, `ASSOCIATED_WITH`, and `INFLUENCES` are deferred rather than implicitly supported.

## 2. Exact domain scope

Add only:

- `EvidenceLinkId`;
- `EvidenceRelationshipKind::{Supports, Contradicts, Complicates, Contextualizes}`;
- `EvidenceProvenance::UserAuthored`;
- bounded `EvidenceSource` variants for Observation, Thought, Emotion, Situation, Memory, Decision, and Outcome;
- bounded `EvidenceTarget` variants for BeliefRevision and ValueRevision, each carrying its anchor ID as required for ownership integrity; and
- immutable `EvidenceLink` with `id`, `subject_id`, `source`, `target`, `relationship`, `provenance`, and optional nonblank `user_note`.

Do not add persistence `created_at_ms` to the domain type.

### Legal source/kind/target matrix

- Observation, Thought, Memory, Decision, and Outcome may use all four relationship kinds with either approved target family.
- Situation and Emotion may use only `Contextualizes` with either approved target family.
- Targets are only BeliefRevision and ValueRevision, and legality must validate the complete source/kind/target triplet rather than independent source/kind rules.
- PersonReference, Belief/Value anchors, and commitment revisions are not source variants.
- No generic type/string entity reference or arbitrary relationship constructor is allowed.

For BeliefRevision, `Supports`/`Contradicts` concern support for or conflict with the represented proposition. For ValueRevision, they concern consistency or tension with the represented orientation/priority and never mean that the Value is true or false. `Complicates` and `Contextualizes` must likewise retain their target-specific meanings from the design. All kinds describe the user's explicit view of the relationship; they do not assert objective truth or moral correctness.

An Observation remains a user-reported observation, and a Thought remains an interpretation-type source after linking. EvidenceLink never erases the source's epistemic type.

### Duplicate-link policy

Distinct EvidenceLink IDs may store the same semantic assertion tuple. Each is a separate immutable user-authored assertion record, but repetition confers no additional evidential weight, confidence, corroboration, or ranking.

Do not add semantic duplicate UNIQUE indexes, a hash/key/blob/registry, or a race-prone repository duplicate precheck. Task 006 contains no scoring or aggregation API. Any future quantitative use requires a separately approved deduplication policy.

## 3. Exact exclusions

Do not implement:

- generic non-evidential relationship edges;
- `AnyEntity`, `Node`, `Graph`, or generic `Relationship<T, U>` abstractions;
- `DEPENDS_ON`, `IMPLIES`, `ASSOCIATED_WITH`, `INFLUENCES`, `EVIDENCE_FOR`, or `EVIDENCE_AGAINST`;
- Observation-to-Thought, Thought-to-Thought, or commitment-to-commitment edges;
- multiple sources in one EvidenceLink;
- system proposals, pending/confirmed/rejected workflow, or system provenance;
- automatic link creation;
- automatic Belief/Value creation or revision;
- recurring-pattern or contradiction detection;
- inference, transitive inference, causal inference, graph traversal, or cycle analysis;
- confidence propagation, scoring, weighting, aggregation, ranking, embeddings, vector search, or semantic search;
- PersonReference psychological state or third-party relationship claims;
- update, delete, reparent, correction, rejection, or revocation APIs;
- semantic timestamps;
- frontend Evidence commands, UI, or arbitrary SQL/type/table selectors;
- Grounded Strength implementation;
- Task 007+ behavior.

Do not add dependencies, an ORM, graph database/library, `rusqlite`, another pool, `Repository<T>`, UnitOfWork, generic CRUD/query framework, WAL, retries, cache, or write queue.

## 4. Persistence contract

### Migration 0004

Create exactly one new domain table:

```text
evidence_links
```

The migration may additionally create only the composite unique parent indexes required for scoped foreign keys and the narrow child/read indexes required by the approved table.

Do not modify migrations 0001–0003.

The table must include:

- typed ID and subject ownership;
- closed relationship/provenance/source/target discriminators;
- explicit nullable source columns for the seven approved source tables;
- explicit Belief anchor/revision target columns;
- explicit Value anchor/revision target columns;
- optional nonblank user note;
- persistence-only `created_at_ms`;
- CHECK constraints for exactly one matching source and target family;
- CHECK enforcement of the complete source/kind/target allowlist, including the Situation/Emotion contextual-only rule;
- real scoped FKs for each source and both target families; and
- `ON DELETE RESTRICT` throughout.

Add the required candidate keys described in `TASK_006_DESIGN.md`. Existing Situation and Decision composite keys must be reused. `"values"` must remain quoted in every direct SQL reference.

Target ownership must remain structurally enforced even though revision rows do not contain `subject_id`: each link carries the relevant anchor ID and revision ID, references `(anchor_id, subject_id)` on the anchor, and references `(revision_id, anchor_id)` on the revision. Migration 0004 may add the matching unique candidate-key indexes; it must not weaken this chain to application-only validation.

Do not add uniqueness over the semantic assertion tuple. Only `EvidenceLink.id` is unique.

Update informational `app_metadata.schema_version` to `4`. `tauri-plugin-sql` remains the sole production migration authority.

### Production operations

Add only:

- `create_evidence_link`;
- `load_evidence_link`;
- `load_evidence_links_for_belief_revision`; and
- `load_evidence_links_for_value_revision`.

Target lists use `ORDER BY created_at_ms ASC, id ASC` for deterministic storage/read order only. They do not rank evidence or express event/causal chronology.

Each operation must use the established shared plugin-managed pool and retained `VerifiedSqliteConnection`. Creation is one parameterized insert and must not introduce `BEGIN IMMEDIATE` or another transaction abstraction without a demonstrated, owner-reviewed need.

`create_evidence_link` may store an equivalent assertion under a different EvidenceLink ID. It must not calculate weight, count, confidence, or corroboration from duplicates. `UserAuthored` provenance requires explicit user intent to assert the exact relationship; ordinary conversation text must never invoke creation automatically.

Reads reconstruct through validated public domain APIs. Invalid persisted IDs, enums, provenance, text, nullable shapes, or source/kind combinations return `DomainReconstruction`; no repair, skipping, substitution, or defaulting is permitted.

## 5. Expected file boundary

Exact organization may follow current narrow module patterns, but do not reorganize completed code for style.

### Expected new files

- `src-tauri/migrations/0004_create_evidence_links.sql`
- one focused domain module and focused domain tests if existing organization warrants it

### Expected modified files

- `src-tauri/src/domain/primitives.rs` — `EvidenceLinkId` only
- `src-tauri/src/domain/mod.rs` — narrow module/re-exports/tests
- `src-tauri/src/lib.rs` — migration 4 registration and registration tests
- `src-tauri/src/commands.rs` — fixed readiness version 4 only
- `src-tauri/src/persistence.rs` — concrete row mapping, four operation families, focused tests
- `src/app/database.ts` — fixed expected schema version 4 only
- `tests/database-boundary.test.ts` — protect the new table/version boundary as necessary

### Forbidden files/areas

- migrations 0001, 0002, and 0003
- Task 002–005 domain semantics
- Belief/Value revision or transaction semantics
- Cargo/package dependency manifests and locks
- Tauri capabilities and `tauri.conf.json`
- UI/components/styles/localization/navigation
- approved planning/history documents during implementation
- Grounded Strength/interaction documents

If another file is materially necessary, STOP and request owner review before broadening scope.

## 6. Phase 0 — Implementation preflight

After this specification is owner-approved, committed, merged, and separately authorized:

1. confirm repository `D:\Dev\NOUS`;
2. confirm `main` is clean and synchronized with its upstream;
3. record the full approved planning merge checkpoint;
4. confirm migrations 0001–0003 hashes/content remain unchanged;
5. create `task/006-evidence-relationships` from that exact checkpoint;
6. confirm this status is updated to an approved/ready specification; and
7. STOP if any condition differs.

Do not implement on `main`.

## 7. Phase 1 — Domain types only

Implement the approved ID, enums, bounded source/target unions, EvidenceLink constructor/getters, serialization, and focused tests.

Test:

- valid IDs and Unicode optional note;
- every legal source category and both target categories;
- blank optional note rejection;
- Situation/Emotion contextual-only enforcement;
- legal and illegal source/kind/target triplets, including target-specific meaning boundaries;
- tampered serialization rejection;
- exact preservation of typed IDs; and
- absence of PersonReference/generic entity variants.

Do not create migration or persistence work until Phase 1 receives owner review.

## 8. Phase 2 — Migration 0004 and readiness

Create/register the one approved migration and advance fixed readiness to version 4.

Test with the real checked-in migrations:

- fresh 0001 -> 0002 -> 0003 -> 0004;
- version-3 -> version-4 upgrade;
- exactly one new domain table;
- exact composite candidate keys, columns, nullability, checks, FKs, indexes, and `ON DELETE RESTRICT`;
- each legal source/target shape;
- duplicate semantic assertion rows with distinct IDs accepted as separate records without a semantic uniqueness constraint;
- discriminator/null mismatch rejection;
- partial target-pair rejection;
- relationship matrix rejection;
- same-subject success and cross-subject failure for every source family and both target families;
- missing source, anchor, and revision rejection;
- schema version 4 informational mirror; and
- migration failure withholding readiness.

Confirm migrations 0001–0003 remain byte-for-byte unchanged. Do not create another migration authority.

Stop for owner review before repository operations.

## 9. Phase 3 — Concrete persistence operations

Add only the four approved operations and a private row representation genuinely needed for SQLite mapping.

Test:

- create/load each source variant and both target families;
- all approved relationship categories;
- optional note absent/present and Unicode;
- duplicate ID and missing/cross-subject references;
- NotFound direct load;
- target-specific zero/one/many lists;
- deterministic storage ordering;
- any corrupt list row failing the whole operation;
- corrupt IDs, enum strings, provenance, source/target shapes, and note reconstruction;
- exact-connection FK verification and disabled-FK `NotReady`; and
- no generic/update/delete/traversal API; and
- equivalent assertion rows remain distinct with no scoring or aggregation behavior.

Do not add frontend commands.

## 10. Phase 4 — Durability, security, and matrix completion

Use the existing temporary on-disk fixture:

1. migrate through version 4;
2. create representative links across all source families and both targets;
3. close all pool/connection handles;
4. reopen the same file;
5. load links and target lists; and
6. verify provenance and source/target identity survive.

Complete only genuine gaps in:

- CHECK/FK/unique/index behavior;
- corruption reconstruction;
- historical revision attachment and no automatic inheritance;
- frontend no-raw-SQL/table/type-selector regression;
- no PersonReference relationship path;
- no automatic inference/target mutation; and
- no duplicate-counting or evidential-weight behavior; and
- all Task 004/005 regressions.

## 11. Phase 5 — Final acceptance review

Run the complete repository verification gate:

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

Review the complete Task 006 diff against the approved design/feasibility/specification. Stop in REVIEW. Do not stage, commit, merge, push, or begin a later task without owner instruction.

## 12. STOP conditions

STOP before changing architecture if:

- SQLite cannot structurally enforce source existence, target consistency, and same-subject ownership;
- application validation would become the sole integrity mechanism;
- migrations 0001–0003 need modification;
- a generic polymorphic edge, entity registry, trigger framework, or many-table redesign appears necessary;
- a new dependency, second pool, WAL, retry policy, ORM, or generic repository appears necessary;
- system proposals, confirmation/rejection workflow, or multi-source bundles become required;
- a PersonReference psychological edge or third-party state is requested;
- a source/target/kind outside the approved matrix is required;
- automatic inference, traversal, scoring, pattern detection, or target mutation is needed; or
- frontend raw SQL/generic persistence commands appear necessary.

Do not silently work around a STOP condition.

## 13. Acceptance criteria

Task 006 is ready for owner implementation review only when:

- the approved bounded domain types exist and reject illegal combinations;
- Evidence remains a traceable explicit user assertion, not source content or proof;
- all links target exact commitment revisions;
- every source/target is same-subject and protected by real FKs;
- source epistemic type remains inspectable;
- PersonReference is absent from the matrix;
- one migration adds exactly one domain table and prior migrations remain unchanged;
- provenance is exactly user-authored and no proposal workflow is invented;
- no link mutates or is automatically inherited by a target;
- repeated equivalent links remain separate authored records and never imply additional evidential weight;
- reconstruction rejects every corrupt semantic field explicitly;
- close/reopen durability passes;
- frontend raw SQL remains unavailable;
- no reasoning/traversal/inference/dependency/UI scope enters the diff; and
- every required verification command passes.

## 14. Required completion report

The future implementation report must include:

- repository path, branch, and full starting checkpoint;
- files added/modified;
- exact domain shape and legal matrix;
- migration 0004 table, candidate keys, FKs, CHECKs, indexes, and registration;
- prior migration hash/integrity evidence;
- final concrete operation surface;
- same-connection FK guarantee;
- provenance and revision-specific history behavior;
- corruption, cross-subject, durability, and security results;
- Rust/frontend test counts and all verification results;
- dependency conclusion and architecture-drift audit;
- excluded/deferred behavior confirmation;
- complete diff stat, untracked files, and final status; and
- any architecture, privacy, or safety concern.

Implementation must end in **REVIEW** without committing or merging.
