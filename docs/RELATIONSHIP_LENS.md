# NOUS Relationship Lens — Future Feature Specification

Status: **POST-BETA / FUTURE**.

This is a perspective-expansion feature, not third-party mind-reading. It must not be implemented during Task 002 or silently enter the active Private Beta path. Promotion requires an explicit master-plan decision and a separately approved task.

## 1. Purpose

Relationship Lens helps the user step outside an emotionally charged interaction and consider multiple plausible interpretations.

Examples:
- romantic partners;
- parent/child relationships;
- friends;
- manager/employee relationships;
- other important interpersonal contexts.

The goal is not mind-reading.

The goal is perspective expansion.

It remains subordinate to NOUS's primary purpose: helping the current user understand their own experience and choices.

## 2. Key distinction

NOUS may say:

> "One possible interpretation is that your partner was asking for reassurance."

NOUS must not say:

> "Your partner's real motive was to get reassurance."

Inference about another person's internal state is always uncertain unless that person explicitly provides their own perspective.

## 3. Single-user Relationship Lens

When only the current user participates, NOUS may use:

- observable interaction history;
- the user's own interpretations;
- the user's own emotional responses;
- repeated interaction patterns;
- alternative plausible explanations.

It may output multiple possibilities.

Example:

```text
Statement:
"You don't love me anymore."

Possible interpretations:
1. literal doubt about the relationship;
2. request for reassurance;
3. expression of hurt or neglect;
4. escalation during conflict.

Insufficient information to know which interpretation is true.
```

## 4. Third-party boundary

Without the other person's explicit participation, NOUS must not present as fact:

- personality type;
- attachment style;
- psychiatric diagnosis;
- hidden motives;
- emotions;
- beliefs;
- values;
- deception probability;
- breakup probability;
- future-behavior probability.

A `PersonReference` remains contextual, not a second full Self Model.

## 5. Shared/consensual future mode

A later version may support a shared relationship space if both people explicitly participate.

Each participant controls what they share.

Possible shared dimensions:
- relationship values;
- communication preferences;
- current concerns;
- selected reflections.

Private memories, full journals, and complete belief graphs must not be shared by default.

## 6. Future research question

Relationship Lens should ask:

> "What other interpretations might help the user understand the interaction?"

not:

> "Can NOUS secretly discover what the other person is really thinking?"

## 7. Backlog boundary

This specification records product constraints only. It does not approve data-model changes, shared spaces, relationship predictions, UI work, or implementation of any kind. Its canonical priority is tracked in `BACKLOG.md`.
