# NOUS Reasoning Model

## 1. Status, purpose, and boundary

**Amendment status: APPROVED / COMPLETE.** Owner Review: **PASS**. Psychological Model Validation, Reasoning Integrity, Response Alignment, the Self Model Relevance Gate, Faithful Explainability, and Bilingual Semantic Consistency are approved. Findings: BLOCKER none; REQUIRED none; REVIEW none.

This amendment is approved as governing architecture. It does not itself authorize implementation; Phase C and Phase D implementation still require separately approved tasks.

Starting checkpoint: `6854099874361dd6cbd075ed32a1ad20b992c083`. Foundation Audit v0.2 is CLOSED; Task 007 is COMPLETE; Task 008 and Phase C have not started. The approved Foundation remains the trusted baseline. This amendment neither reopens the audit nor changes completed task history.

This is the primary source for the reasoning semantics below. [SELF_MODEL.md](SELF_MODEL.md) defines the modeled concepts; [RESPONSE_POLICY.md](RESPONSE_POLICY.md) and [INTERACTION_MODEL.md](INTERACTION_MODEL.md) govern help and delivery; [SAFETY.md](SAFETY.md) retains Care/Safety authority; [VOICE_AND_I18N.md](VOICE_AND_I18N.md) governs expression. [TASK_SYSTEM.md](TASK_SYSTEM.md) still requires separately approved implementation tasks.

A fluent, personalized answer is not necessarily a justified answer. Important claims must have appropriate evidence, valid and revisable modeling, defensible reasoning, and a direct role in answering the user's request.

## 2. Conceptual ordering

```text
Evidence / Grounding
        ↓
Psychological Modeling
        ↓
Model Validation
        ↓
Reasoning
        ↓
Reasoning Validation
        ↓
Response Alignment
        ↓
Expression
```

Concise form: **Grounding → Modeling → Reasoning → Alignment → Expression**. The concise form retains both validation responsibilities.

This is a logical ordering of responsibilities, not a mandatory set of runtime services or a database design. Understanding the current request frames evidence selection from the beginning; final alignment checks that the response actually meets it. A factual or practical request need not pass through psychological interpretation when that interpretation is irrelevant.

Model Validation asks whether an interpretation is justified by its sources, context, time, and contrary evidence. Reasoning Validation asks whether the steps taken from those premises justify this conclusion. Response Alignment asks whether the justified content answers this user's question. Expression may improve clarity and warmth but must not repair a weak premise through rhetoric or hide an invalid reasoning step.

When validation fails, reconsider the affected premise or step, narrow or withhold the unsupported claim, or seek the minimum material clarification. Do not invent a stronger justification to preserve the intended answer. The existing Care Layer is cross-cutting: immediate safety support must not wait for model completeness or a long analytical sequence.

## 3. Epistemic discipline

The type, provenance, context, and time of a premise constrain what may follow from it. Storing a record, confirming its wording, or linking it as evidence does not make its content objective truth. Preserve these distinctions when selecting premises and explaining conclusions:

| Information | Meaning and permitted use | Boundary |
| --- | --- | --- |
| Situation | Context around an experience | Context alone does not establish a motive or trait. |
| User-reported Observation | What the user reports noticing, seeing, hearing, or experiencing | Observation != independently verified objective fact. |
| User Thought | Momentary interpretation, judgment, prediction, or internal statement | Thought != fact; repetition does not automatically make it a Belief. |
| User Emotion | A reported felt state in context | Emotion != external-world fact; intensity does not establish another person's intention. |
| Belief | A relatively stable proposition the user endorses, at a particular revision | Belief != objective truth; advice need not reinforce it. |
| Value | Something the user considers important, at a particular revision | A priority is not a truth claim; repeated behavior does not prove a Value. |
| Memory | Retrospectively selected experience and any user-authored meaning | Recollection is not verified chronology; NOUS must not invent the user's meaning. |
| Decision | A choice the user reports actually making | Considering an option is not a Decision. |
| Outcome | What the user reports later happened following a Decision | Sequence is not causal proof, and Outcome quality does not establish Decision quality. |
| Candidate/Hypothesis | A traceable, unconfirmed system interpretation | Candidate/Hypothesis != confirmed user knowledge, even when well-supported. |
| Pattern | A recurring structure under relevant conditions | Pattern != personality essence, diagnosis, or immutable identity. |
| Current temporary interpretation | A working understanding used for the present interaction | Current temporary inference != durable Self Model knowledge. |
| Concrete historical record | Attributed information about a past experience or representation | Historical record != permanent present-state truth; preserved error history is not proof the error was once true. |
| System interpretation | A claim NOUS derived from identified premises | Keep it distinct from user reports and confirmed knowledge, with its actual basis and limitations. |
| Relevant evidence / EvidenceLink | Information or an explicit assertion bearing on an exact claim or revision | Evidence relevance != proof; relationship type and source provenance remain visible. |

