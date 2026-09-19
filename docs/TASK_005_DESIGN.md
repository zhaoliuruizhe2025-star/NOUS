# Task 005 Design — Lived-Experience Records

## 1. Status and objective

**Status: REVISED AFTER OWNER REVIEW — design only.** The Task 005 semantics recorded here are owner-approved. This revised planning artifact still requires final owner approval and an approved Git planning checkpoint before it authorizes implementation, migration 0003, dependency changes, or an implementation branch.

Task 005 defines the smallest domain and persistence extension needed to represent three user-owned lived-experience records:

- `Memory` — a retrospectively meaningful experience described by the user;
- `Decision` — a choice the user reports having made; and
- `Outcome` — what the user later reports happened after a Decision.

These are Structured Experience records. They are not inference results, Evidence, derived patterns, or claims of objective truth.

## 2. Product boundary

Task 005 records material supplied by the current user. It does not decide what that material proves, infer psychological causes, score its importance, or promote it into a Belief or Value.

The required distinctions are:

| Record | Represents | Does not represent |
| --- | --- | --- |
| `Situation` | Context in which a thought, emotion, observation, or decision occurs | A retrospectively meaningful account of a past experience |
| `Observation` | Something the user reports noticing or providing | A complete autobiographical memory or verified objective fact |
| `Thought` | An immediate interpretation, appraisal, prediction, or internal statement | A completed choice or remembered experience |
| `Memory` | A user-described past experience the user considers relevant now | A duplicate Situation, an objective event record, or inferred psychology |
| `Decision` | A choice the user reports having made | A Thought, recommendation, option-ranking result, or prediction |
| `Outcome` | What the user reports happened after a Decision | Causal proof, success/failure score, evaluation, or system inference |

Concrete details remain user-reported unless another source and its provenance are explicitly represented in a later task. Task 005 has no general provenance model; Task 006 owns Evidence and explicit relationship/provenance work.

## 3. Approved exact domain model

Owner review approved the following minimal model.

### 3.1 Typed identifiers

Add validated, nonblank string newtypes following the existing domain pattern:

- `MemoryId`;
- `DecisionId`;
- `OutcomeId`.

Task 005 does not introduce ID generation. Callers supply canonical typed IDs; persistence reports collisions explicitly.

### 3.2 Memory

Approved fields:

```text
Memory
  id: MemoryId
  subject_id: SelfSubjectId
  situation_id: Option<SituationId>
  description: required text
  user_meaning: Option<required text>
```

`description` is the user's account of the remembered past experience. `user_meaning`, when supplied, is the user's own present description of why the memory matters; it is not a system interpretation. A blank optional meaning is invalid rather than normalized to `None`.

The optional Situation link is direct and non-synthetic. It may be used only when an existing Situation genuinely represents context for the remembered experience. Creating a Situation merely to satisfy Memory storage is forbidden.

Creating a Memory never creates a Situation automatically.

Memory remains different from Situation even when linked:

- Situation is reusable context for structured records;
- Memory is a retrospective account selected by the user as relevant to their current self;
- autobiographical recollection is not treated as a perfectly objective record; and
- Task 005 never infers `user_meaning`, emotional weight, a trait, a Belief, or a Value from it.

This approved boundary deliberately excludes `emotional_weight`, confidence, importance, and system meaning. No approved scale or inference semantics exists for those fields.

### 3.3 Decision

Approved fields:

```text
Decision
  id: DecisionId
  subject_id: SelfSubjectId
  situation_id: Option<SituationId>
  description: required text
```

`description` must describe the choice the user reports having made, not merely the unresolved question or a list of options. Keeping one required choice description avoids prematurely designing option sets, ranking, anticipated outcomes, or Life Paths behavior.

The optional Situation link supplies context when a real Situation already exists. A Decision must remain valid without a Situation, Memory, Outcome, Evidence record, Belief, or Value link.

Creating a Decision never creates a Situation automatically.

### 3.4 Outcome

Approved fields:

```text
Outcome
  id: OutcomeId
  subject_id: SelfSubjectId
  decision_id: DecisionId
  description: required text
```

`description` records what the user reports happened after the Decision. It does not encode whether the Decision was good, whether the Decision caused the result, whether the result was expected, or how NOUS evaluates it.

An Outcome must reference an existing Decision owned by the same SelfSubject. The Decision link is fixed at creation; Task 005 provides no reparenting operation.

### 3.5 Approved cardinality

The approved relationship is:

```text
Decision 1 -> 0..N Outcome
Outcome N -> 1 Decision
```

