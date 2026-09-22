# Task 009 — Correction Foundation

Status: **OWNER-APPROVED — READY FOR GIT CHECKPOINT**. Implementation validation and final Owner Code Review passed. Task 009 becomes COMPLETE after this approved checkpoint is merged into `main`.

## Final approval

- Implementation validation: **PASS**
- Owner Code Review: **PASS**
- BLOCKER: **0**
- REQUIRED: **0**
- REVIEW: **0**
- Migration 0005 SHA-256: `0BDF05884817D645D6AC1C4D24D052F33A7435207FF745A94FC56387DC4B35CE`

## Objective and meaning

Task 009 lets the user deliberately correct an inaccurate NOUS representation of a saved Situation, Observation, or Thought. A **Correction** means the earlier stored representation was wrong. A **UserUpdate** means the user genuinely changed later. **Delete** means the user does not want the record retained. This task implements Correction only.

Superseded values in correction audit are provenance of what NOUS previously stored incorrectly. They are **not** evidence that the user once truly had that Thought, experience, or state. Future reasoning must use the current effective entity row; it must never treat an incorrect superseded value as historical user truth.

## Scope and user flow

- Situation: description.
- Observation: content and explicit relationship to an existing same-subject Situation or no Situation.
- Thought: content, the same relationship choice, and optional user-reported ThoughtConfidence.
- The logical record ID, subject ID, and original storage creation time never change.
- A correction note is optional and user-authored. Blank optional notes normalize to absent.
- The user opens a record in Task 008, chooses **Correct inaccurate record**, reviews and edits temporary fields, then explicitly confirms. Opening, editing, cancellation, stale errors, and no-op requests persist nothing.
- The UI uses equivalent English and Simplified Chinese correction semantics. It does not present Correction as generic Edit, UserUpdate, or Delete.

## Persistence and provenance

Model B stores the current effective representation in the existing entity row and appends an immutable, typed audit row in the corresponding Situation, Observation, or Thought correction table. Migration 0005 adds those three tables and advances schema version from 4 to 5. Older migrations are unchanged. No new dependency is required.

Each target's correction sequence begins at 1 and is contiguous. Audit rows contain validated complete before/after representations, before/after state tokens, an optional note, and correction-recording storage time. The latter is not event time and is not exposed by the Task 009 frontend.

Task 008 reads all three inspect-scope entity tables and all three correction tables in one consistent snapshot, without subject-filtering away corrupt rows. It verifies ownership, relationships, record reconstruction, sequence continuity, before/after chaining, token chaining, and agreement between the latest after-snapshot and the current entity row. A corrupt chain fails the entire inspect load with a safe `dataInconsistent` error. Task 008 displays current values plus compact, explicitly labeled correction provenance. Its grouping remains based only on persisted Situation relationships.

## Stale-write and EvidenceLink boundaries

Each inspected record includes a technical, non-visible state token. A correction request returns the token observed when its review form opened. Inside a `BEGIN IMMEDIATE` transaction the repository compares it with the current server-side head token. A mismatch returns `staleCorrection` with zero writes. Every successful correction receives a new opaque token, so an A → B → A value cycle cannot make an old form current again. The frontend token does not grant authority.

After validating the proposed representation and returning `noChanges` for an identical result, the transaction checks exact EvidenceLink source references by source kind and target ID **without first filtering by subject**. A valid same-subject reference returns `evidenceReferenceBlocked` with zero writes. A matching reference with conflicting ownership returns `dataInconsistent`. Task 009 neither changes nor re-reviews EvidenceLinks.

## Atomicity and architecture

The dedicated `correct_structured_record` command accepts a strict tagged request and no subject authority, audit identity, sequence, timestamp, generated token, SQL, or generic patch. The application constructs typed IDs and validates inputs. One focused repository transaction resolves subject and inspect integrity, loads the target, checks the token, validates the proposed domain value and relationship, handles no-op and EvidenceLink blocking, allocates the sequence, inserts typed audit, performs a guarded entity update, validates the resulting history, then commits. Every failure rolls back.

The correction audit is representation provenance in focused persistence/application types, not a new psychological Self Model entity. Existing Situation, Observation, Thought, ID, and ThoughtConfidence domain types remain authoritative.

## Validation guarantees

Focused checks cover migration 0001 → 0005 and existing v4 → v5, the three entity corrections, relationship and conviction changes, optional notes, stable IDs, repeated sequences, stale and A → B → A protection, no-op-before-EvidenceLink behavior, exact-reference ownership corruption, atomic rollback, malformed/orphan audit detection, correction-aware inspect, close/reopen durability, strict command input, safe errors, frontend-only review/cancel state, no raw SQL, and bilingual meaning. Full repository checks are required before Owner Code Review.

## Explicit non-goals

No generic Edit, UserUpdate lifecycle, Delete, Undo, Restore, export, backup, full version browser, Raw History, semantic event time, correction of other entities, EvidenceLink mutation or stale lifecycle, Pattern, Candidate/Hypothesis, reasoning, automatic NLP correction, generic audit framework, cloud service, external AI/LLM API, Task 010, or Phase C work is authorized by Task 009.