User-reported subjective conviction, Belief endorsement, Value importance, emotion intensity, and NOUS's justification for a conclusion are distinct meanings. Existing numeric user-entered fields retain their approved semantics; they must not become system truth probabilities. Task 007 captures Thoughts with `ThoughtConfidence=None`. This amendment introduces no numeric confidence score or new numeric field.

## 4. Psychological Model Validation

Important Pattern, Candidate, and Hypothesis interpretations must eventually be representable and inspectable along these conceptual dimensions:

- the claim;
- relevant context and conditions;
- the time range it describes, including material uncertainty about that range;
- supporting evidence;
- contradicting and complicating evidence, with contextualizing evidence where relevant;
- plausible alternative interpretations;
- current status, including whether the interpretation has weakened, been corrected, rejected, or replaced;
- why NOUS currently accepts it as a working interpretation;
- what kind of future evidence could weaken or change it.

These are semantic requirements for future Phase C design, not mandatory database columns, a final schema, or a fixed lifecycle enum. Unknown or unavailable information must remain unknown rather than be manufactured to fill a structure.

Validation must keep an interpretation proportionate to the relevant evidence and open to challenge. A long-lived candidate must remain distinct from canonical user knowledge, inspectable, explainable, and correctable. Stability does not make it an immutable hidden psychological profile. Neither model validation nor user disagreement gives NOUS authority over the user's identity.

## 5. Reasoning Integrity

An important conclusion should support a trace of the following form:

```text
Premise / evidence
→ reasoning step
→ intermediate claim
→ reasoning step
→ conclusion
```

Each step must preserve attribution, uncertainty, context, and temporal scope. An intermediate interpretation does not become a fact merely because a later step uses it. An appealing story, a known framework, or a familiar Pattern cannot substitute for evidence supporting the connection.

The following inference jumps are invalid as stated. Additional relevant evidence may support a narrower, bounded conclusion; it does not authorize reductive identity labels. Repetition or fluent wording alone cannot supply the missing justification:

| ID | Invalid jump | Example and boundary |
| --- | --- | --- |
| A | Observation → Trait | One unfinished task → “you are lazy.” A concrete behavior does not establish a broad identity label. |
| B | Emotion → External Fact | Feeling ignored → “the other person deliberately ignored you.” Felt experience does not prove external motive. |
| C | Thought → Fact | “I thought they did not trust me” → “they did not trust me.” Preserve the user's interpretation as an interpretation. |
| D | Repetition → Causation | A repeatedly co-occurs with B → A caused B. Co-occurrence alone does not establish cause. |
| E | Repetition → Identity | Behavior repeated several times → “this is who you are.” Recurrence may support a contextual candidate, not an essential identity. |
| F | Historical Pattern → Present Certainty | A Pattern existed previously → it necessarily exists now. Present claims need current temporal justification. |
| G | Context Transfer | A Pattern in academic group work → the same Pattern necessarily holds in romantic relationships, family, friendships, or unrelated domains. Transfer needs relevant evidence in those contexts. |
| H | Lack of Evidence → Negative Fact | No recent record of social activity → the user is lonely. Missing records do not establish missing activity or a psychological state. |
| I | Evidence Count → Truth | Many duplicated or similar records → the claim is objectively true. Record count is neither independent corroboration nor a confidence algorithm. |
| J | Strong Language → Strong Evidence | Emotionally intense wording → a more factually certain interpretation. Intensity may inform care or style, not factual certainty. |

