# Task 008 — Structured History / Inspect Foundation

Status: **OWNER APPROVED — READY FOR TASK 008 GIT CHECKPOINT**

Final Owner approval:

- implementation completed and implementation validation passed;
- Final Owner Code Review: **PASS**;
- read-only inspect boundary, corruption visibility, relationship-grounded grouping,
  scope truthfulness, storage-time semantics, ThoughtConfidence semantics, and bilingual
  semantics approved;
- BLOCKER: none; REQUIRED: none; REVIEW: none;
- implementation is approved for the Task 008 Git checkpoint;
- Task 008 becomes **COMPLETE** when the approved checkpoint is successfully merged into main.

## Objective

Task 008 adds the first read-only Phase B inspect surface. A user may deliberately open a view of persisted Situations, Observations, and Thoughts for the single current SelfSubject. The purpose is inspectability and data transparency, without psychological interpretation or durable mutation.

## Approved inspect scope

The view includes only:

- Situation, presented as context;
- Observation, presented as user-reported experienced or noticed information;
- Thought, presented as the user's momentary interpretation, judgment, prediction, or internal statement.

This is complete only for those three record types within Task 008. It is not a complete Self Model browser and must not imply that an empty result means NOUS holds no other long-term data. SelfSubject remains ownership infrastructure; its bootstrap identifier and placeholder display name are never returned to the UI.

## Relationship-grounded presentation

The only grouping authority is the persisted `Observation.situation_id` or `Thought.situation_id` relationship. Linked records appear under their validated Situation. A Situation with no children remains visible. An Observation or Thought with no Situation remains visible in its standalone section.

Task 008 does not infer capture bundles or events from timestamps, IDs, insertion order, text similarity, or the Task 007 frontend state. It introduces no CaptureSession, Event, or CaptureBundle entity.

## Epistemic display semantics

Persisted content is displayed faithfully. Observation text remains a user report rather than an independently verified external fact. Thought text remains the user's thought rather than an external fact or third-party psychological claim. The inspect path adds no summary, inference, annotation, promotion, or interpretation.

## ThoughtConfidence

A null ThoughtConfidence produces no conviction row. A valid non-null value is returned unchanged and described only as how strongly the user believed that Thought at that time. It is not NOUS confidence, model confidence, factual probability, or objective certainty.

## Storage ordering

Repository results are ordered by `created_at_ms DESC`, then stable record ID `DESC`, within each record type. The timestamp is storage creation metadata used only for deterministic save ordering. It is neither semantic event time nor lived chronology, is not returned to the frontend, and is not displayed.

## Load-all decision

The first inspect slice performs one deliberate subject-scoped load of all Situation, Observation, and Thought rows. It adds no pagination, cursor, arbitrary limit, infinite scrolling, or silent truncation.

## Subject resolution and corruption visibility

The focused repository read uses one verified SQLite connection and one read transaction for subject resolution, all three table reads, domain reconstruction, ownership validation, and relationship validation.

- zero SelfSubjects and zero inspect-scope rows returns the scoped empty result and creates nothing;
- zero SelfSubjects with any inspect-scope row fails as inconsistent data;
- exactly one SelfSubject requires every inspect-scope row to belong to it;
- more than one SelfSubject fails the single-user invariant without selecting one;
- every non-null Situation relationship must resolve to a valid same-subject Situation.

The operation reads all rows in the three inspect-scope tables before validating ownership. It does not subject-filter corrupt rows into invisibility. Malformed, orphaned, foreign-owned, or unreconstructable rows fail the whole load rather than being skipped, repaired, detached, or reinterpreted.

## Architecture boundary

The flow is:

```text
Inspect UI
→ load_structured_history Tauri command (no request payload)
→ application read workflow
→ focused repository read snapshot
→ SQLite
→ validated records plus internal storage metadata
→ relationship-grounded application read model
→ safe response DTO
→ presentation-only UI
```

The repository owns snapshot consistency, subject and ownership checks, domain reconstruction, relationship integrity, and deterministic storage ordering. The application owns grouping by validated Situation ID and standalone classification. The frontend receives no subject IDs, foreign keys, timestamps, SQL fields, or database metadata.

## Read-only boundary

Inspection creates no SelfSubject and performs no durable write. Task 008 adds no edit, deletion, correction, revision, promotion, analysis, or inference command. Safe command errors are limited to `subjectInvariant`, `dataInconsistent`, `storageUnavailable`, and `loadFailed`; SQL and storage details are not exposed.

## Explicit non-goals

Task 008 does not add a complete Self Model browser; other entity browsing; correction, edit, deletion, undo, revision, export, backup, or delete-all; raw or conversation history; semantic event time; synthetic grouping; search, filtering, pagination, charts, analytics, or scoring; inference, psychological summaries, Pattern, Candidate/Hypothesis, or reasoning; cloud or external AI; multi-user support; Phase C; or Task 009.

## Validation guarantees

Focused Rust and frontend tests cover the scoped empty state without bootstrap, one and multiple subject behavior, corrupt ownership visibility, invalid relationships and domain rows, explicit relationship assembly, standalone records, deterministic save ordering, exact optional subjective conviction, safe errors, absence of write controls and raw SQL, bounded UI wording, and equivalent English and Simplified Chinese semantics. Valid records remain inspectable through the existing persisted SQLite foundation without a migration or dependency change.