A Decision may have no Outcome yet. Every Outcome references exactly one Decision, and multiple Outcomes may reference the same Decision. Migration 0003 must not add `UNIQUE (decision_id)`.

Multiple Outcomes are independent user-reported records. Their cardinality does not imply causal stages, chronological phases, increasing significance, or a psychological sequence.

### 3.6 Approved timestamp policy

`SELF_MODEL.md` lists event/timestamp fields as suggestions, but the repository has no approved temporal value object, precision policy, timezone policy, or approximate-date representation. Task 004's `created_at_ms` is explicitly persistence-only.

Task 005 adds no semantic event, memory-occurrence, remembered, decision, outcome, causal-order, or psychological-effective time. Each table receives only persistence `created_at_ms`, supplied separately at creation and kept out of the domain structs. A richer temporal model is deferred to a separately designed task.

The absence of semantic time does not turn `created_at_ms` into event, decision, outcome, remembered, causal, or psychological-effective time. Presentation and reasoning must not interpret storage creation order as when an experience occurred.

### 3.7 Approved mutability and revision policy

Memory, Decision, and Outcome are immutable create-and-load records in Task 005. No revision tables are added.

This does not claim that user correction is unimportant. It means correction/edit/delete semantics for these records are not sufficiently specified to copy the Belief/Value revision mechanism safely. Task 005 therefore exposes no update, replacement, reparenting, or deletion operation. Later correction/history work requires its own approved semantics and must preserve earlier history rather than silently overwriting it.

## 4. Direct links and ownership

Only these links are approved:

```text
Memory  -> SelfSubject
Memory  -> Situation (optional, same subject)
Decision -> SelfSubject
Decision -> Situation (optional, same subject)
Outcome -> SelfSubject
Outcome -> Decision (required, same subject)
```

No Task 005 record links directly to Belief, Value, Observation, Thought, Emotion, Evidence, or a generic relationship. Such links belong to Task 006's explicit relationship/Evidence design.

### 4.1 Third-party context

Every record belongs to the current SelfSubject. Task 005 does not add a `PersonReference` ownership path or a Memory/Decision/Outcome-to-PersonReference relationship table.

Another person may appear only inside user-authored descriptive content or through already-approved contextual records. That text remains the current user's recollection or perspective; it does not create the other person's Beliefs, Values, Emotions, motives, diagnosis, personality, probability, or predicted behavior. A structured direct PersonReference link is deferred until Task 006 can define explicit relationship cardinality and provenance without creating a third-party profile.

## 5. Validation

The domain should follow current Task 002/003 conventions:

- every typed ID rejects empty or whitespace-only values;
- every required text field rejects empty or whitespace-only values;
- `user_meaning`, when present, must be nonblank;
- user content supports English, Simplified Chinese, and mixed Unicode;
- constructors accept only typed ownership/link IDs;
- serialization must not bypass constructor validation; and
- no constructor repairs, trims into validity, scores, or infers content.

The public domain types should be serializable if the existing `serde` pattern remains appropriate, but no TypeScript duplicate model or frontend command is part of Task 005.

## 6. Approved persistence schema

Migration 0003 should add exactly three production tables and update the informational schema-version mirror to `3`.

It adds no relationship table, Evidence table, revision table, PersonReference junction table, or temporal table.

### 6.1 `memories`

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
```

### 6.2 `decisions`

```sql
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
```

The composite unique key exists so Outcomes can enforce same-subject Decision ownership through a composite foreign key.

### 6.3 `outcomes`

```sql
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

This shape implements the approved zero-to-many cardinality. It does not add `UNIQUE (decision_id)`.

### 6.4 Indexes and migration authority

Add only indexes that support child foreign-key/ownership lookup:

- `memories_subject_id_idx` on `memories(subject_id)`;
- `memories_situation_subject_idx` on `memories(situation_id, subject_id)`;
- `decisions_subject_id_idx` on `decisions(subject_id)`;
- `decisions_situation_subject_idx` on `decisions(situation_id, subject_id)`;
- `outcomes_subject_id_idx` on `outcomes(subject_id)`;
- `outcomes_decision_subject_idx` on `outcomes(decision_id, subject_id)`.

Do not add search, tags, full-text search, generic relationship indexes, or speculative query infrastructure.

Migration 0003 must be registered through `tauri-plugin-sql`, which remains the sole production migration authority. Migrations 0001 and 0002 remain byte-for-byte unchanged. `app_metadata.schema_version = 3` remains an informational readiness mirror, not a second migration controller.

## 7. Repository and application boundary