These constraints do not authorize diagnosis or third-party psychological profiling even if more records are available. A `PersonReference` remains context for the user's own experience; the other person's inner state remains uncertain. No rule engine is implemented by this document.

## 6. Counterevidence and alternative explanations

Important interpretations must consider material supporting, contradicting, complicating, and contextualizing evidence. NOUS must not repeatedly select only evidence that protects its current model. New evidence can strengthen, weaken, complicate, narrow, replace, or reject an interpretation. This is not evidence-count scoring, majority voting, or a requirement that every case contain all four relations.

Copies of the same event, equivalent EvidenceLink assertions, or retellings based on one source must not be counted as independent corroboration. Similarity alone establishes neither independence nor duplication; uncertainty about the basis must remain visible. This document specifies no deduplication algorithm.

Where several explanations remain plausible, consider the alternatives that could materially change the conclusion. Repeatedly taking over group work could reflect concern about quality, deadline pressure, role ambiguity, distrust of teammates, habit, or external responsibility requirements. A current Pattern cannot select one motive by itself.

Do not generate alternatives endlessly or give unsupported alternatives equal standing with strong evidence. Once the relevant evidence adequately supports a bounded conclusion, state it clearly and keep it revisable. Considering alternatives must not excuse reported harm or undermine the Grounded Strength boundary in [INTERACTION_MODEL.md](INTERACTION_MODEL.md).

## 7. Context scope and temporal validity

Psychological conclusions describe recurring structures under conditions. For example:

> In high-stakes academic collaboration, when responsibility is unclear, you have repeatedly tended to take over work.

That does not by itself support “you are controlling” or “you behave this way in intimate relationships.” Evidence from one context cannot silently generalize to another. State-dependent Thoughts must not automatically replace long-term Beliefs or Values; older commitments must not prevent recognizing genuine later change.

Past patterns describe a period. Reasoning about the present must consider recency, repeated later evidence, changed behavior, user correction, genuine user change, context differences, and contrary recent evidence. Older evidence may remain historically true while becoming less representative now. Both improvement and deterioration are possible; neither early negative anchoring nor a positive-only interpretation is justified.

No fixed numerical decay formula or weeks/months threshold is specified. Current `created_at_ms` is storage creation time, not life-event time or psychological-effective time. Revision numbers order commitment representations; they do not supply a complete lived chronology. Missing semantic dates constrain temporal claims and remain unknown until a separately approved temporal feature provides appropriate information.

`UserUpdate` records genuine change. `UserCorrection` records an inaccurate representation. Preserve provenance without continuing to use a corrected mistake as a true past user state. Future derived interpretations must reconsider claims dependent on a corrected or rejected premise; an old model must not defeat correction merely by citing itself. This adds no correction API and does not reinterpret existing records.

## 8. Conclusion strength

The strength of language must not exceed its justification. A possible qualitative progression is:

```text
possible → recurring → well-supported → historically stable
```

These descriptions are not numeric scores, mandatory states, or an automatic promotion ladder. “Historically stable” still has a context and time range; it is not certainty about the present or permanence. Well-supported conclusions remain revisable.

NOUS may say “I don't have enough evidence yet.” It must also be able to state a clear, grounded conclusion when relevant evidence supports one, without endless “maybe/perhaps” or recurring approval questions. Calibration should address material uncertainty and generally become less frequent as relevant evidence matures. Conversational directness never supplies persistence intent or turns candidates into confirmed user knowledge.

## 9. Response Contract

For each meaningful request, the future response process should identify, as appropriate:

| Dimension | Question it answers |
| --- | --- |
| Primary Ask | What explicit question or action does the user want answered? |
| Explicit Sub-questions | Which additional questions did the user actually ask? |
| User Goal | What is the user trying to understand or accomplish? |
| Constraints | What limits, preferences, stakes, or requested format matter? |
| Relevant Context | Which current details or historical records materially affect the answer? |
| Material Ambiguity | What unresolved ambiguity could change the answer or next action? |
| Information that is NOT needed | What related material can be left out? |

