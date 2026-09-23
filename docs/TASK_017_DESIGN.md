# Task 017 — User-Grounded Thought Recurrence Candidate v1

## Status and authority

Task 017 Stage 1 architecture and Stage 2 technical design were Owner approved. Stage 3 implementation has passed Owner Code Review with BLOCKER 0, REQUIRED 0, and REVIEW 0. This document is the Stage 4 documentation checkpoint for Owner review. **Task 017 is not COMPLETE** until the documentation review, approved Git checkpoint, and merge. The starting `main` and current task-branch HEAD are `e89bb42b8ea9f2f81cbefc7e140f5075c8e8a54e`; the branch is `task/017-user-grounded-candidate-v1`. Task 016 is COMPLETE at checkpoint `aedcc643d723742fada7796c91849880e90a44e0`. Foundation Audit v0.2 remains CLOSED. Schema remains v5; migrations remain 0001–0005. Task 018 has not started.

Task 017 implements exactly one transient **User-Grounded Thought Recurrence Candidate v1** mechanism. It is NOUS's first implemented Level 3 system-proposed Candidate mechanism, subject to final Task 017 checkpoint. It does not provide autonomous semantic understanding, Pattern detection, diagnosis, trait or causal inference, third-party psychological inference, semantic chronology, or AI/ML inference.

## Human grounding and system contribution

The user reviews a Task 015 Thought comparison, including its exact Comparison Frame, complete selected Thought scope, Anchor, and Anchor-relative Experience Relations. The user then writes exactly one analysis-specific Thought content **X** in their own words and classifies **every** selected Thought relative to X as `present`, `absent`, or `cannotTell`. A draft may contain unanswered classifications; unanswered is not `cannotTell`. X and the Frame remain exact, untranslated user text. X is neither a system-discovered abstraction nor a tag, Belief, Value, or Pattern.

NOUS validates the complete selected scope against current sources, checks the typed inputs, applies the recurrence proof, considers absent and unknown material, returns zero or one result, and exposes the basis, limits, and transient provenance. The system's contribution is the bounded cross-experience inference and the decision to withhold it when the proof fails. User-provided semantics and markings are not independent evidence or user acceptance of a Candidate.

Task 015 supplies the reviewed workflow, source type, complete selected set in selection order, Frame, Anchor, Experience Relations, and current Thought/context representations. Initial entry requires a reviewed Thought comparison. After handoff, Candidate validity follows those actual dependencies, not Task 015's aggregate `reviewStatus`. Task 015 `similar`, `materiallyDifferent`, and `notComparable` are **not** Candidate-specific evidence: none maps to X `present`, `absent`, or exclusion. Task 015 remains a separate user-assisted comparison workflow; Task 013 remains a separate user-authored Candidate Review.

The only supported source type is **Thought**. A Thought is cognitive content, not external fact or a stable Belief. Observation is unsupported in v1 and is never silently coerced. A linked Situation supplies reviewed context and freshness identity; it is not another occurrence or an experience identifier. Thought subjective conviction does not measure Candidate evidence strength.

## Exact recurrence rule and claim ceiling

An analysis is ready only with a valid Thought handoff, at least two distinct selected Thought IDs, a nonblank Frame and X, an Anchor in the scope, one explicit classification for every selected Thought and no extra classification, structurally valid Anchor-relative Experience Relations, and no known stale dependency. Readiness does not imply that the recurrence proof succeeds. The user must separately press Analyze; finishing classifications, reviewing Task 015, saving, entering the page, starting the app, and focus refresh do not generate a Candidate.

After one complete `loadStructuredHistory()` refresh and validation of the entire frozen scope against that single snapshot, the deterministic decision is:

```text
if Anchor classification != present: zero Candidate
if fewer than two Thoughts are present: zero Candidate
proof edges = present non-anchor Thoughts whose relation to Anchor
              is differentExperiences
if proof edges are empty: zero Candidate
otherwise: exactly one bounded system Candidate
```

