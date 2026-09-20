# Task 006 Design — Evidence and Explicit Relationships

## 1. Status and purpose

**Status: PROPOSED / UNDER OWNER REVIEW — planning only.** This document does not authorize implementation, migration 0004, dependency changes, runtime changes, or an implementation branch. Owner approval and an approved Git planning checkpoint are required first.

Task 006 defines the smallest durable foundation needed to say that one existing user-owned record explicitly bears on one exact Belief or Value revision. Its purpose is traceability and inspectability, not automated reasoning.

The task adds:

- one bounded `EvidenceLink` concept;
- an explicit legal source/target matrix;
- explicit user provenance;
- revision-aware persistence; and
- focused integrity, reconstruction, durability, and security tests.

It adds no inference, relationship discovery, graph traversal, scoring, or automatic Self Model mutation.

## 2. Evidence and relationship semantics

### 2.1 What Evidence means

An `EvidenceLink` is a durable user assertion that one exact source record supports, contradicts, complicates, or contextualizes one exact BeliefRevision or ValueRevision.

It is a first-class, inspectable assertion with its own identity and provenance. It is not independent factual content: the referenced source remains the content-bearing record.

Evidence is not:

- the source record itself;
- proof that the target is objectively true or morally correct;
- an inference result or hidden confidence score;
- a generic note or tag;
- an instruction to create or revise a Belief or Value;
- an automatic promotion of a Thought, Memory, Decision, or repeated occurrence; or
- permission to overwrite history.

Each Task 006 EvidenceLink has exactly one source and exactly one target. If several records bear on a target, they are represented by several independently inspectable links. Multi-source evidence bundles and aggregate arguments are deferred.

### 2.2 What a relationship means

A relationship is any typed connection between records. Existing direct domain relationships—such as Observation to optional Situation, Outcome to Decision, or BeliefRevision to Belief—are relationships but are not Evidence.

Task 006 does not add an arbitrary non-evidential relationship graph. Its only new relationship family is the bounded EvidenceLink assertion. Observation-to-Thought, Thought-to-Thought, generic association, causal influence, and similar graph edges remain deferred until a concrete workflow and semantics justify them.

This distinction prevents the model from treating every connection as evidential and prevents `AnyEntity -> AnyEntity` from becoming the architecture.

### 2.3 Direction and targets

Evidence direction is:

```text
typed source record -> relationship kind -> exact commitment revision
```

Targets are revision-specific:

- `BeliefRevision`, not the Belief anchor; or
- `ValueRevision`, not the Value anchor.

An anchor identifies continuity and ownership. A revision contains the exact proposition or value state to which the assertion applies. A link to one revision is never silently inherited by a later revision.

Task 006 does not target Observation, Thought, Emotion, Situation, Memory, Decision, Outcome, PersonReference, Belief anchor, or Value anchor. It also does not define evidence for future conclusion types.

## 3. Approved-for-review vocabulary

Task 006 proposes four relationship kinds:

| Kind | BeliefRevision meaning | ValueRevision meaning | Automatic effect |
| --- | --- | --- | --- |
| `Supports` | The user explicitly says the source is consistent with or lends support to the proposition represented by that revision. It does not prove objective truth. | The user explicitly says the source is consistent with, exemplifies, or lends support to the value orientation or priority represented by that revision. It does not make the Value “true.” | None |
| `Contradicts` | The user explicitly says the source conflicts with or counts against the proposition represented by that revision. | The user explicitly says the source conflicts with or is in tension with the value orientation or represented priority. It never means that the Value is false. | None |
| `Complicates` | The user explicitly says the source materially qualifies the proposition or shows that a simpler formulation is incomplete. | The user explicitly says the source materially qualifies how the value orientation applies, is prioritized, or is represented. | None |
| `Contextualizes` | The user explicitly says the source supplies relevant context for understanding the proposition without evidential polarity. | The user explicitly says the source supplies relevant context for understanding the value orientation or priority without evidential polarity. | None |

The following conceptual vocabulary is not approved in Task 006:

- `DEPENDS_ON` — ambiguous between logical, causal, practical, and historical dependence;
- `IMPLIES` — suggests inference and transitive reasoning;
- `ASSOCIATED_WITH` — too weak to remain semantically inspectable;
- `INFLUENCES` — risks unverified causal attribution;
- `EVIDENCE_FOR` / `EVIDENCE_AGAINST` — redundant with `Supports` / `Contradicts` in this bounded model.

