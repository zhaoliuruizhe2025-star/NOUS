# Task 016 — User-Grounded System Candidate Architecture

## Status and authority

Stage 1 architecture review: **OWNER APPROVED**. This document is the Task 016 documentation checkpoint for Owner review; Task 016 becomes **COMPLETE** only after documentation review, Owner approval, checkpoint, and merge into `main`. Starting main: `273eede0a9132106fe38d78801e37e489c6d0299`. Task 015 is complete at `f937936eb74b006d708775e56a0360f940476584`. Phase B is closed; schema remains v5 with migrations 0001–0005. Foundation Audit v0.2 remains closed.

**Current direct Task-015-to-system-Candidate generation: NOT AUTHORIZED. Candidate System feasibility: NOT REJECTED.** The preferred next architecture direction is to design one complete **User-Grounded Candidate System v1** mechanism. This is a future design direction, not authorization to implement it or to start Task 017. Autonomous psychological-semantic Candidate generation remains unauthorized.

Documentation-checkpoint findings: **BLOCKER: 0; REQUIRED: 0; REVIEW: 0**. The current *capability* is blocked by missing grounding and validation; that is not a blocker to recording this architecture decision.

## What Task 015 supports now

Task 015 provides a strictly transient, user-authored Comparison Frame; Thought **or** Observation selection; exactly one Anchor; and Anchor-relative Comparability, Meaning Relation, and Experience Relation judgments. Its current sources include current representation state tokens and linked Situation context. Its reviewed output faithfully shows those inputs. It does not synthesize semantic groups, assume transitivity, infer recurrence or causation, generate a Candidate, or persist the comparison.

Keep three levels separate:

1. **User grounding:** the selected records, Frame, Anchor, and user judgments.
2. **Faithful summary:** for example, “Under this Frame, you marked A and B similar and from different experiences.” This does not identify the particular shared meaning.
3. **System Candidate:** a possible, reviewable recurring structure that NOUS derives through a defined mechanism, validates against a specific claim before presentation, and keeps transient and unresolved.

Task 015 reaches Level 2. Level 2 must not be renamed a Candidate to claim system generation. A Comparison Frame defines an angle of comparison, not a proposition to adopt. NOUS may cite it verbatim as user input but may not transform it into an unsupported claim or broaden claim scope from it. Thought remains cognitive content; Observation remains user-reported noticed or experienced information.

## Two remaining semantic gaps

**Missing commonality.** “Similar in this respect” does not identify what is similar. “I will disappoint everyone” and “I am not good enough” may be similar under several interpretations; Task 015 does not select one. NOUS cannot invent self-criticism, fear of failure, low confidence, evaluation sensitivity, relationship insecurity, or another psychological concept from the comparison flag or raw text. Two Anchor-relative similarities do not establish that the compared records are similar to one another or share the same aspect.

**Candidate-specific validation.** Task 015's `materiallyDifferent`, `notComparable`, and `cannotTell` are judgments relative to the Comparison Frame. They are not automatically contradicting, complicating, contextualizing, or neutral with respect to a later claim X. The future mechanism must evaluate the *complete selected analysis scope* against X, including material absence, uncertainty, context, dependence, and relevant alternatives, before showing a Candidate. User review after presentation cannot substitute for that validation.

Task 015 improves user-grounded comparability and similarity, supplies a bounded selected scope, and adds user-reported experience dependence. It does not establish statistical independence, semantic event time, a concrete shared aspect, Candidate-relative counterevidence, or autonomous natural-language understanding. Record counts and user comparison judgments are not separate corroborating events or confidence scores.

## Preferred future v1: human semantics and system reasoning

Design one narrow **user-grounded recurring-content Candidate** class before considering a semantic model. A human may supply the Comparison Frame, pairwise judgments and experience relations, an explicitly scoped shared semantic aspect X, and grounding of X against relevant records. A future claim-specific assessment might distinguish X as present, absent, uncertain, or not applicable in a record. These are conceptual examples, **not approved fields, enum names, UI, or implementation rules**.

The system may then combine the explicit relations, check current source and context representations, inspect the entire selected scope, evaluate whether X is represented in more than one user-reported different experience, handle absent and uncertain material, determine whether a bounded claim is supportable, and return **zero or one** transient Candidate with its actual basis and limitations. It must return zero when a meaningful claim would require new semantic invention or cannot pass claim-specific validation. Exact input conditions, output wording, and non-output rules belong to a separately approved mechanism design.

