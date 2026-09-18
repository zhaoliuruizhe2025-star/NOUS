# Task 003 Design — Commitments and Revision History

## 1. Purpose

Task 003 is the next proposed domain-model task after the completed Task 002 foundation. It defines durable, user-owned commitments and preserves their change over time without introducing persistence, inference, or a reasoning engine.

This approved design is translated into `CODEX_TASK_003.md`. Task 003 remains proposed and does not become READY or implementation-authorized until the owner reviews and approves that specification and its starting checkpoint.

## 2. Why this work is separate

Beliefs and Values share three important characteristics:

- both belong only to the current `SelfSubject`;
- both are durable enough to require history rather than replacement;
- both need user correction and provenance before later reasoning can use them.

Memory, Decision, Outcome, and generic Evidence have different relationship and provenance requirements. Combining them with commitment revisions would make the task harder to review and would force premature decisions about evidence links and lived-experience relationships.

## 3. Proposed scope

Implement only:

- `Belief`;
- `BeliefRevision`;
- `Value`;
- `ValueRevision`;
- typed identifiers and small value objects required by those types;
- revision ordering, validation, serialization, and focused unit tests.

Use the existing Rust `domain` module. Do not duplicate this model in TypeScript.

## 4. Explicit exclusions

Task 003 must not implement:

- SQLite schema, migrations, repositories, or application services;
- generic `Evidence` or cross-entity evidence links;
- `Memory`, `Decision`, or `Outcome`;
- graph relationships, contradiction detection, or inference;
- automatic promotion of a `Thought` into a `Belief`;
- NLP extraction, prediction, response routing, Care detection, UI work, or network behavior;
- third-party beliefs, values, emotions, motives, diagnoses, traits, or predictions.

## 5. Domain definitions

### 5.1 Belief

A `Belief` is a relatively durable proposition that the current user currently or historically endorses to some degree. It is not an objective fact, diagnosis, or system conclusion.

Examples:

- “A meaningful life is constructed through choices.”
- “Important opportunities may not come again.”

A `Belief` is an identity and ownership anchor: `id` and `subject_id`. Its changing content belongs in append-only `BeliefRevision` records. A BeliefRevision inherits ownership through `belief_id`; it must not duplicate `subject_id` unless a future design demonstrates a need. A current-state projection may select the latest revision later; Task 003 must not overwrite older revisions.

### 5.2 Thought versus Belief

A `Thought` is a momentary interpretation, appraisal, prediction, or internal statement. A `Belief` is a more durable proposition the user has endorsed.

Repeated Thoughts, observations, memories, decisions, or outcomes may become relevant context for a future user-led Belief revision. They do not automatically create or update a Belief. Task 003 contains no inference path from `Thought` to `Belief`.

### 5.3 Belief endorsement

Task 003 should use an optional, user-entered `endorsement` value rather than a generic truth-confidence field.

- Range: `0..=100` when supplied.
- Meaning: how strongly the user says they currently hold the proposition at that revision.
- It does not represent objective truth, system certainty, prediction confidence, or evidence strength.
- `None` means the user recorded the proposition without quantifying endorsement.

An endorsement of `0` is valid historical information: it can record that the user no longer endorses an earlier proposition. Later presentation must not silently treat a numeric value as a judgment about the person.

### 5.4 BeliefRevision

Every initial Belief state and later change is a `BeliefRevision`. A revision is a complete snapshot of the revision-specific state, including:

- `id`;
- `belief_id`;
- positive `revision_number`;
- proposition text;
- optional user-entered endorsement;
- optional user-entered change note;
- `RevisionOrigin`: `InitialUserEntry`, `UserUpdate`, or `UserCorrection`.

`InitialUserEntry` is the first user-authored canonical state. `UserUpdate` means the user's actual Belief changed over time. `UserCorrection` means the previous stored representation was inaccurate, mistyped, or otherwise incorrectly recorded; Mirror must not interpret it as psychological change.

Task 003 can validate only that a `revision_number` is positive. An isolated revision cannot enforce global uniqueness or strict sequencing across a Belief's history; those rules belong to the history, persistence, or application layer in Task 004 unless a separately approved aggregate abstraction is introduced. A shared timestamp policy is also deferred to that task.

User correction creates a new user-authored revision and never edits prior history. System inference is not a `RevisionOrigin` variant and remains out of scope: a future inference must be a separate, traceable proposal/result with its sources, rule version, and uncertainty. It must not overwrite a user revision; only a user-confirmed change may create a canonical revision.

A revision may represent rewording, refinement, a changed endorsement, evolution of the same underlying commitment, or correction. If the core semantic commitment has materially changed into a different proposition, create a new Belief rather than forcing it into the old history. Task 003 must not infer semantic continuity automatically; when ambiguous, user confirmation decides.

### 5.5 Value

A `Value` is an enduring orientation or priority that matters to the current user. It is not a proposition that is true or false.

Examples:

