# NOUS Foundation Audit v0.2

**Record status:** The original audit, classifications, findings, residual risks, and verdict below are preserved as the pre-correction record. See **Owner Disposition and Bounded Corrections** at the end for the subsequent owner decisions and current documentation-correction status.

## Scope

This is the finite Round 2 architecture consistency audit against the owner's approved Foundation baseline A–R. It examines governing documentation and the actual implementation through Task 006. It does not reopen the Concept Audit, authorize corrections, redesign NOUS, or begin Task 007 or Round 3. Only actual discrepancies and unresolved semantic ambiguity are findings; absent future functionality is not a defect.

### Repository verification

On 2026-09-20, the initial repository state was verified in `D:\Dev\NOUS`:

- Branch: `main`.
- `HEAD`, local `main`, and the local remote-tracking reference `origin/main` all resolved to `7b7ba78bc55772bb8de795a0babd548038c8286b`.
- Working tree: clean, with no untracked files reported.
- Approved history includes Task 004 merge `0643ba2`, Task 005 merge `717d142`, and Task 006 implementation `a924780` / merge `7b7ba78`.
- No Task 007 specification/implementation or Foundation Audit correction was present in the inspected checkpoint or working tree.
- Created and switched to `planning/foundation-audit-v0.2` from that checkpoint. Git reference creation required sandbox permission; no existing branch or commit was changed.

The upstream comparison above is against the repository's `origin/main` reference; no fetch or remote write was performed. Historical planning status text describing Tasks 004–006 as future/proposed was not allowed to override the approved Git history. Such stale status text is not a Foundation finding or a request to rewrite completed task history.

### Material inspected and checks

The review covered `README.md`, `START_HERE.md`, `AGENTS.md`, and the Product, Principles, Self Model, Architecture, Decisions, Interaction Model, Response Policy, Task System, Master Plan, Roadmap, Safety, Onboarding, Voice/i18n, UI Boundary, Development Protocol, Backlog, Frameworks, Academic Foundation, and Relationship Lens documents. It also inspected the Task 002–006 specifications and designs, and the Task 004–006 feasibility documents.

Implementation inspection covered every production Rust domain module, validation and serialization, repository operations and row reconstruction in `persistence.rs`, migrations 0001–0004, migration registration and pool setup, the Tauri command surface, frontend database/UI boundary, manifests/configuration/capabilities, domain tests, and relevant persistence/migration/history/security tests. The actual application boundary currently exposes database readiness only; there is no domain capture application service or domain command connected to the frontend. This is an unimplemented workflow, not an inference or authorization bypass.

Existing checks were run without adding tests or dependencies:

