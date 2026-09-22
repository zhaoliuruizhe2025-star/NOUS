# Task 010 — Delete Foundation

Status: **OWNER-APPROVED — READY FOR GIT CHECKPOINT**. Task 010 becomes COMPLETE after the approved checkpoint is merged into `main`.

## Final approval

- Implementation validation: **PASS**
- Owner Code Review: **PASS**
- BLOCKER: **0**
- REQUIRED: **0**
- REVIEW: **0**
- Schema remains version **5**; migrations 0001–0005 are unchanged.
- Migration 0005 canonical Git-byte SHA-256: `0BDF05884817D645D6AC1C4D24D052F33A7435207FF745A94FC56387DC4B35CE`

## Purpose and scope

Task 010 lets the user deliberately delete one saved Situation, Observation, or Thought from the active local NOUS database. Delete means the user no longer wants NOUS to retain that logical record. Correction means NOUS represented a record inaccurately; UserUpdate means the user's state later changed. Neither Correction nor UserUpdate is synthesized by Delete. Other Self Model entities remain outside this task.

## Storage semantics

Delete physically removes the target entity row and all of its own typed Task 009 correction rows in one transaction. Correction provenance that refers to a *different* logical target is an external dependency and blocks Situation deletion. There is no tombstone, Trash, Undo, Restore, deleted-ID registry, or content-bearing deletion audit. The deleted logical ID no longer loads; normal system behavior must not deliberately reuse it, although Task 010 does not impose permanent schema-level ID non-reuse.

A successful delete removes the target and its own provenance from the current active database. It does not promise forensic erasure of SQLite storage pages, deletion from operating-system or user-created backups and exports, or removal of similar information independently stored in other records.

## External dependency boundary

Any valid external semantic reference blocks with `dependencyBlocked` and zero writes. A corrupt exact-target reference yields `dataInconsistent`. Dependency discovery starts from the exact target ID without first filtering by subject or EvidenceLink source kind. No external row is cascaded, detached, rewritten, or re-reviewed.

Situation dependencies are current Observation, Thought, Emotion, Memory, and Decision relationships; EvidenceLink Situation sources; and before/after Situation references in Observation and Thought correction provenance. Observation and Thought can each be EvidenceLink sources. A target's own correction rows are internal provenance, removed with that target.

## Transaction, stale state, and inspect result

The dedicated strict `delete_structured_record` command accepts only record type, target ID, and the opaque expected state token obtained from Task 008. It accepts no subject authority, dependency instruction, SQL, or generic patch. Opening and cancelling the confirmation make no write. Only explicit **Permanently delete** invokes the command.

One verified SQLite connection starts `BEGIN IMMEDIATE`. Within that transaction the repository validates the complete Task 008/009 inspect snapshot, resolves the single SelfSubject invariant, locates and reconstructs the target and its correction chain, compares the current server-side token to the expected token, and checks all exact external references. A stale token returns `staleDelete` with zero writes. A clean database without a SelfSubject has no target; orphan inspect data is inconsistent; multiple SelfSubjects fail safely.

After dependency checks pass, the transaction deletes the target's own correction rows and requires the affected count to match the validated chain length. It then deletes exactly one subject-guarded target row, validates the resulting complete structured-history snapshot on the **same transaction**, and commits. Any failure rolls back. The command returns the already validated in-memory history; it does not reload after commit. Existing Task 008 read DTOs remain active-record-only, without deletion metadata.

## User-facing meaning

English and Simplified Chinese both say **Delete record / 删除记录** and **Permanently delete / 永久删除**, distinguish Delete from Correction, explain that the target's correction history is removed, and state that the action cannot be undone in the current version. Dependency and stale-state errors are safe and do not expose SQL or dependent content. The UI makes no personalization-loss appeal.

## Validation

Focused tests cover deliberate confirmation, zero-write open/cancel/blocked paths, deletion of each supported type and its own correction rows, stale tokens, exact-target dependency and corruption classification, transaction rollback, post-delete history, close/reopen durability, strict command input, safe errors, bilingual copy, and the unchanged schema/dependency boundary.

Implementation validation: **PASS**. Rust: 117 unit tests and 3 documentation tests; frontend: 21 tests across 6 files. Cargo formatting, Clippy with warnings denied, TypeScript typecheck, ESLint with zero warnings, production build, and `git diff --check` passed. Standalone and corrected-target deletion remained durable after database close/reopen. Schema remains version 5; migration 0005 and dependencies are unchanged.

## Explicit non-goals

No Delete All, full Self Model deletion, external cascade or detachment, EvidenceLink mutation, tombstone, content-bearing deletion audit, Undo/Restore, export, backup, UserUpdate, Correction redesign, Pattern/Candidate, reasoning, cloud service, external AI/LLM API, Task 011, or Phase C work is authorized by this task.
