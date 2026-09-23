# Task 014 — System-Proposed Candidate Architecture Foundation

## Status and authority

Architecture review: **APPROVED** by the Owner. Architecture Owner Review: **PASS**. Documentation Owner Review: **PASS**. Task 014 becomes COMPLETE only after its approved checkpoint is merged into main. This document records architecture, not implementation authorization.

Starting main: `340af5e99a2e1c075b6c01a8426a5d5b8a65a013`. Phase B is closed. [Task 012](TASK_012_DESIGN.md) is complete; Task 013 Transient Candidate Review Foundation is complete. Schema remains v5 with migrations 0001–0005. Foundation Audit v0.2 remains closed and the [Reasoning Architecture Amendment](REASONING_MODEL.md) remains authoritative.

**Capability verdict: AUTONOMOUS PSYCHOLOGICAL-SEMANTIC CANDIDATE GENERATION NOT CURRENTLY AUTHORIZED.** The prior capability blocker is resolved for this documentation checkpoint by withholding that capability. It is not a blocker for Phase C as a whole. Documentation-checkpoint findings: **BLOCKER: 0; REQUIRED: 0; REVIEW: 0**.

**Selected next direction: TRANSIENT USER-ASSISTED SEMANTIC COMPARISON PREREQUISITE.** Task 015 has **NOT STARTED** and requires its own Stage 1 Scope + Owner Review. No Task 015 semantics or implementation are authorized here.

## Why current records cannot ground autonomous semantic proposals

Current Situation, Observation, and Thought data can identify record type, explicit Situation relationship, current effective representation, stable logical ID, and representation state token. A structural connection such as “these Thoughts are linked to these Situations” follows from stored relationships. Current approved mechanisms cannot reliably determine that Situation text means an “evaluative context,” that different Thought texts both express “self-criticism,” or that natural-language records provide relevant psychological counterevidence. They also lack reliable experience independence and semantic event time.

A shared Situation can reflect one capture context rather than multiple experiences. Multiple rows do not establish recurrence; different Situation IDs do not establish independent experiences; the same Situation ID is not a complete model of dependence. Same words do not necessarily have the same meaning, and different words do not necessarily have different meanings. Storage time is not lived-event time. Unknown dependence or time remains unknown.

Structural association must not cross into psychological interpretation without an approved semantic grounding mechanism. Adding “may” to unsupported generated wording does not repair the missing basis. Pure structural display may remain useful infrastructure, but the Owner has not selected it as the next standalone system-Candidate product slice.

## Trigger, quantity, and claim boundary

Any eventual first system-Candidate analysis must begin only after the user explicitly selects or authorizes the analysis material **and** explicitly starts analysis. App startup, ordinary conversation, structured Save, background scans, and merely entering an analysis page are not triggers. Ordinary data capture does not imply authorization for psychological analysis.

An eventual first approved system-generated slice may return **zero or one** transient Candidate per analysis. Zero is a valid successful result when grounding is insufficient. This limit controls the interaction; it is not a confidence threshold, ranking, record-count test, or proof that no other possibilities exist.

The first eventual system-generated Candidate slice is descriptive only. It must not generate a causal claim. Co-occurrence does not establish that one condition caused another. Neither positive nor negative claims receive a weaker evidence standard. A system Candidate must concern the current user's experience, never an unconsenting third party's personality, hidden motive, diagnosis, or predicted behavior. A Candidate remains an unconfirmed, falsifiable interpretation, not a durable Pattern or accepted Self Model knowledge.

Analysis scope is the exact material examined; claim scope is the conditions to which the proposed interpretation says it applies. Both remain explicit. An absent counterexample among examined records does not imply one is absent from the Self Model or the user's life. A generated scope cannot silently transfer from specific situations to all relationships, teamwork, or identity. Missing semantic time prevents claims of recency, duration, improvement, deterioration, or current applicability.

## Contract for any future system mechanism

Before implementation, a concrete mechanism must define:

- exact features it can inspect and the approved analysis scope;
- semantic and contextual scope it can justify;
- the material it can identify as supporting, contradicting, complicating, or contextualizing;
- how it handles material it cannot semantically evaluate;
- how it handles potentially dependent records and unknown experience independence;
- temporal limitations and unknowns;
- relevant alternative explanations when they materially change the interpretation;
- conditions under which it returns **no Candidate**;
- the boundary and wording of any generated claim;
- mechanism identity and version;
- source, context, and generated-result invalidation conditions.

An inspectable rule is viable only when its input meaning and output claim are both supported. It must explain what it examined, which explicit feature connected the sources, why that feature supports the wording, what counterevidence was considered, and what it cannot establish. Rule identity and version must correspond to the mechanism actually used. This contract does not authorize a generic rule engine, a detection algorithm, or a schema.

Model Validation must inspect the approved analysis scope for material support, contradiction, complication, context, and material that cannot be evaluated. It cannot retrieve only confirming records. Mixed evidence may justify a genuinely narrower or qualified claim, but arbitrary exceptions cannot be added endlessly to evade counterevidence. “No identified counterevidence” must not become “no counterevidence exists.”