Extend the existing concrete persistence boundary with only:

- `create_memory` / `load_memory`;
- `create_decision` / `load_decision`;
- `create_outcome` / `load_outcome`; and
- `load_outcomes_for_decision`, ordered deterministically by `created_at_ms ASC, id ASC`.

The Outcome query ordering is storage/read determinism only. It is not event chronology, causal order, outcome significance, or psychological order. If two Outcomes have the same `created_at_ms`, the canonical string form of their typed ID is the deterministic tie-breaker.

Creation accepts a validated domain object plus persistence-only `created_at_ms`. Loading reconstructs through typed IDs and public constructors. Missing records return `PersistenceError::NotFound`; structural violations return `ConstraintViolation`; corrupt rows return `DomainReconstruction`.

Each operation must reuse the shared plugin-managed pool, acquire the actual connection used, verify `PRAGMA foreign_keys = 1` on that connection, retain it, and execute on it. These are single-row writes; no multi-step transaction or `BEGIN IMMEDIATE` sequence is required by the approved model.

No generic CRUD, unrestricted list/query builder, update/delete API, repository trait for style, `Repository<T>`, `UnitOfWork`, ORM, alternate pool, or frontend SQL path is permitted.

## 8. Testing design

Implementation should add the smallest focused coverage for:

- domain construction, typed-ID validation, required/optional text, Unicode, and serialization tamper resistance;
- real migration sequence 0001 -> 0002 -> 0003 on a fresh on-disk database;
- real version-2 -> version-3 upgrade and informational schema version 3;
- exactly the three new tables, their columns, foreign keys, checks, unique key, and indexes;
- create/load round trips for Memory, Decision, and Outcome;
- Memory/Decision with and without a Situation;
- nullable Memory `user_meaning`;
- missing subject rejection;
- valid same-subject and invalid cross-subject Situation links;
- same-subject Decision-to-Outcome creation succeeds;
- missing and cross-subject Decision-to-Outcome creation fails;
- multiple Outcomes for one Decision succeed;
- deterministic Outcome ordering, including an equal-`created_at_ms` ID tie-break, without asserting semantic chronology;
- duplicate typed IDs and required-text checks;
- `ON DELETE RESTRICT` through test-only SQL;
- corrupt IDs/text/link rows returning `DomainReconstruction` where injectable;
- close/reopen durability for all three records and Outcome ordering; and
- readiness version 3 plus preservation of the frontend no-raw-SQL security regression.

Tests must execute the real checked-in migrations and use temporary on-disk SQLite without a new dependency or second production migration registry.

## 9. Explicit exclusions

Task 005 excludes:

- Evidence and provenance entities;
- generic or inferred relationship graphs;
- links from lived-experience records to Beliefs/Values or other entities beyond the explicit schema above;
- recurring-pattern detection and reasoning traces;
- psychological inference or causal attribution;
- automatic Memory creation or Durable Self Model promotion;
- Thought-to-Belief promotion;
- confidence, emotional-weight, importance, success, utility, or quality scoring;
- prediction, recommendation, Life Paths, Care, response routing, or adaptive interaction behavior;
- option analysis and anticipated-outcome modeling;
- correction/edit/delete/history workflows beyond immutable creation;
- semantic occurrence/effective/modification timestamps;
- final UI/UX, forms, CRUD screens, or domain commands exposed to React;
- backup/export, sync/cloud/mobile, accounts, or multi-user behavior;
- third-party psychological profiles; and
- all Task 006+ behavior.

## 10. Task-size review

**Owner decision: keep Memory + Decision + Outcome together as one Task 005.** The bounded lived-experience layer adds three small immutable domain records, one migration, and seven narrow persistence operations. They share ownership, validation, migration, reconstruction, and durability infrastructure. Decision and Outcome require joint ownership/cardinality testing.

Do not split this work into Task 005A/005B. Such a split would duplicate migration/readiness and persistence verification without creating a clearer semantic boundary.

## 11. Owner decisions resolved during planning review

Owner review resolved the following semantics:

1. `Decision 1 -> 0..N Outcome` rather than `0..1`;
2. the exact minimal fields, including optional user-authored `Memory.user_meaning`;
3. optional same-subject Situation links on Memory and Decision;
4. no semantic event/decision/outcome timestamps in Task 005;
5. immutable create/load records with no Task 005 revision tables; and
6. no direct PersonReference links until Task 006 relationship design.

No new unresolved product decision or feasibility blocker was introduced by these amendments. The revised executable specification and planning checkpoint still require final approval before Task 005 implementation begins.