The strongest experience conclusion is always **at least two experiences the user identified as different**. Even if A↔B and A↔C are both marked `differentExperiences`, B↔C remains unknown; v1 never derives three distinct experiences, transitivity, statistical independence, or a confidence score. Additional `present` records whose relation is `sameOrPossiblySame` or `cannotTell` remain visible but do not raise the proven lower bound.

The Candidate claim is limited to this: **based on the user's record-by-record markings, within the Thought records reviewed here, the user-defined content appears in records associated with at least two experiences the user identified as different.** Its Claim Scope is the reviewed Thought set, not the user's life generally. Its Analysis Scope is the **complete** selected Thought set, including `absent` and `cannotTell` records. NOUS does not validate X itself as true, infer a general tendency, stable Pattern, Belief, Value, identity, trait, diagnosis, cause, motive, third-party psychology, external fact, future behavior, probability, confidence, or temporal trend. Candidate is not Pattern.

`absent` limits breadth but does not defeat this minimal existential proof; it is not automatically Task 013 “contradicting,” nor proof that X was absent from an entire experience. `cannotTell` is explicit uncertainty, neither support nor contradiction; it does not block an otherwise valid proof. There is no majority rule, ratio, numeric penalty, or artificial unanimity requirement. The result shows all selected sources and markings, proof edges, non-proof `present` relations, and any absent or uncertain records.

A **semantic zero** is a valid successful analysis when the Anchor is not `present`, fewer than two Thoughts are `present`, or no `present` non-anchor Thought has a `differentExperiences` relation to the Anchor. It means only that these reviewed markings do not establish this Candidate. It does not mean X is false, never occurs, or that no real-life Pattern exists. Incomplete grounding remains a draft; invalid structure is not a semantic zero; changed or deleted sources require re-review; storage, data, or load failure is a separate error state.

## Attribution, provenance, and expression

The bilingual UI separates **User-defined Thought content / 用户定义的想法内容**, the user's Comparison Frame, and **Candidate proposed by NOUS / NOUS 提出的候选结论**. The fixed system conclusion refers to “this content / 这一内容” instead of inserting X into an unqualified factual assertion. X and the Frame are rendered as ordinary escaped React text, without translation, HTML interpretation, Markdown execution, or automatic linkification. A third-party, causal, diagnostic, or external-factual phrase inside X remains explicitly user-authored; NOUS does not endorse its truth. English and Simplified Chinese preserve the same attribution, reviewed scope, uncertainty, and “at least two” quantifier.

The transient structured payload records `proposer=system`; mechanism `user-grounded-thought-recurrence` version 1 and expression version 1; attempt, request, and dependency revision; the exact user-authored X and Frame; `sourceType=thought`; Anchor; complete ordered Analysis Scope with every classification; current Thought logical IDs, content and state tokens; linked Situation identity, description and state token where present; all relevant `present` Experience Relations; actual Anchor proof edges; fixed experience lower bound 2; absent, uncertain, and non-proof present records; Claim Scope; and limitations. Explanation reflects these actual factors, not invented post-hoc reasons or raw chain-of-thought. Task 006 `UserAuthored` EvidenceLink is not reused. None of this is persisted.

## State, freshness, and invalidation

The transient workflow distinguishes `closed`, `draft`, `checking`, `zeroResult`, `proposed`, `dismissed`, `stale`, and `loadError`. Changing the literal X text clears **all** X classifications, invalidates any result, and advances the dependency revision; changing it back does not restore old markings. Changing one classification preserves the others, invalidates the result, and requires another explicit analysis. Dismissal hides only the current session proposal, preserves current grounding, and permits a later explicit rerun; it is not rejection, falsification, counterevidence, or a durable preference.

Every explicit Analyze performs one `loadStructuredHistory()` read and checks every frozen selected Thought against that snapshot: logical ID and type, state token and current content, plus linked Situation identity, token, and description or the same standalone status. A changed or deleted selected source invalidates the **whole** proposal or zero result. The scope is not silently shrunk for Candidate generation. Stale Candidate state retains only reason and affected IDs, not old source/context text or generated prose; current Task 015 records must be reviewed again. Corrected or deleted old representations cannot remain active Candidate evidence. The implementation also revokes Task 015 answers when a defensive content/context comparison detects a change without a changed token.