| Check | Result |
| --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --offline --locked` | PASS: 80 tests and 3 compile-fail documentation tests; no failures. Existing linker output produced an import-library creation warning. |
| `npm test -- --run` | PASS: 7 tests in 2 files, including all 4 frontend database-boundary checks. |
| Initial sandbox attempts | Cargo build-lock access and Vite child-process startup were blocked; approved reruns passed. These were execution-environment failures, not product test failures. |
| Git scope and whitespace checks | No tracked-file changes; only `docs/FOUNDATION_AUDIT_V02.md` is untracked. `HEAD`, `main`, and `origin/main` remain at the approved checkpoint. No whitespace errors were reported for the audit document. |

This was not a release-build, live UI, or behavioral validation of future reasoning/Care systems. Tests used their existing isolated fixtures; the user's application database was not inspected. Only this audit document is added; production code, tests, migrations, dependencies, and earlier documents remain unchanged. No commit, merge, or push was performed.

## Approved Concept Baseline

NOUS is a local-first personal cognitive system offering an informed external perspective, grounded conclusions, practical and emotional support, and requested directional advice while leaving final authority with the user. Normal operation must not depend on external AI services or token billing.

Current interpretation, revisable candidate hypotheses, and confirmed durable knowledge are distinct. Situation is context; Observation is user-reported experience; Thought is momentary cognitive content; Emotion permits multiple independent felt states; Belief is a relatively stable endorsed proposition; Value is a meaningful priority. Memory is a user-selected retrospective experience with user-owned meaning. Decision records an actual choice and may have multiple Outcomes; aftermath is distinct from decision quality or retrospective evaluation.

Evidence supports, contradicts, complicates, or contextualizes an exact interpretation without becoming proof or gaining weight from duplicate storage. Patterns remain contextual and revisable. Time, context, genuine change, and correction must remain distinguishable. Negative experiences and supported difficulties are legitimate; neither flattery nor identity labeling replaces accurate understanding.

Natural interaction, limited skippable calibration, relevant use of history, evidence-sensitive language, and adaptive delivery support the user without forcing ontology forms or constant confirmation. Care is warm and truthful; immediate safety takes precedence over analysis. The model belongs to the user, preserves third-party boundaries, and must ultimately support inspection, correction, export, and deletion without manipulation. No new Thinking/Reasoning or Mood entity is authorized.

### Classification register

Each row assigns exactly one classification to the named portion of the approved baseline. Composite sections are split where an implemented invariant and an absent future capability need different classifications. `CONSISTENT` means the present design/implementation does not contradict the principle; it does not certify future behavior that has not been built.

| Baseline principle assessed | Classification | Repository basis / finding |
| --- | --- | --- |
| A — Local personal cognitive system, external perspective, mirror rather than identity authority; no required external AI | CONSISTENT | Product/Principles/Architecture; local SQLite and readiness-only shell; no application AI/network call path. |
| A — Useful directional advice and clear grounded conclusions when warranted | MISMATCH | Governing advice prohibitions and blanket tentative-response wording; F01–F02. |
| B — Current interpretation does not automatically become confirmed durable truth | CONSISTENT | Self Model epistemic layers; no inference pipeline; commitment origins exclude system inference. |
| B — Long-lived, traceable, correctable candidate hypotheses separate from confirmed knowledge | NOT YET IMPLEMENTED | No Candidate/Hypothesis entity, persistence, inference, or workflow; separate future proposals are explicitly contemplated. |
| B — Calibration diminishes as evidence matures, without constant confirmation | MISMATCH | Reflection policy directs confirmation/rejection and tentative presentation without an evidence-maturity exception; F02. |
| B — Stable/high-confidence models remain revisable, never complete or immutable | CONSISTENT | Principles §4 and Self Model §§8–9; immutable stored records do not declare interpretations eternally true. |
| C — Current-user ownership, contextual third parties, uncertain third-party motives | CONSISTENT | Typed SelfSubject ownership; contextual PersonReference; no PersonReference Evidence source/target; Interaction Model's epistemic boundary. |
| D — Situation / user-reported Observation / momentary Thought remain distinct; no authorized Thinking/Reasoning entity | CONSISTENT | Separate domain types/tables and explicit task semantics; no automatic conversion or new reasoning entity. |
| E — Multiple emotions with different strengths/context, distinct from composition or system certainty; no Mood entity | CONSISTENT | Independent Emotion rows allow a shared Situation; no exclusivity, normalization, composition-percentage output, or Mood entity. See the numeric explanation below. |
| E — Emotion temporal change and correction workflows | NOT YET IMPLEMENTED | Storage creation time only; no emotion correction/inference workflow; no architectural prohibition on later approved semantics. |
| F — Thought / Belief / Value distinctions, user endorsement rather than objective truth, meaningful priorities, change and correction | CONSISTENT | Separate domain objects; explicit UserUpdate/UserCorrection; no automatic Thought/behavior promotion. |
| F — Personalized examination of unsupported beliefs instead of reinforcing them | NOT YET IMPLEMENTED | No advice/reasoning engine; contradicting and complicating evidence can already be stored. |
| G — Selected retrospective Memory, optional user meaning, no automatic promotion or positive-only restriction | CONSISTENT | Task 005 design and `Memory`; separate description/user_meaning; no valence filter or automatic authoring. |
| G — Time-aware comparison of past thoughts and successive retrospective meanings | NOT YET IMPLEMENTED | Separate record concepts exist, but semantic times and lived-experience correction/history workflows were explicitly deferred. |
| H — Actual Decision, zero-to-many Outcomes, outcome distinct from evaluation/causal proof/decision quality | CONSISTENT | Task 005 semantics and migration 0003; mandatory same-subject Decision FK, no unique decision_id, scores, or evaluation field. |
| H — Recurring Decision → Outcome relationships used as nondeterministic advice context | NOT YET IMPLEMENTED | No pattern or advice engine; storage ordering does not claim causal chronology. |
| I — Four evidence relations, traceable basis, exact historical revision, no proof/weight/duplicate corroboration, counterevidence permitted | CONSISTENT | Task 006 domain, migration 0004, repository, and tests; Emotion/Situation contextual-only; no aggregation or target mutation. |
| J — Contextual revisable patterns and long-term self-description, including difficulties and change without identity labels | NOT YET IMPLEMENTED | No Pattern engine or self-description generator; provisional/correctable design, no flattering or negative-label classifier. |
| J — Natural interaction rather than mandatory ontology maintenance or recurring raw-model approval | CONSISTENT | Product §2.1, Decisions D016, UI Boundary §4; current scaffold contains no ontology-entry requirement. |
| J — Natural-language understanding and conversational capture | NOT YET IMPLEMENTED | No parser, conversation store, or capture workflow; temporary structured prototypes do not redefine normal product interaction. |
| K — Context-sensitive temporal interpretation; old patterns can weaken and improvement/deterioration can be recognized | NOT YET IMPLEMENTED | Context links and commitment revision sequences provide foundations; no temporal pattern inference, recency policy, or anchoring behavior exists. |
| L — Revision differs from correction; historical records preserved without asserting an incorrect interpretation was true | CONSISTENT | Distinct RevisionOrigin values and persisted checks; no system-inference origin or history renderer conflating the two. |
| L — Natural-language rejection/correction, canonical interpretation, and multiple retrospective evaluations | NOT YET IMPLEMENTED | Governing correction semantics exist; system interpretation, presentation, and broader correction workflows do not. |
| M — Nondiagnostic authority, truthful warm care, safety before detached analysis, no romanticized/nihilistic crisis framing | CONSISTENT | Safety §§2–9, Principles §§9–10, Voice care/anti-template rules, and Interaction Model; no contrary runtime response behavior. |
| M — Health-context handling and operational Care/Safety detection, routing, and support | NOT YET IMPLEMENTED | No diagnosis or health-context engine; Care remains an explicit future pre-beta implementation gate. |
| N — Response need differs from delivery style; context/mixed signals matter; minimal useful questions and relevant history only | CONSISTENT | Interaction Model dimensions and Grounded Strength modifier explicitly reject a rigid anger route; Response Policy makes action-first optional. Mode examples do not establish an exclusive runtime enum. |
| N — Adaptive routing/composition that combines priorities and safety overrides | NOT YET IMPLEMENTED | No response-mode classifier, composer, or history injection exists. |
| O — Language strength tracks evidence, including clear revisable conclusions | MISMATCH | Blanket tentative wording remains in governing response guidance; F02. |
| O — Meaning of the existing stored numeric ThoughtConfidence | AMBIGUOUS | Range is implemented; whose confidence it describes is not established; F04. No calculated psychological probability was found. |
| P — Requested grounded directional advice, scaled to stakes/reversibility/evidence, with final user authority | MISMATCH | Explicit recommendation ban and slight-leaning ceiling; F01. Advice generation itself remains absent. |
| Q — Local data sovereignty, no silent upload/cloud inference, third-party privacy, future inspection/export/deletion remain possible | CONSISTENT | Local database, narrow capabilities, no transmission path; RESTRICT is an integrity policy with deliberate future privacy deletion explicitly allowed. |
| Q — User-facing inspection/correction/export/deletion, including future candidate data | NOT YET IMPLEMENTED | No such product workflows or candidate store exist; their absence is not a failure. |
| R — Limited, natural, skippable calibration without compelled disclosure | MISMATCH | Onboarding is expressly mandatory and specifies no skip path; F03. |
| R — Implemented onboarding and informative, non-manipulative deletion/leaving communication | NOT YET IMPLEMENTED | No onboarding runtime or deletion flow/copy exists; no coercive deletion behavior was found. |

## Architecture Consistency Findings

### F01 — Governing advice restrictions contradict approved advice authority

**Severity:** REQUIRED

**Concept principle affected:** A and P.

**Existing file/code location:** `AGENTS.md:125` (§6); `docs/PRINCIPLES.md:38` (§3) and `:153` (§13.1); `docs/PRODUCT.md:78` (§5); `README.md` (Life Paths and core principles); `docs/INTERACTION_MODEL.md` §8.1; `docs/RESPONSE_POLICY.md` §3.4; `docs/DECISIONS.md` D017; `docs/VOICE_AND_I18N.md:98`.

**What the current repository actually does:** Standing agent instructions say NOUS must never output “NOUS recommends X.” Principles says NOUS provides descriptive prediction, not normative recommendation. Product excludes a “life-advice recommender.” Newer guidance permits only a “slight” directional leaning and still retains the recommendation-language prohibition. No advice engine implements these restrictions yet.

**Why this conflicts with the approved concept:** The approved baseline explicitly permits and expects requested, evidence-grounded recommendations, including a recommendation to wait before an irreversible choice. Retaining user authority and distinguishing prediction from recommendation do not require withholding advice or always limiting it to a slight leaning. These are conflicting governing instructions, not simply different wording for user agency.

**Smallest correction boundary:** A separately authorized documentation correction should replace the blanket prohibition/slight-only ceiling with the approved advice boundary across these governing passages. Preserve final user choice, uncertainty appropriate to stakes and reversibility, no imposed worldview, and no primary-UI winner score. Do not implement advice, alter prediction semantics, or rewrite completed task specifications.

### F02 — Reflection guidance requires tentativeness and confirmation irrespective of evidence maturity

**Severity:** REQUIRED

**Concept principle affected:** A, B, and O.

**Existing file/code location:** `docs/RESPONSE_POLICY.md:87–88` (§3.3), `:181` (§6), and `:238` (§11); `docs/PRODUCT.md:201` (§9).

**What the current repository actually does:** Reflection goals include surfacing patterns tentatively and asking the user to confirm or reject interpretations. The deeper-inference threshold requires tentative presentation even when a statement matches a well-established recurring pattern. The historical-grounding rule says the connection “must remain” tentative. Interaction Model §§5 and 8 separately discourage ending every turn with a question and limit clarification, but do not remove these mandatory tentative/confirmation directions.

**Why this conflicts with the approved concept:** The baseline permits direct grounded conclusions when evidence is strong and expects fewer calibration questions as the relationship matures. Correctability is required; perpetual tentative expression and explicit confirmation are not. The repository's general question-restraint guidance does not reconcile this specific conflict. There is no current response engine to accuse of repetitive questioning; this is a governing-design mismatch.

**Smallest correction boundary:** Clarify these response/product passages so certainty of language and calibration frequency follow evidence and current need. Keep weak inferences tentative, important conclusions traceable and correctable, and confirmation where materially useful. This correction does not relax deliberate user intent for Task 006 EvidenceLinks or merge future candidate hypotheses into canonical revisions. No Candidate Layer or response implementation is required by this finding.

### F03 — Mandatory onboarding conflicts with skippable calibration

**Severity:** REQUIRED

**Concept principle affected:** R.

**Existing file/code location:** `docs/ONBOARDING.md:7` (§1), read with §§2–3.

**What the current repository actually does:** The approved-future-design document calls onboarding “mandatory,” creates a Bootstrap Self Model through its questions, and transitions to normal conversation after onboarding. It limits length and avoids clinical questionnaires, but provides no skip provision. There is no implemented onboarding screen or runtime gate.

**Why this conflicts with the approved concept:** The baseline allows limited high-value questions while explicitly allowing users to skip. A mandatory calibration stage with no stated ability to decline questions leaves a conflicting future interaction requirement in place. This finding does not claim that the existing app currently coerces disclosure.

**Smallest correction boundary:** Amend only the governing onboarding requirement to permit skipped questions and continuation with limited context, without requiring sensitive disclosure or implying NOUS cannot work without it. No new onboarding design or implementation is necessary for this audit correction.

### F04 — ThoughtConfidence has a numeric contract but an unresolved epistemic meaning

**Severity:** REVIEW

**Concept principle affected:** O, with implications for B and D.

**Existing file/code location:** `src-tauri/src/domain/entities.rs:157` (`Thought`); `src-tauri/src/domain/primitives.rs:140`; `src-tauri/migrations/0002_create_self_model.sql` (`thoughts.confidence`); `src-tauri/src/persistence.rs:1607` (`ThoughtRow`); `docs/SELF_MODEL.md` §2.3; `docs/TASK_002_DESIGN.md` §5; `src-tauri/src/domain/tests.rs:24`.

**What the current repository actually does:** A Thought may carry `ThoughtConfidence`, an integer in `0..=100`, stored and reconstructed without a provenance or semantic definition for that number. Tests attach `65` to “They may not care about me.” Governing text calls the field confidence and requires bounds, but does not establish whether it means the user's conviction in their own thought, NOUS's confidence in interpreting that thought, or a truth probability. In contrast, BeliefEndorsement and ValueImportance are explicitly defined as user-entered quantities distinct from system certainty.

**Why this conflicts with the approved concept:** This is unresolved ambiguity, not proof that NOUS currently computes a forbidden probability. The approved baseline requires separating user cognitive content from system interpretation and forbids unsupported psychological precision. The repository does not supply enough evidence to select a meaning safely before a capture workflow starts collecting this field.

**Smallest correction boundary:** Owner judgment is needed on the intended meaning and permitted source of `ThoughtConfidence`, or whether it should remain unused by capture pending a later approved decision. Record that resolution in current governing guidance. Do not silently reinterpret stored numbers, choose a new scale, modify migration 0002, or implement a scoring system. Any required code/data change must be a separately authorized bounded task.

## Confirmed Consistencies

- **No automatic promotion or psychological persistence.** Thought, Observation, Emotion, Belief/Value revisions, and lived-experience records have separate types and tables. The repository inserts the record supplied to each explicit operation; it contains no NLP extraction, inference, repeated-Thought promotion, behavior-to-Value conversion, or system-authored Memory meaning. `RevisionOrigin` contains only InitialUserEntry, UserUpdate, and UserCorrection. No hidden Candidate Model is present.

- **Emotion is not one exclusive user state.** `Emotion` is one record, not a single global field. Migration 0002 permits many Emotion rows with the same subject/Situation, and `constructs_valid_related_domain_records` uses sadness intensity `70` and anxiety intensity `55` together. These independent bounded intensities are not normalized shares and do not sum to 100. The internal helper name `define_percentage!` does not, by itself, establish emotional-composition percentages or system confidence. No numeric emotion output/calculation exists. Emotion semantic times/correction are deferred rather than falsely inferred from storage time.

- **User-reported facts and interpretations remain distinct.** Task 005's product-boundary table explicitly says an Observation is not verified world truth. EvidenceSource preserves the source family, including Thought as interpretation and Memory as retrospective recollection. There is no third-party motive classifier. Free-text user reports can mention people; that does not convert the text into the system's factual endorsement.

- **Third-party ownership is structurally bounded.** PersonReference contains contextual names, relationships, and notes only. Commitment owners use SelfSubjectId, not PersonReferenceId; compile-fail tests protect the distinction. Evidence has no PersonReference variant or generic type/table selector. The storage API can represent different subject IDs for referential integrity tests; no production workflow turns another person into a modeled SelfSubject.

- **History and correction have distinct stored semantics.** `commitments.rs` explicitly distinguishes UserCorrection from genuine UserUpdate. Atomic anchor-plus-initial-revision creation and repository-generated append sequences use `BEGIN IMMEDIATE`; earlier rows are preserved. Migration 0002 independently constrains origin and sequence. Preserving an earlier incorrect representation for provenance is not a claim it was a true past user state; no current renderer or inference engine makes that claim.

- **Memory, Decision, and Outcome retain their approved meanings.** Memory stores an optional user-authored meaning separately from its description and can omit a Situation. Decision records a reported choice. Outcome must reference a same-subject Decision; many Outcomes per Decision are allowed. The create operations do not synthesize Situations or classify outcomes as good/bad, prove causality, or evaluate decision quality. No positive-only content filter exists.

- **Evidence implements the approved narrow contract.** `evidence.rs`, migration 0004, and the four repository operation families preserve seven typed source families, exact commitment-revision targets, the four relations, and UserAuthored provenance. Situation and Emotion only contextualize. SQL checks and composite FKs enforce shape, existence, anchor/revision identity, and subject ownership. Reads reconstruct validated records and reject invalid shapes/values. Creating a link never changes a target; old links do not move to new revisions. Contradicts and Complicates are fully available. Duplicate assertions remain separate records but no count, confidence, ranking, or aggregation interprets them as stronger evidence.

- **Deliberate authorship remains an explicit future workflow obligation.** Task 006 requires intentional creation of the exact relationship. A Rust constructor cannot independently prove human intent, but the current app exposes no Evidence command or automatic caller that could falsely claim it. The UserAuthored-only enum is not being used for inferred relationships.

- **Persistence is local and protected from frontend SQL.** `lib.rs` registers migrations 1–4 through the SQL plugin, then installs the exact preloaded `sqlite:nous.db` pool. Repository operations verify foreign keys on the connection they retain and execute through. The sole app command is fixed `database_status`; frontend capabilities contain only `core:default` and no `sql:*` permissions. The TypeScript side receives schema readiness, not a database handle or arbitrary query interface. Source/configuration inspection found no application AI calls, API-key collection, token billing, telemetry, or psychological-data upload. Development URLs/schema references and the local asset origin are not such dependencies.

- **Deletion is deferred, not architecturally prohibited.** `ON DELETE RESTRICT` prevents accidental history loss; it is not immutable storage enforcement against the owner. Task 004 design §3.1 explicitly permits future authorized deletion in dependency order. The present explicit FK graph does not form an unavoidable deletion cycle, and there are no undeletable-record triggers or remote copies in application code. This supports future design feasibility, not a claim that privacy deletion/export already works.

- **Natural interaction, Care, and context remain compatible future directions.** Product and UI Boundary require normal conversational input, not manual ontology maintenance. Interaction Model separates delivery dimensions, allows mixed contextual signals, rejects anger-to-mode keyword routing, and excludes irrelevant history. Safety gives immediate human support precedence, forbids diagnosis and romanticized death, and requires reviewed bilingual care copy. No implemented response engine introduces flattery, broad negative identity labels, medical causal explanations, nihilistic crisis messaging, or detached crisis analysis.

## Future Principles Not Yet Implemented

These are the `NOT YET IMPLEMENTED` portions in the register, not additional correction work:

- Natural-language interpretation, Raw History/conversation storage, natural capture, and onboarding/calibration runtime.
- A persistent Candidate/Hypothesis Layer, source-backed hypothesis lifecycle, and inspection/rejection of derived interpretations.
- Pattern detection and long-term self-description that can recognize strengths, difficulties, context-dependent differences, improvement, deterioration, and weakening old patterns without identity labels.
- Semantic event/effective times and temporal comparisons. Current `created_at_ms` records storage creation only; revision_number supplies commitment ordering. They do not establish when life events happened.
- Correction/history workflows beyond existing user-authored Belief/Value revisions, including Emotion, Memory, Decision, Outcome, and Evidence assertions; successive retrospective evaluations and current canonical interpretation.
- Reasoning that examines beliefs, uses counterevidence, evaluates Decision/Outcome history without causal certainty, and provides requested advice. The advice-policy contradiction is F01; the absent engine is not a defect.
- Adaptive response composition, evidence-sensitive language generation, relevant historical personalization, and operational Care/Safety routing. No readiness indicator certifies these future capabilities.
- Data inspection/export/privacy deletion workflows and informative deletion/leaving communication. Future candidate data remains subject to the same sovereignty boundary.

No new Thinking/Reasoning or Mood domain entity is requested. No deferred feature is promoted into Task 007 by this audit.

## Residual Risks

1. **The three governing mismatches remain live instructions.** Without separately authorized correction, later work could obey the advice ban, blanket tentative/confirmation wording, or mandatory onboarding instead of the approved baseline. F01–F03 define the entire identified correction boundary.
2. **ThoughtConfidence remains semantically unresolved.** Passing range/serialization tests cannot establish whose confidence a stored number represents. F04 needs owner disposition before a capture workflow gives that number a user-facing meaning.
3. **Foundation tests do not validate future psychological behavior.** They verify domain/storage/security invariants. Semantic event time, non-commitment corrections, candidate lifecycle, and response/Care behavior remain absent. The existing design explicitly defers them; this limits what the audit can certify and does not demand their immediate implementation.

No additional implementation-level Foundation contradiction was found. No speculative risk backlog or optional cleanup is proposed.

## Round 2 Verdict

PASS WITH REQUIRED CORRECTIONS

The implemented foundation through Task 006 is compatible with the approved conceptual baseline. Three current governing-document contradictions require bounded correction before Task 007 implementation: advice authority, evidence-sensitive conclusions/calibration, and skippable onboarding. One stored-field semantic ambiguity requires owner review. None requires rejecting or redesigning the completed storage/domain foundation on the evidence inspected.

The audit is complete and stops for owner review. No corrections have been implemented, and no later task or audit round has begun.

## Owner Disposition and Bounded Corrections

Following technical review and owner review of the original **PASS WITH REQUIRED CORRECTIONS** verdict, the owner authorized only documentation corrections for F01–F04. The original findings above describe the inspected starting state; their wording and original file/line references are historical, not unresolved governing instructions after this correction pass.

| Finding | Owner disposition | Documentation correction |
| --- | --- | --- |
| F01 | **OWNER ACCEPTED — bounded governing-doc correction authorized.** | Removed blanket recommendation prohibitions and permanent slight-leaning limits from `AGENTS.md`, `README.md`, `docs/PRINCIPLES.md`, `docs/PRODUCT.md`, `docs/INTERACTION_MODEL.md`, `docs/RESPONSE_POLICY.md`, and `docs/VOICE_AND_I18N.md`. Marked D017's original advice clause in `docs/DECISIONS.md` superseded while preserving its history. Requested advice may be clear and grounded, with final user authority, evidence/stakes/reversibility-sensitive strength, and no imposed worldview, blind reinforcement of self-defeating behavior, or newly authorized primary UI scoring/ranking/winner mechanics. |
| F02 | **OWNER ACCEPTED — bounded governing-doc correction authorized.** | Updated `docs/RESPONSE_POLICY.md` §§3.3, 6, and 11 and `docs/PRODUCT.md` §9. Weak evidence remains tentative; stronger relevant evidence permits direct grounded conclusions and generally fewer confirmation questions. Important interpretations remain traceable, revisable, and correctable/rejectable. Candidate hypotheses remain separate from confirmed knowledge; deliberate intent for EvidenceLink creation is unchanged. |
| F03 | **OWNER ACCEPTED — bounded governing-doc correction authorized.** | Updated `docs/ONBOARDING.md` §§1 and 3 to allow skipped questions or onboarding, no compelled sensitive disclosure, and continuation with limited initial context while normal interaction builds understanding. The guided experience remains; mandatory psychological intake does not. |
| F04 | **OWNER RESOLVED — ThoughtConfidence is optional user-reported subjective conviction in that Thought at that time; not NOUS confidence or objective truth probability; NOUS does not automatically calculate it.** | Recorded the meaning and capture boundary in `docs/SELF_MODEL.md` §2.3. Numeric conviction requires explicit user provision/selection; qualitative certainty does not authorize exact numbers. Calibration may be limited and skippable, without routine rating prompts. No existing stored values are reinterpreted. |

**Final status: Round 2 COMPLETE.** The owner reviewed the bounded correction diff: F01 PASS, F02 PASS, F03 PASS, and F04 PASS / resolved. The final `PRINCIPLES.md` §3 wording correction also passed owner review. No BLOCKER or REQUIRED issue remains within Round 2 scope. The original **PASS WITH REQUIRED CORRECTIONS** verdict remains the historical pre-correction audit result. Round 3 has NOT started, and Task 007 has NOT started.

The bounded correction pass changed only the ten governing Markdown files named above and this audit record. No production source, migration, test, manifest, lockfile, completed Task 002–006 specification, or unrelated roadmap/status text is changed. No advice, confidence, candidate, response, inference, or onboarding engine is implemented. Before this authorized finalization, no commit, merge, or push had been performed. Task 007 and Round 3 work remain unstarted.

**Correction validation:** Focused searches and manual diff inspection found no remaining live governing contradiction within F01–F04. D017's old advice clause is explicitly superseded; original audit findings and completed task records remain historical. References to tentative hypotheses, optional gentle invitations, and confirmation for canonical knowledge/EvidenceLink creation do not require perpetual tentative language or constant conversational confirmation. Every changed passage is attributable to F01–F04 or this disposition/status record.

At correction validation, `git diff --check` passed and the audit document was checked for trailing whitespace. The branch was `planning/foundation-audit-v0.2` at `7b7ba78bc55772bb8de795a0babd548038c8286b`, with ten modified tracked Markdown files and this untracked audit document; nothing was staged. Existing Rust/frontend suites were not rerun for these documentation-only corrections. Their results earlier in this document belong to the original audit.