Generated language must be bounded by the mechanism's actual findings. A template may express an inspectable rule's result, but fluent or tentative wording cannot create missing evidence. The system must not convert a user Thought into external fact, record frequency into confidence, or structural co-occurrence into psychological or causal meaning. The user remains the final interpreter.

## Transient provenance and stronger invalidation

Even without persistence, a future system-generated proposal needs runtime provenance sufficient to answer “Why did NOUS propose this?” faithfully:

- proposer = system and the actual mechanism identity/version;
- exact analysis scope, source logical IDs, reviewed representation state tokens, and relied-upon Situation context;
- generated claim and bounded claim scope;
- actual support, contradiction, complication, and context basis;
- material counterevidence, dependence and temporal unknowns, and relevant alternatives;
- limitations and conditions that would weaken or change the proposal.

These are conceptual explanation requirements, not authorization to store any new fields. Do not expose raw chain-of-thought or invent reasons afterward. Task 006 `UserAuthored` EvidenceLink cannot serve as system-derived provenance.

Task 013 user-authored Candidate wording may remain a draft when evidence becomes stale. A future **system-generated** claim depends on its sources: correction, relevant change, or deletion can invalidate the generated wording, scope, explanation, evidence roles, counterevidence interpretation, and current validity status. The affected result must be withdrawn pending revalidation, not left current while only evidence cards are marked stale. Old inaccurate correction values and deleted content cannot remain active evidence or be retained in generated explanation merely to preserve the proposal. A user edit to system wording or scope changes the claim under review; the original mechanism's justification cannot silently validate that edited claim.

The existing Task 013 transient Candidate Review is the preferred future review surface for a system proposal. Its current-source display, representation-token and linked-Situation stale checks, and in-memory lifecycle are reusable boundaries. It does not currently supply system-proposer provenance, a mechanism explanation, system counterevidence assessment, generated-claim invalidation, or user-edit revalidation. No integration is implemented by Task 014.

Transient dismissal means the user does not continue that proposal in the current interaction. It is not counterevidence, a durable rejection, proof of falsity, or future suppression memory. Without separately approved dismissal persistence, the same proposal may recur in another explicitly requested analysis session. Do not silently store a rejection to avoid that limitation.

## Selected prerequisite and rejected shortcuts

The Owner selected a future **strictly transient user-assisted semantic comparison** prerequisite for separate Task 015 scoping. Its purpose is to evaluate whether user-provided comparisons can distinguish meaningfully comparable records, possibly shared experiences, different experiences, counterexamples, material differences, and unknowns. Task 015 Stage 1 must decide the exact semantics and bounds before any implementation. No persistent tags, categories, experience IDs, comparison relationships, or derived provenance are pre-approved. If persistence proves necessary, Owner review is required.

Neither a local LLM, embedding model, classifier, nor external model is assumed. Any future local semantic model requires separate architecture for capability and validation, false positives, provenance, version changes, runtime and hardware needs, privacy, explanation, source invalidation, and reproducibility. Running locally alone does not make its interpretation trustworthy.

The following shortcuts remain invalid:

| Shortcut | Missing justification |
| --- | --- |
| Keyword frequency or record count → psychological Pattern, recurrence, or confidence | Meaning, experience dependence, and qualitative evidence assessment |
| Same words → same meaning; different words → different meaning | Actual semantic comparison |
| Different Situation IDs → independent experiences; same Situation ID → complete dependence model | Experience identity and dependence |
| Structural relation → psychological interpretation | Approved semantic grounding |
| Co-occurrence → causation | Independent causal justification |
| No identified counterevidence → proof | Coverage and unknown handling |
| Tentative or strong wording → sufficient or strong evidence | Validated basis |
| User acceptance → stronger evidence; dismissal → counterevidence | Separation of user response from evidence assessment |

No persistent label shortcut, autonomous database-wide analysis, keyword psychological detector, model integration, or standalone pure-structural Candidate task is authorized.

## Reasoning Architecture placement

Grounding obtains approved current sources and explicit semantic grounding input. Psychological Modeling generates a possible Candidate only when a justified mechanism exists. Model Validation examines support, counterevidence, dependence, context, time, and material alternatives **before presentation**. Reasoning uses only an adequately validated interpretation relevant to the user's goal. Reasoning Validation blocks identity, causation, scope, and certainty jumps. Response Alignment checks the explicitly requested analysis. Expression presents calibrated wording, provenance, and limits. User review does not replace pre-presentation Model Validation.

## Documentation checkpoint boundary

This checkpoint synchronizes the governing reasoning, Self Model, roadmap, and backlog wording with this Owner decision. It does not alter completed Task 012 history or Task 013 behavior. It creates no implementation, SQL, migration, dependency, new persistent entity, system Candidate mechanism, or Task 015 work. Schema remains v5 and migrations remain 0001–0005. Foundation Audit v0.2 and the Reasoning Architecture Amendment are not reopened.

Documentation validation and final diff hygiene are recorded in the Owner review report. This checkpoint changes no implementation, tests, schema, migrations, dependencies, or configuration. Status: **ARCHITECTURE APPROVED — OWNER DOCUMENTATION REVIEW PASSED**.
