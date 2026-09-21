# Task 007 — Structured Capture Foundation

## Objective and status

Task 007 implements the first narrow Phase B Remember workflow. Implementation and validation are complete, and final Owner Code Review passed with no BLOCKER, REQUIRED, or REVIEW findings. The implementation is approved for the Task 007 Git checkpoint. Task 007 becomes COMPLETE when that approved checkpoint is successfully merged into `main`.

## Approved user flow

```text
temporary natural expression
→ deliberate review
→ user-edited structured draft
→ explicit Save
→ domain validation
→ atomic local persistence
→ saved result display
```

The original natural-language input remains frontend memory only. Entering review, editing, removing fields, or cancelling does not persist anything. There is no automatic extraction or inference. Only the Save handler invokes the dedicated `save_structured_capture` command, and raw input is absent from that command's strict payload.

The review UI uses ordinary prompts for context, experienced details, and thoughts. Internal names remain Situation, Observation, and Thought, but users do not need to know that ontology.

## Capture shape

A capture contains an optional Situation, zero or more Observations, and zero or more Thoughts. At least one record is required. Any valid combination may be saved; missing categories are not manufactured.

When a new Situation is present, every Observation and Thought in that capture references it. Without a Situation, those references are `None`. Task 007 does not attach new records to older Situations.

ThoughtConfidence is always `None`. The request cannot supply confidence, subject ownership, an arbitrary Situation link, raw input, or another domain entity.

## Lazy SelfSubject bootstrap

The first successful Save lazily bootstraps the existing SelfSubject entity with internal ID `self` and neutral, non-user-facing display name `Self` if no subject exists. Exactly one existing subject is reused. More than one subject is an invariant error; NOUS does not guess.

The subject check and optional creation occur inside the same capture transaction as all other writes. A later failure rolls the subject back with the capture, so an unsuccessful first Save cannot leave an empty bootstrap identity.

## Architecture and atomicity

```text
React capture UI
→ fixed Tauri command DTO
→ structured-capture application workflow
→ SqliteSelfModelRepository::create_structured_capture_atomic
→ plugin-managed SQLite connection
→ validated domain result
```

The application workflow rejects empty input, resolves or prepares the current subject, constructs every domain object, and sets ThoughtConfidence to `None` before asking the repository to write. The repository rechecks the current-subject invariant under `BEGIN IMMEDIATE`, inserts the optional subject and all capture records, and commits only after every insert succeeds. Any failure explicitly rolls back.

The frontend supplies opaque record IDs from the browser's built-in `crypto.randomUUID()`, matching the existing caller-supplied typed-ID boundary without adding a dependency. The backend never accepts a frontend subject ID.

The command returns only validated IDs, content, optional Situation links, and the `None` ThoughtConfidence result needed by the UI. It does not return SQL rows, statements, internal errors, or persistence timestamps. It does not perform a post-commit reload.

## Domain entities used

- SelfSubject, only for lazy single-user bootstrap and ownership;
- Situation;
- Observation;
- Thought.

## Explicit non-goals

Task 007 does not implement Emotion, Belief, Value, Memory, Decision, Outcome, EvidenceLink, PersonReference modeling, Raw History, conversation storage, Pattern, Candidate/Hypothesis, NLP extraction, inference, promotion, history browsing, post-save editing/deletion/revision, export, backup, Phase C, Phase D, external AI, or cloud services.

## Validation guarantees

Focused Rust and frontend tests cover variable capture shapes, empty/invalid rejection, `None` confidence, one-subject reuse, multiple-subject rejection, atomic rollback including bootstrap rollback, cross-subject rejection, out-of-scope table non-creation, confirmed third-party interpretations remaining Thoughts, strict safe command DTOs/errors, no raw frontend SQL, close/reopen durability, cancellation boundaries, and equivalent English/Simplified Chinese message structures.

No migration, schema, manifest, lockfile, or dependency change is part of Task 007.