A user-authored Shared Aspect may be part of that **complete** v1 mechanism. Its text remains user-authored, explicitly tied to the materials it describes, inspectable, transient, and distinct from a tag, Belief, Pattern, or system-discovered concept. Task 016 neither authorizes this field nor selects a standalone Shared Aspect feature to imply progress toward Candidate generation. Grounding X as present in some records still does not validate the resulting Candidate: absence, uncertainty, scope, dependence, source freshness, and material alternatives must also be considered.

This division is useful only if NOUS contributes actual reasoning. Wrapping X in tentative grammar, checking nonblank fields, or restating a complete user-authored conclusion is insufficient. A future mechanism should make a defensible bounded inference from several partial premises, explain why it proposed the claim or returned zero, and show where its wording came from. User-provided semantics do not make the resulting system interpretation user-confirmed. Evidence assessment, user acceptance, and current applicability remain separate.

## Scope, recurrence, and mixed material

The minimum Task 015 relation that could ground cross-experience comparison is one Anchor-relative pair marked `comparable`, `similar`, and `differentExperiences`. By itself it permits only a faithful statement that the **user** marked two records from reportedly different experiences as similar under the Frame. With future explicit grounding of X and claim-specific validation, a narrow v1 may be able to state that X appears in more than one user-reported different experience **within the reviewed scope**. This is not independent corroboration, a confidence level, a temporal trend, or a Pattern.

If A↔B and A↔C have such judgments, B↔C remains unknown; B and C may describe the same experience. Do not infer three independent experiences, a semantic group, or transitivity. A `sameOrPossiblySame` or `cannotTell` experience relation cannot silently count as a different experience.

Mixed results must remain visible. `materiallyDifferent` is not automatically Candidate counterevidence; `notComparable` is not contradiction; `cannotTell` is unresolved, not neutral support. Nor is artificial unanimity required. A future rule must decide how each *claim-specific* result affects X and the exact claim, and return zero if material ambiguity defeats a faithful bounded statement. Analysis scope is everything examined, including nonsupporting and unknown selected material; claim scope is the narrower set of conditions justified by that assessment. Missing counterevidence among selected material does not prove its absence elsewhere.

## Zero output, trigger, and proposal boundary

The first future generation mechanism returns **zero or one** Candidate. Zero is a valid successful outcome. Reasons may include missing X, incomplete claim-specific grounding, no required cross-experience relation, stale source or context, material unknowns, semantic invention, unjustified scope expansion, or a claim that would assert causation, identity, third-party psychology, unsupported time, or objective external truth. These are conceptual gates; exact v1 rules are not fixed here.

The user must explicitly select or authorize the analysis material **and** explicitly start Candidate analysis. A possible future handoff is: complete grounding → review it → choose “Look for a possible Candidate” → validate → receive zero or one proposal. Save, app start, page entry, the final comparison answer, focus refresh, and ordinary interaction are not triggers. A system proposal is a possible interpretation, cheap to reject, transient, unresolved, and neither durable knowledge nor a Pattern. Dismissal is session-local; it is not counterevidence, falsification, a future suppression preference, or persistence authorization.

The proposed claim must concern only the current user's reported content or experience. It cannot infer personality, identity, diagnosis, cause, a stable trait, external objective truth, another person's mind, future behavior, or recent/increasing/worsening/improving/longstanding history without separately approved semantic time evidence. A user-authored causal phrase may be displayed as attributed input; NOUS must not adopt it as its own causal conclusion. Positive and negative proposals have the same grounding standard.

## Provenance, editing, and invalidation

Any future v1 mechanism must identify its actual version, exact accepted inputs and analysis scope, sources and their current representation tokens, relied-upon Situation context, Frame, Anchor, each relevant user judgment, any explicit X and its per-record grounding, mixed and unknown material considered, generated claim and claim scope, limitations, what NOUS added, and why the result remained tentative. Claim phrases must be attributable to user input, source text, or a defined system rule/template; fluent prose cannot conceal a new abstraction. Explanations must reflect actual factors, not post-hoc reasons or raw chain-of-thought. None of this is persistent in the first slice; Task 006 `UserAuthored` EvidenceLink does not become derived provenance.

