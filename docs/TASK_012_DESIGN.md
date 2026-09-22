# Task 012 — Connect Architecture Foundation

Task 012 records the Owner-approved Phase C architecture for Candidate, Hypothesis, and Pattern. It is a documentation checkpoint. It authorizes no Phase C implementation, database schema, migration, source test, dependency, or Task 013 work. Phase B is closed; the Phase B Foundation Checkpoint passed. The approved starting main is `f976eaab1030e60f18d07b4a0f166a1165af62e5`, with schema v5. Foundation Audit v0.2 remains closed and the Reasoning Architecture Amendment remains authoritative.

## Purpose and concepts

Connect may notice a possible recurring structure across user records without defining the user's identity from outside. The user's authority over their identity remains final. A conclusion always retains its source type, proposer, context, temporal limits, relevant counterevidence, and revisability.

**Candidate** is a reviewable, falsifiable possible recurring structure worth examining. It may eventually be proposed by NOUS or the user, with proposer provenance preserved. It is not confirmed knowledge, an identity, trait, diagnosis, objective fact, or causal fact. A strong single report may prompt examination but cannot alone establish recurrence. Task 013's first slice permits only user-initiated, transient Candidates.

**Hypothesis** is a possible explanatory proposition considered during Psychological Modeling and Model Validation. It is a reasoning role, not a separate durable domain entity or lifecycle. A concrete, user-reviewable explanatory claim is represented as a Candidate. Co-occurrence does not establish a causal Hypothesis, and Task 013 does not confirm causal claims.

**Pattern** is a recurring, evidence-grounded, context-bounded and temporally bounded structure that has passed Model Validation and whose current claim and scope the user has explicitly accepted for durable Self Model use. Neither strong evidence alone nor silence can promote a Candidate. Acceptance is not objective proof. A Pattern remains revisable; it is not identity, diagnosis, immutable trait, causal claim by default, or certainty about the present. Task 012 creates no durable Pattern.

## Three independent dimensions

Evidence assessment, user acceptance, and applicability must not collapse into a promotion score or single one-way lifecycle:

| Dimension | Conceptual examples | Boundary |
| --- | --- | --- |
| Evidence assessment | possible; recurring evidence observed; well-supported | Qualitative with reasons. No count, score, weight, probability, or numeric system confidence. |
| User acceptance | unresolved; accepted; rejected; superseded | Explicit acceptance of the claim and its scope is required for a durable Pattern. Silence is not acceptance. |
| Applicability | context-bounded; historically supported; currently applicable; current applicability unknown; needs revalidation | Historical stability is not current certainty. A changed source can require revalidation even when the claim wording is unchanged. |

New evidence may strengthen, weaken, complicate, narrow, replace, or reject a claim. Scope narrowing cannot endlessly add ad-hoc exceptions merely to escape material counterevidence. ThoughtConfidence continues to mean only the user's subjective conviction in a Thought at the represented time.

## Grounding and epistemic type

Sources retain their approved meanings. Situation supplies context, not a psychological conclusion. Observation is a user report of noticed or experienced information, not independently verified external fact. Thought supplies cognitive content, not proof that its subject is true. Emotion is reported affect, not proof of an external event or another person's motive. A BeliefRevision is the user's explicit proposition at that exact revision, not objective truth; a ValueRevision is an explicit priority at that exact revision, not an inferred preference. Memory is retrospective experience and user meaning, not a verified event log. Decision is an actual reported choice, not hidden motive. Outcome is a reported later occurrence, not causal proof or a quality judgment. An EvidenceLink is one user-authored relationship assertion, not a second independent lived event. PersonReference supplies context for the user's experience and never authorizes a third-party psychological model.

Task 006 EvidenceLink remains a `UserAuthored` typed source pointing to an exact BeliefRevision or ValueRevision. It cannot silently become system-derived Pattern evidence. Derived provenance may conceptually distinguish supporting, contradicting, complicating, and contextualizing roles, but any **durable** derived provenance requires a separately approved typed design. Task 013 stores no derived provenance.

The actual evidence used for a review must remain distinguishable from unrelated available records. Reuse of one record, several records about one lived experience, dependent retellings, and materially different experiences are not automatically equivalent. Memory, Thought, and Observation can describe the same experience; multiple rows and repeated EvidenceLinks are not independent corroboration by default. Task 013 need not solve automated duplicate detection and must not claim independence from counts.

## Scope, time, and counterevidence

Claims identify the exact source Situations where relevant, describe the conditions and analysis scope, and say which broader domains are unsupported. Evidence from academic teamwork does not automatically apply to all collaboration, intimate relationships, or the person's identity. No universal life-category ontology is introduced.

Storage `created_at_ms` / `savedAtMs` is not lived-event time. It cannot establish recency, duration, improvement, deterioration, or current applicability. Unknown semantic time remains unknown; there is no fixed decay formula. A historically supported Pattern may no longer apply now.

Supporting, contradicting, complicating, and contextualizing material is evaluated against the same claim, comparable context and time, and possible evidence dependence. Mixed records may require a narrower claim, a weaker assessment, a different explanation, or rejection rather than binary true/false treatment. A missing counterexample within selected records is not proof of its absence elsewhere. Recurrence or co-occurrence does not establish causation.