None of the approved kinds mutates, ranks, confirms, rejects, or revises the target.

## 4. Legal source/kind/target matrix

Legality is evaluated over the complete triplet:

```text
source type + EvidenceRelationshipKind + target type
```

It is not modeled merely as a source-to-kind rule or as a kind that automatically applies to every target. The initial matrix is deliberately asymmetric and closed.

### 4.1 BeliefRevision targets

| Source type | `Supports` | `Contradicts` | `Complicates` | `Contextualizes` |
| --- | --- | --- | --- | --- |
| `Observation` | yes | yes | yes | yes |
| `Thought` | yes | yes | yes | yes |
| `Memory` | yes | yes | yes | yes |
| `Decision` | yes | yes | yes | yes |
| `Outcome` | yes | yes | yes | yes |
| `Situation` | no | no | no | yes |
| `Emotion` | no | no | no | yes |

For a BeliefRevision, evidential polarity concerns the proposition represented by that exact revision. It remains a user-authored assertion about how the source bears on the proposition, not an objective verification of it.

### 4.2 ValueRevision targets

| Source type | `Supports` | `Contradicts` | `Complicates` | `Contextualizes` |
| --- | --- | --- | --- | --- |
| `Observation` | yes | yes | yes | yes |
| `Thought` | yes | yes | yes | yes |
| `Memory` | yes | yes | yes | yes |
| `Decision` | yes | yes | yes | yes |
| `Outcome` | yes | yes | yes | yes |
| `Situation` | no | no | no | yes |
| `Emotion` | no | no | no | yes |

For a ValueRevision, evidential polarity concerns consistency or tension with the orientation or priority represented by that exact revision. A Value is not a proposition that becomes true or false.

The following source types have no legal triplet with either target:

- `PersonReference`;
- `Belief` or `BeliefRevision` as a source; and
- `Value` or `ValueRevision` as a source.

Important interpretations of the matrix:

- An Observation remains a user-reported observation, not verified objective fact. `Supports` records the user's explicit assertion about the link; it does not verify the target proposition.
- A Thought remains an interpretation. `Supports` from a Thought is support from an interpretation-type source and does not promote it into an Observation.
- A Memory remains retrospective and potentially imperfect.
- A Decision or Outcome may bear on how a commitment is represented, but does not prove causality.
- Emotion can provide context about the user's felt experience, but Task 006 does not treat feeling as factual proof for or against a proposition.
- Situation supplies context, not evidential polarity.
- PersonReference is neither an evidence source nor a target. Its existence identifies context, not another person's inner state.
- Outcome already has a direct required Decision link. Task 006 does not duplicate that relationship or turn Outcome into evidence targeted at Decision.

The matrix is a closed allowlist of triplets. Adding a source type, target type, relationship kind, or newly legal triplet requires a separately reviewed schema/domain change.

## 5. Provenance and user-confirmation boundary

### 5.1 Initial provenance model

Task 006 supports only:

```text
EvidenceProvenance::UserAuthored
```

`UserAuthored` means the current SelfSubject explicitly directs or intentionally completes creation of the exact source, relationship kind, and target revision tuple through a future focused application action. Merely mentioning two records in one conversation is not enough.

The durable link must result from explicit user intent to assert that relationship. Ordinary conversational text is never implicit permission to create an EvidenceLink, even when the text appears to mention both records or uses language similar to an approved relationship kind.

Ownership identifies whose Self Model contains the assertion. Provenance identifies that the assertion was explicitly made by that user. An optional nonblank `user_note` may preserve the user's own explanation of why the source bears on the target. The note is explanatory context, not a second evidence source and not an inference result.

### 5.2 Proposals and confirmation

Task 006 has no system-proposed, pending, confirmed, or rejected workflow. Those states would require a proposal artifact, presentation rules, and historical acceptance/rejection semantics that do not yet exist.

A future system proposal must remain separate and noncanonical until the user explicitly confirms the concrete link. Future confirmation must not rewrite the proposal into having always been user-authored. That design may add a distinct provenance or proposal history later; Task 006 does not reserve pretend functionality now.