- autonomy;
- family;
- compassion.

Like `Belief`, `Value` is an identity and ownership anchor. Its label and priority state belong in append-only `ValueRevision` records. A ValueRevision inherits ownership through `value_id` and must not duplicate `subject_id` unless a future design demonstrates a need.

### 5.6 Value importance

Task 003 may use an optional, user-entered `importance` value in the `0..=100` range.

- Meaning: the value’s relative salience or priority to the user at that revision.
- It is not truth-confidence, moral worth, diagnosis, system ranking, or a recommendation.
- `None` allows a user to name a Value without quantified priority.

No automatic importance score or cross-user comparison is allowed.

### 5.7 ValueRevision

Every initial Value state and later change is a `ValueRevision` with:

- `id`;
- `value_id`;
- positive `revision_number`;
- user-facing label;
- optional user-entered importance;
- optional user-entered change note;
- `RevisionOrigin`: `InitialUserEntry`, `UserUpdate`, or `UserCorrection`.

The RevisionOrigin meanings and positive-only `revision_number` validation are the same as for BeliefRevision. Cross-revision uniqueness and strict sequencing remain a Task 004 history/persistence/application concern. Value revisions preserve historical priority and wording rather than updating an old state in place. A revision may represent rewording, refinement, changed importance, evolution of the same underlying Value, or correction. If the core semantic commitment has materially changed, create a new Value; user confirmation decides continuity when ambiguous.

## 6. Evidence boundary

`Evidence` is not part of Task 003.

Evidence is a traceable source record or explicit link that may support, complicate, or contextualize a model change. An Observation is not itself Evidence; it may later be a source record referenced by an Evidence link. Neither record automatically mutates a Belief or Value. Evidence is not an inference result or hidden score.

Potential evidence sources eventually include user-confirmed Observations, Thoughts, Memories, Decisions, Outcomes, and direct user statements. Memory, Decision, and Outcome do not exist yet, so a generic Evidence entity or asymmetric ad hoc links would be premature.

Task 003 revisions may contain a user-authored change note, but that note is not an Evidence entity and does not claim proof. Generic Evidence and explicit cross-entity relationship rules are deferred until the source model is broader and a separate relationship design is approved.

## 7. Lived-experience entities deferred from Task 003

### Memory

A `Memory` is a retrospectively meaningful user-described past experience. A `Situation` is context in which a current thought, emotion, or decision occurs. A Memory may refer to a past Situation later, but it is not a duplicate Situation: it includes remembered meaning and may be recorded long after the event.

### Decision and Outcome

A `Decision` records a user choice in context. An `Outcome` records what the user later reports happened after that Decision.

They should be implemented together: a Decision may have no Outcome yet, but an Outcome must refer to a Decision. Neither should require a synthetic Situation, Memory, or Evidence record.

## 8. Relationship rules and staging

Task 003 requires only these typed, non-synthetic links:

```text
Belief         -> current SelfSubject
Value          -> current SelfSubject
BeliefRevision -> Belief (inherits ownership)
ValueRevision  -> Value (inherits ownership)
```

Task 003 defers links between Beliefs/Values and Observations, Thoughts, Memories, Decisions, Outcomes, or PersonReferences. It also defers generic relationship types, evidence direction, and derived patterns.

The next proposed sequence is:

1. **Task 003** — Belief, Value, and their append-only revisions.
2. **Task 004** — persistence foundation for approved Task 002 and Task 003 entities.
3. **Task 005** — Memory, Decision, and Outcome as lived-experience records.
4. **Task 006** — generic Evidence foundation and explicit cross-entity relationships, without a reasoning engine.

This sequence preserves the Task 002 rule that the model must not invent data or placeholder relationships.

## 9. Third-party boundary

Every Task 003 entity belongs to the current `SelfSubject`. `PersonReference` remains contextual only.

Task 003 must not add Beliefs, Values, revisions, importance, endorsement, inferred motives, or psychological state to any `PersonReference`.

## 10. Validation and testing expectations

At minimum, implementation should test:

- valid initial Belief and Value revisions;
- empty/whitespace text rejection;
- ID and positive revision-number validation, without claiming isolated revisions enforce cross-history uniqueness or sequencing;
- `0`, `100`, and out-of-range endorsement/importance validation;
- Chinese, English, and mixed-language content;
- preservation of multiple revisions rather than mutation;
- all `RevisionOrigin` variants and their distinct user-authored semantics;
- serialization round trips and rejection of invalid serialized values;
- structural proof that no Task 003 entity belongs to `PersonReference`.

## 11. Open questions for later tasks

- The shared timestamp representation and persistence ordering policy.
- The final presentation semantics for an endorsement of `0`.
- Evidence-source vocabulary, direction, and user-confirmation workflow.
- Whether a future system proposal should be modeled as a dedicated derived artifact or a constrained provenance variant after reasoning design is approved.
- The UI flow for creating, reviewing, and correcting revisions.
