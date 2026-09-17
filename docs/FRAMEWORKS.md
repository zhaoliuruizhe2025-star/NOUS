# NOUS Framework Library Specification

## 1. Goal

NOUS may eventually use ideas from psychology, philosophy, ethics, and decision theory as transparent interpretive lenses.

The Framework Library must not become a pile of quotations or an authority engine that decides how a person should live.

## 2. Framework categories

At minimum distinguish:

- `EMPIRICAL_PSYCHOLOGY`
- `CLINICAL_OR_THERAPEUTIC_MODEL`
- `PHILOSOPHICAL_FRAMEWORK`
- `ETHICAL_FRAMEWORK`
- `DECISION_THEORY`
- `USER_DEFINED_FRAMEWORK`

These categories should not be treated as equivalent forms of evidence.

A philosophical argument is not assigned a fake scientific confidence score merely because it is in the same library as an empirical psychological model.

## 3. Framework rule concept

A future structured rule may include:

- id
- frameworkId
- concept
- category
- premises
- applicability conditions
- interpretation/inference
- limitations
- counterpoints
- source citation
- source edition/version
- rule version

## 4. Multi-lens behavior

Multiple frameworks may interpret the same situation.

Example:

- cognitive lens: distinguish an event from an interpretation/prediction;
- Stoic lens: distinguish controllable actions from external outcomes;
- existential lens: examine choice, responsibility, identity, and situated freedom;
- self-determination lens: examine autonomy, competence, and relatedness.

NOUS must not:
- tally votes;
- declare the framework with the most supporting rules the winner;
- convert multiple traditions into a single "correct philosophy" score.

## 5. Source discipline

Every framework entry must have a traceable source.

Preferred source order:
1. primary text when legally and practically appropriate;
2. high-quality scholarly/reference source;
3. reputable academic/clinical source;
4. carefully documented secondary interpretation.

Do not copy large copyrighted passages into the repository.

Store:
- original summaries written for NOUS;
- structured concepts/rules;
- brief necessary quotations only when legally appropriate;
- citations.

## 6. Psychological humility

A theoretical or therapeutic concept may help organize reflection but must not be used to diagnose a user.

Claims about empirical support should be kept separate from philosophical interpretation.

## 7. First-release boundary

v0.1 does not require a full Framework Library.

For v0.1:
- define interfaces/data structures only if needed;
- avoid hard-coding a large theory database;
- focus first on Self Model correctness.

Framework content should be added incrementally and reviewed.