Therefore a casual statement, repeated record, or system-detected correlation cannot create an EvidenceLink in Task 006.

## 6. Domain model

### 6.1 Types

Proposed types:

```text
EvidenceLinkId

EvidenceRelationshipKind
  Supports
  Contradicts
  Complicates
  Contextualizes

EvidenceProvenance
  UserAuthored

EvidenceSource
  Observation(ObservationId)
  Thought(ThoughtId)
  Emotion(EmotionId)
  Situation(SituationId)
  Memory(MemoryId)
  Decision(DecisionId)
  Outcome(OutcomeId)

EvidenceTarget
  BeliefRevision {
    belief_id: BeliefId,
    revision_id: BeliefRevisionId
  }
  ValueRevision {
    value_id: ValueId,
    revision_id: ValueRevisionId
  }

EvidenceLink
  id: EvidenceLinkId
  subject_id: SelfSubjectId
  source: EvidenceSource
  target: EvidenceTarget
  relationship: EvidenceRelationshipKind
  provenance: EvidenceProvenance
  user_note: Option<required text>
```

The anchor ID included in each target is an ownership bridge for SQLite integrity and makes the revision's commitment identity explicit. It does not permit anchor-level evidence.

`created_at_ms` remains persistence-only storage metadata and is not part of the domain object. Task 006 adds no event time, effective time, confirmation time, or psychological time.

### 6.2 Construction invariants

The public constructor must:

- accept only validated typed IDs;
- reject blank supplied `user_note` rather than convert it to `None`;
- enforce the closed source/kind/target triplet matrix;
- reject `Supports`, `Contradicts`, or `Complicates` for Situation and Emotion;
- expose no generic type-name/string-ID constructor;
- expose no arbitrary relationship kind; and
- perform no database lookup, inference, scoring, or target mutation.

Typed enums are bounded discriminated unions, not a generic `AnyEntity`, `Node`, `Graph`, or `Relationship<T, U>` framework.

## 7. Revision and history behavior

EvidenceLinks are immutable create/load records in Task 006.

- A link attaches to one exact BeliefRevision or ValueRevision.
- Later commitment revisions do not inherit prior links automatically.
- Old links remain inspectable with the old revision.
- Creating a new revision does not delete, move, or reinterpret an old link.
- Task 006 provides no update, delete, reparent, revocation, correction, or rejection API.
- Duplicate or revised user judgments must not be silently collapsed by inference.

Future correction/rejection/history semantics need a separate design. They should preserve the original assertion and later change rather than erase history.

### 7.1 Duplicate assertion policy

Task 006 permits two rows with different EvidenceLink IDs to contain the same semantic assertion tuple:

```text
subject + source + relationship kind + target revision + provenance
```

Each row is a distinct immutable user-authored assertion record. Repeated storage does **not** create additional evidential strength, confidence, weight, or independent corroboration. Optional notes do not change that rule.

The only uniqueness constraint is the EvidenceLink primary key. Task 006 deliberately does not add a large family of partial UNIQUE indexes, a normalized graph key, a semantic hash, a JSON identity blob, or an entity registry merely to canonicalize the sparse typed row.

Repository preflight duplicate rejection is also not used: without a matching database constraint it would be race-prone and would falsely imply canonical uniqueness. Future reasoning or scoring code must not count repeated equivalent rows as independent evidence unless a separately approved deduplication policy defines equivalence and historical treatment.

## 8. Third-party and epistemic boundary

Every EvidenceLink belongs to the current SelfSubject, and every source and target must belong to that same subject.

Task 006 must not encode:

```text
PersonReference -> BELIEVES / FEELS / INTENDS / WILL_DO -> anything
```

PersonReference cannot be a Task 006 source or target. References to another person remain context within the user's own Observation, Thought, Memory, Situation, or other user-owned record. A link continues to describe how the user's record bears on the user's commitment; it does not establish the third party's psychology or motive.

The source type remains visible during reconstruction and inspection. An Observation, Thought, Emotion, and Memory are not epistemically interchangeable. Repetition does not upgrade an interpretation into fact.

## 9. Proposed migration 0004

Migration 0004 should add exactly one new domain table:

```text
evidence_links
```