The same discipline applies to negative and positive patterns. A bounded observation of withdrawal in several reviewed team conflicts can be examined without labeling the user bad at teamwork. Following through on several reviewed plans does not prove a general or permanent personality strength. PersonReference remains a contextual link: a Candidate may discuss the user's reported anxiety before interactions with P but cannot make a durable claim that P is manipulative, dislikes the user, or has a diagnosis.

## Representation identity and invalidation

Derived review distinguishes logical source identity from the representation actually used. For Situation, Observation, and Thought this may use logical ID plus the current representation state token. For Belief and Value, it uses exact revision identity. This conceptual boundary does not grant a token authority over the record.

Correction, change of a relevant representation, or deletion invalidates an affected prior derived evaluation until revalidation. Future reasoning uses the current effective source; superseded inaccurate correction values cannot remain active evidence. A corrected old evaluation cannot be presented as though its original conclusion had always rested on the new text. Deleted content must not be copied into derived provenance solely to keep a conclusion alive. Remaining evidence must be judged semantically, not by subtracting one from a count. The source Situation and other context on which an evaluation actually relied are part of this validity question.

If NOUS's Candidate or future Pattern wording was inaccurate, that is UserCorrection: the wrong interpretation is representation provenance, not a true historical user state. If the interpretation was accurate for an earlier period and the user later changed, that is genuine evolution/UserUpdate. Task 013 adds neither lifecycle.

## Explainability and reasoning placement

An answer to “Why did NOUS notice this?” must accurately disclose the claim, proposer, analysis scope, actual sources and their roles, relevant contexts, known and unknown time, material counterevidence, plausible alternative explanations where relevant, current qualitative assessment, and what could weaken or change it. If an automated rule was actually used in a future slice, its rule/version belongs to that actual basis. Do not fabricate post-hoc reasons or store raw internal chain-of-thought.

The approved order is: Grounding collects valid source representations and limitations; Psychological Modeling forms possible explanations and Candidate claims; Model Validation tests support, contradiction, dependence, scope, time, and alternatives; Reasoning uses relevant validated interpretations; Reasoning Validation checks unsupported causal, context, identity, or certainty jumps; Response Alignment checks relevance to the user's goal; Expression presents a bounded conclusion. Hypothesis is a role across modeling and validation. Candidate is the reviewable claim. A future user-accepted Pattern may be durable, but acceptance does not replace Model Validation. Existing Care/Safety priorities remain cross-cutting.

## Task 013 first implementation boundary

The next separately scoped task is **Task 013 — Transient Candidate Review Foundation**. Its approved conceptual first slice is user initiated: the user proposes Candidate wording and explicitly selects current Situation, Observation, or Thought records. Roles may be marked supporting, contradicting, complicating, or contextualizing. The user may adjust wording and scope. The review remains transient. No Candidate, Pattern, or Hypothesis is persisted; there is no automatic detection, automatic durable inference, LLM requirement, local-model requirement, or external model requirement.

If a selected record changes or is deleted before an existing review result is reused, that result cannot be treated as a valid current review. Task 013's independent design should use current representation identity and state-token validation where appropriate. Task 012 does not prescribe a persistence mechanism or implement that behavior.

Any later durable Candidate or Pattern capability brings Inspect, Correction, genuine update/evolution, Delete, source invalidation, typed provenance, Export, Backup, and explanation obligations. Task 012 authorizes no persistent derived layer. Task 013 requires separate specification and implementation authorization.

## Invalid shortcuts

The following jumps remain invalid: Observation → Trait; Emotion → External Fact; Thought → Fact; repetition/co-occurrence → causation; repetition → identity; historical Pattern → present certainty; context transfer without evidence; lack of evidence → negative fact; evidence count → truth or confidence; strong language → strong evidence. User silence → confirmation, user acceptance → stronger evidence, and multiple records → independent evidence by default are likewise invalid. No numeric Candidate/Pattern confidence, score, probability, or evidence weight is approved.

## Approved hard constraints and checkpoint boundary

1. Evidence assessment, user acceptance, and applicability are separate dimensions and must never be silently collapsed.
2. Source representation correction, relevant change, or deletion invalidates affected prior derived evaluation until revalidation.
3. Context scope, temporal limits and unknowns, evidence dependence, and analysis scope remain explicit.
4. New durable derived interpretation requires its own typed provenance and full data-sovereignty lifecycle; Task 006 `UserAuthored` EvidenceLink semantics cannot be borrowed.

This documentation checkpoint changes no code, migration, schema, test, dependency, or current Task 006–011 behavior. The Foundation Audit remains closed; Phase B remains closed; Phase C implementation has not started; Task 013 has not started. Architecture Owner Review: **PASS**. BLOCKER: **0**. REQUIRED: **0**. REVIEW: **0**. Task 012 becomes COMPLETE after its approved checkpoint is merged into main.

Status: **ARCHITECTURE APPROVED — OWNER REVIEW PASSED**.