This Response Contract is a transient conceptual responsibility. It is not a persistent Self Model entity, a questionnaire, a command payload, or a database design. Do not persist it as psychological knowledge merely because it exists during response generation. Use information already supplied and ask only for missing information that materially matters.

## 10. Response priority and question dependencies

| Priority | Content | Purpose |
| --- | --- | --- |
| P0 | Direct Answer | Address the user's actual explicit question or requested action. |
| P1 | Required Reasoning | Supply what is needed to justify or understand that answer. |
| P2 | Relevant Context | Add context that materially improves understanding or action. |
| P3 | Optional Expansion | Include useful but nonessential related material only when appropriate. |

P2/P3 must not crowd out or delay P0. These are response-content priorities, not an optional engineering backlog or a rigid visible answer template. Much relevant information does not compensate for failing to answer the Primary Ask. A direct answer may state a material limitation when the evidence is insufficient.

Multi-part questions must respect their dependencies. For “Why do I keep doing this?”, “Has this pattern changed?”, and “What should I do?”, the logical dependency may be:

```text
historical evidence
→ pattern interpretation
→ current temporal comparison
→ recommendation
```

Do not present downstream advice as established while its upstream assumptions are unresolved. Distinguish what is supported from conditional advice or missing evidence. This semantic dependency requirement does not prescribe a graph engine or force an analytical preamble before every answer.

When requested advice has sufficient relevant grounding, give useful direction under [PRINCIPLES.md](PRINCIPLES.md) §13.1. Preserve final user authority, scale strength to stakes and reversibility, and challenge harmful historical habits when justified rather than recommend them merely because they recur.

## 11. Self Model relevance and emotion

**Memory availability != Memory relevance.** Self Model information enters current reasoning only when it materially changes understanding of the question, relevant reasoning, advice, response strategy, or Care/Safety handling. Every substantial historical detail included in the answer must have a defensible role. NOUS must not display personal history simply to demonstrate that it remembers.

Ordinary emotion generally modifies style and prioritization; it does not automatically replace the user's task. For “I'm so frustrated. Why does this code keep throwing NullPointerException?”, debugging remains the primary need. Warmth, pacing, or concise wording may help; automatic emotional counseling would miss the request.

Existing Care/Safety policy remains authoritative when immediate self-harm or suicide safety is implicated. Safety and real human support may override normal task priorities. Grounded Strength remains an interaction modifier under its existing conditions; neither emotion nor that modifier creates stronger evidence, hidden motives, or permission to escalate hostility.

## 12. Response Alignment check

Before an important response, conceptually check:

1. Did it directly answer the Primary Ask?
2. Were all explicit Sub-questions addressed, including material limits on what can be answered?
3. Is any major section unnecessarily answering a different question?
4. Did every use of long-term history pass the relevance gate?
5. Do important conclusions follow from the cited premises/evidence?
6. Did any reasoning step turn possibility into certainty without justification?
7. Did NOUS ask again for information the user already supplied?
8. If advice was requested and evidence is sufficient, did it provide useful direction rather than only analysis?
9. If a factual answer was requested, did it avoid unnecessary psychological interpretation?
10. Is the answer complete enough to stop?

Existing safety overrides still apply. The checklist is not a recurring user approval form. Logical completeness does not mean maximum length: once the user's information need is satisfied, STOP.

## 13. Explainability and faithful reasoning traces

An important conclusion should be able to answer “Why do you think that?” using the actual relevant basis:

```text
Claim ← reasoning ← evidence / model / current context
```

User-facing explanations should identify relevant evidence, major reasoning factors, material uncertainty, counterevidence where material, and why the conclusion follows. If a model interpretation is used as a premise, its source basis, scope, and status must remain available for inspection rather than function as unexplained authority. Preserve applicable rule/framework provenance without treating a theory as proof.

An explanation must be faithful to the factors that produced the conclusion. Do not manufacture a post-hoc account from convenient records that did not actually ground it. Inspectable structured reasons do not require disclosure or storage of internal raw chain-of-thought, hidden tokens, or every internal deliberation step.

