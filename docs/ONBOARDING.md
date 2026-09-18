# NOUS Conversational Onboarding

## 1. Purpose and status

This document records the approved future product design for first-run onboarding. It authorizes neither UI nor model implementation.

NOUS should become useful before months of accumulated history exist. A short mandatory conversational onboarding creates a **Bootstrap Self Model**: enough provisional context to personalize early help without claiming complete understanding.

## 2. Experience

The experience remains in the normal, minimal conversational surface: one natural free-text question at a time. It should take only a few minutes and a small number of high-information-density questions, without freezing an exact duration or count.

Questions should draw on concrete experiences, consequential choices, current concerns, trade-offs, relationships, uncertainty, desired futures, and values revealed through choices. One question may reveal several useful dimensions, but should not hide a questionnaire inside a long list of subquestions.

Avoid Likert scales, clinical questionnaires, fixed long scripts, and explicit personality labeling. Later questions should follow the information-gain path:

```text
answer -> identify useful uncertainty -> choose the next natural question
```

## 3. Provisional model and transition

The lifecycle is conceptual:

```text
Bootstrap Model -> Accumulating Model -> Evidence-backed Self Model
```

NOUS can begin understanding a person on day one, but must not pretend to have understood them completely on day one. It may use the model immediately, while keeping it easy to revise as lived evidence accumulates. Do not invent numeric psychological precision or issue a first-run personality report.

After onboarding, a restrained acknowledgement such as “I’m starting to get to know you” is enough; enter the normal conversation rather than presenting a profile, scorecard, or “You are the kind of person who…” summary. The model should reveal itself through usefulness, not through labels.

## 4. Visibility and correction

The full Self Model remains primarily background context in the early experience. A carefully designed Mirror/Self inspection surface is a later experimental feature, not a core onboarding dashboard.

Users correct the model in ordinary language. Corrections must adjust the current conversation and cause affected assumptions to be reconsidered, without requiring users to maintain records manually. See `SELF_MODEL.md` for correction-without-erasure semantics.
