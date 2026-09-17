# NOUS Self Model

## 1. Purpose

The Self Model is an explicit, inspectable representation of information the user has provided and patterns the system has derived.

It is not a claim to reproduce consciousness, identity, or a soul.

The model should preserve the difference between:
- raw user input;
- structured observations;
- interpretations;
- longer-term beliefs/values;
- system-derived relationships.

## 2. Modeling subject and third-party boundary

### 2.1 SelfSubject

In v0.1 there is exactly one deep modeling subject:

- the current NOUS user.

Beliefs, Values, Thoughts, Emotions, Memories, Decisions, derived patterns, and predictions belong to this SelfSubject.

NOUS must not create a second full SelfSubject for an unconsenting real person.

### 2.2 PersonReference

Other people may appear in the user's records as lightweight contextual references.

Examples:
- partner;
- parent;
- friend;
- professor;
- coworker.

A `PersonReference` may contain only what is necessary to connect the user's own situations and memories, such as:
- id
- user-chosen display name or nickname
- relationship label
- optional notes explicitly entered by the user

A PersonReference must not own a psychological profile.

Do not attach durable inferred:
- Beliefs
- Values
- Emotions
- personality traits
- diagnoses
- behavioral predictions

to a third-party PersonReference.

### 2.3 Perspective rule

When the user says:

> "Alex does not care about me."

NOUS should represent the user's perspective, for example:

- Situation/Observation: Alex has not replied for three days.
- Thought: "Alex does not care about me."
- Emotion: sadness/anxiety.

It must not silently store:

- `Alex.belief = does_not_care_about_user`
- `Alex.emotion = indifference`
- `Alex.prediction = likely_to_leave`

The system models what the event means **to the user**.


## 3. Core entities

### 2.1 Observation

An `Observation` is a record of something the user reports noticing or providing.

Examples:
- "My friend did not reply for five hours."
- "I have an offer to study abroad."
- "I noticed I felt calmer after making the decision."

An observation is evidence available to the model. It is not automatically a belief update.

Suggested conceptual fields:
- id
- source
- content
- timestamp
- language
- userConfirmed
- createdAt

### 2.2 Situation

A `Situation` describes the context in which thoughts, emotions, and decisions occur.

Examples:
- receiving an offer;
- conflict with a friend;
- an upcoming exam;
- being alone late at night thinking about meaning.

Suggested fields:
- id
- description
- startTime / timestamp
- relatedObservations
- tags

### 2.3 Thought

A `Thought` is a relatively immediate interpretation, appraisal, prediction, or internal statement.

Examples:
- "If I leave, I may hurt people I love."
- "This might be my only opportunity."
- "Maybe this person does not care about me."

A Thought is not automatically an objective fact and is not necessarily a stable Belief.

Suggested fields:
- id
- content
- timestamp
- confidence
- relatedSituation
- relatedEmotions
- relatedBeliefs
- relatedMemories

### 2.4 Emotion

An `Emotion` represents a named felt state at a time or within a situation.

Examples:
- sadness;
- anxiety;
- guilt;
- anger;
- hope;
- excitement.

Suggested fields:
- id
- label
- intensity
- timestamp
- relatedSituation
- relatedThoughts

A single situation may contain multiple emotions.

### 2.5 Belief

A `Belief` is a relatively persistent proposition the user currently accepts to some degree.

Examples:
- "A meaningful life is constructed through choices."
- "Important opportunities may not come again."
- "People are morally responsible for their actions."

Suggested fields:
- id
- proposition
- confidence
- importance
- status
- createdAt

Belief change must use revisions/history rather than replacing the past.

### 2.6 BeliefRevision

Represents a Belief at a particular point in time.

Suggested fields:
- id
- beliefId
- confidence
- wording
- timestamp
- evidenceLinks
- reasonForRevision

### 2.7 Value

A `Value` describes what matters to the user, not what is true or false.

Examples:
- autonomy;
- family;
- stability;
- knowledge;
- authenticity;
- achievement;
- compassion.

Do not use "confidence" as the primary meaning of a Value.