## 14. Bilingual Semantic Consistency

**Language may change expression. Language must not change reasoning semantics.** For equivalent input and context, English and Simplified Chinese must preserve:

- factual conclusion and psychological interpretation;
- evidence strength and uncertainty level;
- advice direction and degree of authority;
- Pattern context and temporal scope;
- Care/Safety priority;
- user agency.

“Possible” in English must not become “基本确定” in Chinese. A cautious Chinese recommendation must not become an English command. A Pattern scoped to academic teamwork must stay equally scoped in both languages.

Tone, rhythm, idiom, sentence structure, warmth, and culturally natural phrasing may differ. Those differences cannot upgrade certainty, invent a motive, change the recommendation, widen a Pattern, or alter safety priority. Equivalent meaning, rather than literal translation or matching localization keys alone, is an explicit future validation requirement. Existing human review of safety copy in both languages remains required.

## 15. Fit with the approved Foundation and future validation

| Existing boundary | Role in future reasoning; preserved limit |
| --- | --- |
| Task 004 persistence and revision history | Supplies validated user records and ordered revisions. Storage validity does not establish psychological truth; the repository does not become a reasoning engine. |
| Task 005 Memory / Decision / Outcome | Supplies user-authored retrospective experience, actual choices, and later reported outcomes, without invented meaning, causal proof, or automatic promotion. |
| Task 006 EvidenceLink | Remains one immutable, deliberately UserAuthored assertion from one allowed source to one exact BeliefRevision or ValueRevision, with same-subject ownership. |
| Four evidence relations | Supports, Contradicts, Complicates, and Contextualizes inform interpretation according to source and target meaning. Value relations concern orientation or priority, not true/false values. Situation and Emotion remain contextual-only sources in the current closed matrix. |
| Exact revision and duplicate policy | Old links remain with their original revisions; later revisions do not inherit them automatically. Repeated equivalent rows do not add semantic weight. |
| Future Pattern / Candidate / Hypothesis | Adds revisable system interpretations with validated scope and provenance, distinct from confirmed knowledge. EvidenceLink does not already target these concepts, and its UserAuthored provenance must not be reused to disguise inferred links. Broader relationships and derived provenance need separate future designs. |
| Temporal revision and correction | Preserves genuine change versus inaccurate representation. Richer semantic event time and derived-interpretation lifecycle remain future work. |
| Task 007 structured capture | Explicit Save persists only the approved capture records with lazy SelfSubject bootstrap in one atomic transaction. Temporary raw input, review drafts, and interpretations do not become stored history. No capture behavior changes here. |
| Advice, Care/Safety, and Grounded Strength | Use validated relevant reasoning while preserving user agency, no diagnosis, and existing safety precedence. Modifying expression does not modify truth conditions. |
| Local ownership and privacy | Core reasoning requires no external AI/LLM API or token billing. Future candidates and explanations remain sensitive user data, subject to inspectability, correction, and data sovereignty; no third-party psychological models or telemetry are authorized. |

Phase C planning must make model validation, counterevidence, alternatives, context/time scope, and faithful reasoning provenance explicit. Phase D planning must make request/goal alignment, relevant Self Model selection, requested advice from validated reasoning, explainability, and bilingual semantic consistency explicit. Neither phase starts through this document.

Future bounded tasks should validate these semantics with focused cases: the ten invalid inference jumps; counterevidence and plausible alternatives changing a conclusion; corrections and later evidence weakening old interpretations; irrelevant history excluded; explicit multi-part asks and their dependencies addressed; requested grounded advice given; ordinary emotion preserving the task; Care/Safety precedence; and equivalent English/Chinese conclusions, uncertainty, scope, and agency. Traces should be checked against the actual basis used. This is validation direction, not tests, algorithms, schemas, new task numbers, or implementation authorization.

No Rust, TypeScript/React, tests, migrations, schema, dependencies, scoring, NLP, reasoning engine, Pattern/Candidate implementation, external/cloud AI, or Task 008 work is included. The amendment is **APPROVED / COMPLETE**.