It may also add only the composite unique indexes required to make scoped foreign keys enforceable against existing tables. It must not modify migrations 0001–0003.

### 9.1 Required parent keys

Migration 0004 should add unique indexes equivalent to:

```text
observations(id, subject_id)
thoughts(id, subject_id)
emotions(id, subject_id)
memories(id, subject_id)
outcomes(id, subject_id)
beliefs(id, subject_id)
belief_revisions(id, belief_id)
"values"(id, subject_id)
value_revisions(id, value_id)
```

`situations(id, subject_id)` and `decisions(id, subject_id)` already have matching unique constraints. These new indexes do not alter identity or semantics; they expose composite candidate keys for scoped foreign keys.

### 9.2 `evidence_links` shape

The table should contain:

- `id`, `subject_id`;
- `relationship_kind`, constrained to the four approved values;
- `provenance`, constrained to `UserAuthored`;
- `source_kind`, constrained to the seven approved source types;
- one nullable typed source column per approved source table;
- `target_kind`, constrained to `BeliefRevision` or `ValueRevision`;
- nullable belief anchor/revision columns;
- nullable value anchor/revision columns;
- optional nonblank `user_note`;
- persistence-only `created_at_ms`.

CHECK constraints must guarantee:

1. exactly one source column is non-null and it matches `source_kind`;
2. exactly one target family is populated and it matches `target_kind`;
3. the complete source/kind/target triplet is present in the approved matrix, including Situation and Emotion permitting only `Contextualizes`;
4. required IDs/text are nonblank; and
5. enum strings are closed to the approved values.

Foreign keys must guarantee:

- direct subject ownership;
- `(source_id, subject_id)` references the selected source table's scoped candidate key;
- a Belief target's `(belief_id, subject_id)` references the Belief anchor and `(revision_id, belief_id)` references its revision;
- a Value target's `(value_id, subject_id)` references the Value anchor and `(revision_id, value_id)` references its revision; and
- all relationships use `ON DELETE RESTRICT`.

The explicit nullable columns are intentionally verbose. They allow SQLite to enforce every allowed reference. A compact polymorphic `(source_type, source_id, target_type, target_id)` row cannot provide these foreign keys.

### 9.3 Indexes and schema version

Add only indexes needed for foreign-key checks and approved reads:

- subject lookup;
- each populated source FK family;
- BeliefRevision target lookup; and
- ValueRevision target lookup.

Partial indexes on non-null source/target columns are acceptable if supported by the resolved SQLite stack and proven by migration tests. Do not add graph traversal, transitive closure, text search, scoring, or speculative analytics indexes.

Do not add semantic duplicate uniqueness indexes. Different EvidenceLink IDs may record the same assertion tuple under the duplicate policy in section 7.1.

Update `app_metadata.schema_version` to `4` as an informational readiness mirror only. `tauri-plugin-sql` remains the sole production migration authority.

## 10. Repository and application boundary

Proposed concrete operations:

- `create_evidence_link(link, created_at_ms)`;
- `load_evidence_link(EvidenceLinkId)`;
- `load_evidence_links_for_belief_revision(BeliefRevisionId)`; and
- `load_evidence_links_for_value_revision(ValueRevisionId)`.

The two target-specific list operations serve the concrete traceability question: “What explicit sources bear on this exact revision?” They should use deterministic storage ordering `created_at_ms ASC, id ASC`, which is not evidential strength, event chronology, or causal order.

No generic graph query, arbitrary source/target selector, arbitrary type string, traversal API, update/delete API, frontend SQL path, or frontend domain command is part of Task 006.

This is intentionally an explicit-evidential-relationships task only. It does not implement the future general-purpose relationship/graph layer. `DEPENDS_ON`, `IMPLIES`, `ASSOCIATED_WITH`, `INFLUENCES`, and other non-evidential relationship families remain deferred by design, not accidentally omitted.

Creation is a single parameterized insert after validated construction. It requires no new transaction model. Every operation must reuse the plugin-managed pool and the existing retained same-connection foreign-key verification path. Reads must reconstruct every typed ID, enum, optional text, source/target combination, and matrix rule through validated domain APIs; corruption fails explicitly.

## 11. Required invariants