Suggested fields:
- id
- canonicalName
- userLabel
- importance
- createdAt

Value importance may change over time and should therefore support revisions/history.

### 2.8 ValueRevision

Suggested fields:
- id
- valueId
- importance
- timestamp
- evidenceLinks
- reasonForRevision

### 2.9 Memory

A `Memory` is a user-described past experience that the user considers relevant to their current self.

NOUS does not assume autobiographical memory is a perfectly objective record.

Suggested fields:
- id
- description
- eventTime / approximateTime
- recordedAt
- emotionalWeight
- relatedBeliefs
- relatedValues
- userMeaning

### 2.10 Decision

A `Decision` is a choice made by the user in a defined context.

Suggested fields:
- id
- situationId
- description
- options
- chosenOption
- timestamp
- relatedValues
- relatedBeliefs
- anticipatedOutcome

### 2.11 Outcome

Represents what the user later reports happened after a Decision.

Suggested fields:
- id
- decisionId
- description
- timestamp
- laterEvaluation
- relatedObservations

### 2.12 Evidence

`Evidence` links observations/experiences/decisions to possible model updates.

Evidence should not mutate a Belief or Value by itself.

Suggested fields:
- id
- sourceEntityType
- sourceEntityId
- targetEntityType
- targetEntityId
- direction
- strength
- timestamp
- explanation

## 4. Relationships

The graph layer may support relationships such as:

- `SUPPORTS`
- `CONTRADICTS`
- `DEPENDS_ON`
- `IMPLIES`
- `ASSOCIATED_WITH`
- `INFLUENCES`
- `EVIDENCE_FOR`
- `EVIDENCE_AGAINST`

Not all relationship types should apply to all entity types.

The schema should make illegal combinations difficult to create.

## 5. Important distinctions

### Thought vs Belief

Thought:
> "Maybe this person does not care about me."

Belief:
> "Close relationships are inherently unstable."

A recurring Thought may contribute evidence toward a Belief, but repeated occurrence alone must not automatically prove that Belief is true.

### Value vs Belief

Value:
> "Autonomy is important to me."

Belief:
> "People live better lives when they have autonomy."

These are related but not interchangeable.

### Observation vs Interpretation

Observation:
> "The message has not received a reply for five hours."

Interpretation:
> "They are ignoring me."

NOUS should preserve this distinction.

## 6. Conceptual processing loop

A simplified loop:

```text
Observation / Situation
        ↓
      Thought
        ↕
      Emotion
        ↓
     Decision
        ↓
      Outcome
        ↓
   New Observation
        ↓
      Evidence
        ↓
Self Model Revision
```

Additional influences include:

```text
Memory ───────→ Thought
Belief ───────→ Thought
Value ────────→ Decision
Emotion ──────→ Thought / Decision
Outcome ──────→ Memory / Evidence
```

This is not claimed to be a complete psychological theory. It is a practical computational model.

## 7. Temporal model

A person's current state must not erase their past state.

Example:

```text
Belief: "People have free will."

2026-09   confidence 0.82
2027-02   confidence 0.61
2027-10   confidence 0.34
```

The current view can use the latest revision while Mirror can reconstruct earlier states.

## 8. Derived patterns

NOUS may derive patterns such as:
- recurring Thought patterns;
- repeated value trade-offs;
- contradictions/tensions;
- differences between stated Values and past Decisions;
- changing interpretations across similar Situations.

Derived patterns must store:
- evidence references;
- calculation/rule version;
- timestamp;
- uncertainty.

## 9. User correction

A user can reject or revise system interpretation.

The system should preserve:
- what NOUS inferred;
- whether the user accepted/rejected it;
- the corrected interpretation, if provided.

User correction is valuable evidence and should not be treated as an error condition.

## 10. v0.1 boundary

v0.1 models and stores the entities above, but does not need:
- advanced probabilistic prediction;
- automatic NLP extraction from unrestricted text;
- clinical inference;
- a complete psychological theory;
- autonomous philosophy reasoning.

Manual or semi-structured user entry is acceptable in the first prototype.
