# Task 002 Design — Self Model Foundation

## 1. Why Task 002 is deliberately small

Task 001 proved that the desktop stack works.

Task 002 is the first real NOUS domain-code task.

The goal is correctness and clarity, not feature count.

This task must establish a strong base before database persistence, reasoning, or UI is added.

Under `MASTER_PLAN.md`, Task 002 is the next Foundation task. The master plan does not expand this scope.

## 2. Scope

Implement the first foundational domain types:

- `SelfSubject`
- `PersonReference`
- `Observation`
- `Situation`
- `Thought`
- `Emotion`

Also implement:
- shared identifier types/primitives as needed;
- validation required to maintain basic invariants;
- unit tests.

## 3. Out of scope

Task 002 must NOT implement:

- database schema changes;
- SQLite persistence for these entities;
- Belief;
- Value;
- Memory;
- Decision;
- Outcome;
- Evidence;
- Belief Graph;
- Philosophy Engine;
- Relationship Lens;
- prediction;
- automatic NLP extraction;
- Care detection;
- final UI/UX.
- response routing or adaptive interaction strategy;
- Expression Lab;

Those belong to later tasks.

## 4. Modeling rules

### SelfSubject

v0.x has exactly one deep modeling subject:
the current user.

`SelfSubject` is not a psychiatric/personality profile.

### PersonReference

Represents another real person only as relationship/context.

It may contain:
- identifier;
- user-chosen display name/nickname;
- relationship label;
- optional user-entered context notes.

It must NOT contain durable inferred:
- Beliefs;
- Values;
- Emotions;
- diagnoses;
- personality traits;
- future behavior predictions.

### Observation

Represents something observed/reported.

An Observation must remain distinguishable from the user's interpretation.

Example:

Observation:
> "No reply for five hours."

Thought:
> "They may not care about me."

### Situation

Represents context in which Thoughts/Emotions occur.

### Thought

Represents a momentary interpretation, appraisal, prediction, or internal statement.

A Thought is not automatically a fact or a durable Belief.

### Emotion

Represents a named emotional state with intensity.

A Situation may have multiple Emotions.

## 5. Validation/invariants

At minimum:

- IDs cannot be empty.
- Emotion intensity must be bounded.
- Thought confidence, if represented numerically, must be bounded.
- required text fields cannot be empty/whitespace.
- PersonReference must not expose fields for third-party inferred psychological state.
- model types should be serializable if doing so fits the existing Rust architecture cleanly.

Do not invent complex scoring.

## 6. Language

Internal identifiers/types use English.

User content may contain English, Simplified Chinese, or mixed-language text.

The model must not assume English-only content.

## 7. Implementation location

Prefer the Rust domain/core layer for Task 002 because these are core application entities, but first inspect the current Task 001 architecture.

If there is a strong technical reason not to place the domain model in Rust:
- stop;
- explain the reason;
- do not silently move the architecture boundary.

Do not duplicate the same domain model independently in Rust and TypeScript during this task.

## 8. Testing

Add focused unit tests covering:

- valid construction;
- rejection of invalid/empty required fields;
- numeric-bound validation;
- Unicode/Chinese text support;
- multiple emotions associated with a situation where modeled;
- third-party boundary represented structurally;
- serialization round-trip if serialization is added.

Prefer clear tests over large snapshot tests.

## 9. Success condition

Task 002 succeeds when NOUS has a tested, readable, framework-independent domain foundation for:

```text
SelfSubject
PersonReference
Observation
Situation
Thought
Emotion
```

with no UI or persistence coupling.

Task 002 reaches **COMPLETE** only after its implementation and tests pass, architectural boundaries are confirmed, the user reviews the result, and an approved Git checkpoint is created. Until then it is in progress or review.