1. Every link belongs to one SelfSubject.
2. Source, target anchor, and target revision belong to that same subject.
3. The source, relationship kind, and target triplet is in the closed matrix.
4. Exactly one typed source and one revision target exist.
5. Provenance is explicit and user-authored.
6. A link never mutates its target.
7. Later revisions do not inherit links automatically.
8. Source epistemic type remains visible.
9. PersonReference is never a source, target, or owner.
10. Historical records cannot be deleted through the Task 006 API.
11. Repeated equivalent rows remain separate authored records and never imply additional evidential weight.

## 12. Testing design

Future implementation tests should cover:

### Domain

- typed ID validation and serialization;
- every legal source/kind/target family;
- Situation/Emotion contextual-only rule;
- illegal kind combinations;
- optional nonblank user note and Unicode;
- tampered deserialization rejection; and
- absence of generic/PersonReference variants.

### Migration

- real fresh 0001 -> 0002 -> 0003 -> 0004;
- real version-3 -> version-4 upgrade;
- exact table, columns, checks, FKs, parent keys, and indexes;
- one-and-only-one source/target checks;
- every legal and representative illegal source/kind/target triplet;
- cross-subject rejection for every source family and both target families;
- missing source/anchor/revision rejection;
- illegal source/kind/target rejection;
- `ON DELETE RESTRICT`; and
- failed migration withholding readiness version 4.

### Repository and reconstruction

- create/load every legal source variant against both revision target families;
- target-specific lists and deterministic storage ordering;
- missing source/target and duplicate ID handling;
- corrupt IDs, enums, provenance, optional text, source shape, target shape, and matrix combination;
- one corrupt row failing the entire list rather than being skipped;
- exact-connection FK enforcement, including disabled-FK `NotReady`; and
- close/reopen durability with provenance intact.
- duplicate semantic assertions with different IDs remaining distinct records without any aggregation, ranking, or weight behavior.

### Security and scope

- no frontend raw SQL or arbitrary relation/table/type selector;
- no frontend Evidence commands in this task;
- no PersonReference psychological graph;
- no automatic relationship creation, inference, target mutation, or traversal; and
- all Task 004/005 persistence and security tests remain green.

## 13. Explicit exclusions and deferrals

Task 006 excludes:

- generic non-evidential graph edges;
- Observation-to-Thought and Thought-to-Thought relationship APIs;
- multi-source evidence bundles;
- system proposals and confirm/reject workflows;
- recurring-pattern or contradiction detection;
- graph traversal, transitive closure, and cycle analysis;
- causal inference or `Influences` semantics;
- confidence propagation, Bayesian scoring, hidden weights, ranking, or aggregation;
- embeddings, vector similarity, semantic search, or LLM-generated links;
- automatic Belief/Value creation or revision;
- reasoning traces beyond inspecting the explicit link and source;
- third-party psychological relationships;
- mutation, deletion, correction, revocation, and privacy-deletion UX;
- semantic timestamps;
- final UI, “Why?” surface, capture workflow, or frontend commands;
- Grounded Strength implementation; and
- all Task 007+ behavior.

Traversal and derived reasoning belong to later Phase C tasks. Task 006 supplies trustworthy explicit edges only.

## 14. Owner decisions incorporated during planning review

Owner review approved and refined these bounded choices:

1. Evidence is a revision-targeted first-class link, not a standalone content record.
2. Each link has exactly one source; multi-source bundles are deferred.
3. Targets are only BeliefRevision and ValueRevision.
4. The four-kind vocabulary is `Supports`, `Contradicts`, `Complicates`, `Contextualizes`.
5. Situation and Emotion are contextual-only sources.
6. Initial provenance is only `UserAuthored`; proposals/confirmation/rejection are deferred.
7. Task 006 adds no generic non-evidential relationship table.
8. Links are immutable and revision-specific with no automatic inheritance.
9. Duplicate semantic assertions may be stored as distinct authored records but carry no cumulative evidential weight.
10. Legality is evaluated over the complete source/kind/target triplet, with distinct BeliefRevision and ValueRevision meanings.

No unresolved semantic or technical blocker remains in this revised planning draft. Implementation is still unauthorized until the revised design, feasibility result, and `CODEX_TASK_006.md` receive final owner approval and an approved Git planning checkpoint.
