# Task 011 — Export / Backup Foundation

Status: **IMPLEMENTATION VALIDATION: PASS. OWNER CODE REVIEW: PASS.** BLOCKER: **0**. REQUIRED: **0**. REVIEW: **0**. Schema remains v5; migrations are unchanged; migration 0005 canonical Git-byte SHA-256 remains `0BDF05884817D645D6AC1C4D24D052F33A7435207FF745A94FC56387DC4B35CE`. The approved direct dependency is `tauri-plugin-dialog = "=2.7.3"`. Task 011 becomes COMPLETE after its approved checkpoint is merged into main.

## Product boundary

Task 011 provides two distinct explicit local actions. **Export data** produces a portable, versioned UTF-8 JSON document of all currently persisted user/domain information. **Create database backup** produces a consistent logical SQLite snapshot of the current database. Neither action restores or imports data, uploads a file, or changes the Self Model. The user chooses a local destination in a native Save dialog. Closing the dialog cancels without a file or error.

## Portable JSON v1

The document has `format: "nous-user-data"`, `formatVersion: 1`, `sourceSchemaVersion: 5`, a generation-time `exportedAtMs`, and `subjectScope: "singleCurrentUser"`. Every collection is present, including when empty. Stable logical IDs identify records and relationships; records sort by ID, revisions by `revisionNumber`, and corrections by `sequence`. The format and keys are independent of UI language. Persisted user text is copied without translation or semantic rewriting.

The collections cover PersonReference, Situation, Observation, Thought, Emotion, Belief and every BeliefRevision, Value and every ValueRevision, Memory, Decision, Outcome, and EvidenceLink. Belief/Value anchors may have zero revisions under the existing domain contract. Revision `origin` keeps initial entry, genuine user update, and user correction distinct. EvidenceLink uses a typed source `{type,id}` and typed exact revision target with anchor and revision IDs, plus relation, provenance, user note, and saved time. No confidence, weight, or inferred score is added.

`representationCorrections` has separate Situation, Observation, and Thought arrays. Each event contains the target ID, sequence, `priorInaccurateRepresentation`, `replacementRepresentation`, optional user note, and `recordedAtMs`. A superseded prior value means NOUS previously stored an inaccurate representation; it is **not** a true historical user state. Current entity collections contain only current effective representations. `savedAtMs` is storage/save time, `recordedAtMs` is correction-recording storage time, and `exportedAtMs` is export-generation time; none is lived-event time.

The internal single-user SelfSubject placeholder (`self` / `Self`), raw `app_metadata` and `_sqlx_migrations` rows, correction state tokens, database and output paths, OS identity, SQL details, and UI state are excluded from portable JSON. Zero SelfSubjects with zero user/domain/correction rows yields a valid empty document without bootstrap. Orphan data or more than one subject fails safely.

## Snapshot and integrity

One verified SQLite connection and one read transaction load all approved domain tables and all three correction tables. Reads do not filter by subject before validation. The existing Task 008/009 loader validates Situation, Observation, Thought, and correction chains; the Task 011 loader additionally reconstructs every other domain row, validates subject ownership, storage timestamps, optional Situation links, Decision-to-Outcome links, Belief/Value revision ownership and sequence, and EvidenceLink source and exact revision target existence. Any malformed or inconsistent semantic data fails the entire portable export with `dataInconsistent`; no corrupt row is skipped or transformed into a successful normal JSON artifact. Serialization completes in memory before a file is published.

## Backup and artifact boundary

Backup runs fixed `VACUUM INTO ?` with a bound destination on a verified SQLx SQLite connection and no open transaction. It creates a consistent logical SQLite copy while the application database is open; byte-for-byte file-layout identity is not promised. The backup includes schema v5, tables, indexes, all rows, SelfSubject, correction tokens, app metadata, and SQLx migration state. The generated artifact is opened read-only and checked with SQLite `integrity_check`, schema metadata, migration state, and expected schema objects before success. This is structural backup validation, not a certification of psychological/domain semantics. A semantically inconsistent but structurally usable database may still be backed up.

Both operations generate a complete artifact in a unique temporary directory on the selected destination filesystem. The final path is never overwritten: an existing file yields `destinationExists`; publication uses a hard link that atomically fails if the final name appeared concurrently. Unsupported hard linking yields `destinationUnavailable`, with no rename/copy fallback. Temporary artifacts are removed on failure where possible. Paths are not persisted, returned to the frontend, or logged. Output receives no additional NOUS-specific encryption and is not uploaded automatically. A backup is not currently restorable inside NOUS.

## Architecture and validation

Persistence owns the full-domain validated snapshot and bound SQLite backup primitive. Application code owns portable JSON mapping, serialization, artifact lifecycle, and backup validation. The Rust command opens the native Save dialog using the approved `tauri-plugin-dialog = "=2.7.3"` dependency; the frontend invokes two focused payload-free commands and shows completed, benign cancelled, or safe-error states. No frontend filesystem/SQL authority, export-history table, or Self Model write is introduced. English and Simplified Chinese explain Export, Backup, privacy, lack of extra encryption, and lack of in-app Restore equivalently.

Schema stays at v5. Migrations 0001–0005 are unchanged. Migration 0005 canonical Git-byte SHA-256 remains `0BDF05884817D645D6AC1C4D24D052F33A7435207FF745A94FC56387DC4B35CE`. Cargo.toml and Cargo.lock change only for the approved Rust dialog plugin and its necessary transitive dependencies; no npm package, explicit FS plugin registration, new capability, cryptography, or archive dependency is added.

Implementation validation status: **PASS**. Rust: 126 unit tests and 3 documentation tests. Frontend: 24 tests across 7 files. Cargo formatting, Clippy with warnings denied, TypeScript typecheck, ESLint with zero warnings, production build, and `git diff --check` passed. Focused validation covered empty and complete portable exports, whole-export corruption refusal, correction semantics without state tokens, bound `VACUUM INTO` on an open file-backed database, SQLite backup integrity/schema/migration objects, semantically inconsistent but structurally usable backup, non-overwrite publication, deleted-row absence, and close/reopen export stability. Owner Code Review: **PASS**. BLOCKER: **0**. REQUIRED: **0**. REVIEW: **0**.

## Explicit non-goals

No Restore, Import, cloud backup/sync, scheduled backup, file sharing, custom encryption, ZIP/CSV/PDF export, conversation or Raw History export, semantic event time, Pattern/Candidate, reasoning, Phase C, or management of previously created artifacts is part of Task 011.