A change to any materially used source representation, linked Situation or linkage, source selection or type, Frame, Anchor, comparison judgment, shared semantic grounding, claim-specific classification, generated claim scope, or mechanism semantic version invalidates the **whole generated proposal**, including wording, evidence assessment, explanation, and current validity. Do not keep a psychological conclusion current by marking only evidence cards stale. Re-evaluate from current sources after an explicit user request; first v1 need not attempt incremental semantic repair. Deleted content and corrected inaccurate values cannot remain active evidence or copied explanation content.

The first system proposal should not allow direct editing of its validated wording. The user may inspect, dismiss for this session, leave it unresolved, or change grounding and rerun. If editing is added later, preserve the original system proposal separately from current user-edited wording; edited text cannot inherit the original validation. A “this seems right” action would not strengthen evidence, create a Pattern, or save the claim.

Task 013 remains a user-authored Candidate review surface. Future integration may reuse current-source display and transient lifecycle, but requires system proposer provenance, mechanism identity/version, pre-presentation Model Validation, whole-proposal invalidation, explanation, and any edit provenance. Task 015 comparison states must not be automatically converted into Task 013 evidence roles. No integration is authorized here.

## Candidate Product Test after future implementation

After a separately approved User-Grounded Candidate System v1 is implemented, conduct a dedicated **Candidate Product Test**, separate from unit and integration tests. Include realistic cases with multiple reportedly different experiences sharing grounded content; one relevant experience; several records from the same or possibly same experience; mixed present and absent material; `cannotTell`; corrected, deleted, or stale sources; user input that already supplies the whole conclusion; and partial grounding that NOUS integrates into a new bounded claim.

Evaluate at least three outcomes:

1. **Correctness:** Does the claim stay within its evidence, scope, epistemic type, and limitations?
2. **System cognitive contribution:** Did NOUS infer something useful that the user had not already fully written? Hide the final Candidate and ask whether the user can trivially reconstruct the complete conclusion from their required inputs.
3. **Interaction cost:** Is the amount of manual semantic grounding proportionate to the value of the proposal?

If v1 is correct, contributes meaningful cognition, and has acceptable cost, continue structured Candidate/Pattern development under separately approved tasks. If it is correct but largely paraphrases user input or demands too much labeling, treat that as evidence of the practical limit of manual grounding and separately evaluate local semantic capability. If even the bounded deterministic mechanism is unreliable, stop expanding it and reconsider the Connect/Reflect direction. The Product Test decides among these later paths; this checkpoint does not preselect its result.

## Optional later local semantic research

NOUS does not currently require a model. A later, separately approved **local semantic grounding model** could propose comparability, similarity, a shared aspect, aspect presence/absence/uncertainty, or Candidate-relative classifications. It would be a semantic proposer/evaluator, never a truth, psychological, or Pattern authority. User confirmation/correction and structured reasoning would remain outside it. A clean grounding interface could let human-provided inputs later be supplemented by local proposals without replacing the reasoning layer; this is architectural compatibility, not a guaranteed product or research result.

Such a model requires its own Foundation-level capability, validation, provenance, versioning, privacy, hardware/runtime, and source-invalidation review. Local execution alone does not establish trustworthiness. Normal NOUS use remains independent of external AI/LLM APIs, user API keys, cloud semantic processing, and token billing. None is authorized here.

## Foundation consistency and checkpoint boundary

This decision preserves Foundation Audit v0.2, the [Reasoning Model](REASONING_MODEL.md), [Self Model](SELF_MODEL.md), [Task 012](TASK_012_DESIGN.md), [Task 014](TASK_014_DESIGN.md), and actual Task 013/015 behavior. Evidence assessment, user acceptance, and current applicability stay distinct. Thought is not fact; Observation is not objective fact; repeated Thought is not Belief; behavior is not Value; user grounding is not independent evidence; acceptance is not evidence strength; different experiences are not statistical independence; Candidate is not Pattern; and a future model proposal would not be truth. Model Validation precedes presentation. Care/Safety and the third-party boundary retain priority.

Task 016 changes architecture documentation only. It creates no implementation, field, enum, algorithm, SQL, migration, dependency, model, persistence, Candidate generator, Task 013/015 behavior change, or Task 017 work. Documentation Owner review, checkpoint, and merge are still pending. Schema remains v5 and migrations remain 0001–0005.