Selection or source-type change, Frame change, and Anchor change close the Candidate attempt. X or any X classification edit, or a `present` non-anchor Experience Relation edit, invalidates the generated result and returns to draft. A mechanism semantic-version change cannot reuse an earlier transient result; leaving/reloading the workflow discards it. Comparability-only and Meaning Relation-only edits do not by themselves invalidate the Candidate. A relation change for an `absent` or `cannotTell` Thought updates the handoff without invalidating an existing result because that relation is neither used in its proof nor rendered as a Candidate reason; if that Thought later becomes `present`, the classification edit invalidates the result. Language switching only rerenders. Unselected history changes are not Candidate dependencies.

Request identity, Candidate attempt identity, dependency revision, and the component's active request gate prevent a late read from publishing after a relevant edit, newer request, dismissal, clear, leave, or unmount. Focus never generates a new Candidate. When an existing proposal or semantic zero is checked on focus, it is temporarily withheld as current; an identical current-source snapshot restores the **same** result without rerunning recurrence reasoning. A changed source withdraws it as stale, and a failed read withdraws current-valid status as a load error. No automatic regeneration occurs.

Review permits inspecting the proposal, complete grounding, explanation and limitations; leaving it unresolved; dismissing for this session; returning to grounding; editing grounding; and explicitly rerunning. There is no Accept, Confirm, direct Candidate edit, Pattern promotion, durable dismissal or rejection, browser storage, export, or backup semantics. Closing or reloading the transient workflow discards its state.

## Implementation surface and validation

Task 017 is frontend-only and reads through the existing `load_structured_history` command. There is no new Tauri command, Rust or backend change, history DTO change, backend write, SQL, migration, schema change, dependency, manifest, lockfile, capability, or configuration change.

Added: `src/app/thought_recurrence_candidate.ts`, `src/components/SystemCandidateReview.tsx`, `tests/thought-recurrence-candidate.test.ts`.

Modified: `src/components/SemanticComparison.tsx`, `src/i18n/en/common.json`, `src/i18n/zh-CN/common.json`, `src/styles/global.css`, `tests/semantic-comparison.test.ts`, `tests/i18n.test.ts`.

Task 013's `src/app/candidate_review.ts` and `src/components/CandidateReview.tsx`, plus `src/app/App.tsx` and `src/app/semantic_comparison.ts`, remain unchanged. Task 018 has not started.

Owner-reviewed Stage 3 validation: **68 tests PASS; typecheck PASS; lint PASS; build PASS; `git diff --check` PASS.** The first Windows Vite build attempt stopped at `spawn EPERM` under an execution restriction; the same build passed when subprocess creation was permitted. That restriction was not an application defect. Documentation-only changes do not require repeating the full implementation suite.

## Product evaluation and future boundary

A dedicated **Candidate Product Test is required after Task 017 merge and before expanding Candidate semantics**. Automated tests establish contract correctness, not product value. The controlled test must examine correctness, NOUS's cognitive contribution, interaction cost, false-authority risk, visibility of mixed material, and quality of zero output. If users effectively author the entire conclusion and NOUS adds little, or the required manual grounding is burdensome, treat that as a real product-failure signal rather than automatically adding more fields.

Current v1 uses no LLM, embeddings, classifier, local model, remote model, or external AI API. A future local semantic model is only an optional, separately reviewed research path: it might propose grounding, the user would confirm or correct it, and the same inspectable Candidate reasoning would remain responsible for the bounded conclusion. A model proposal would not be truth. No model API or abstraction is introduced by Task 017; normal NOUS use remains independent of external AI.

Foundation remains **CLOSED**. Evidence assessment, user acceptance, and current applicability remain separate; Candidate is not Pattern; Thought is not fact; Observation is not objective external fact; repeated Thought is not Belief; behavior is not Value; user grounding is not independent evidence; different experiences are not statistically independent. No count-based confidence, causation, diagnosis, third-party psychological inference, or semantic chronology is introduced. Care/Safety and the third-party boundary retain their authority.
