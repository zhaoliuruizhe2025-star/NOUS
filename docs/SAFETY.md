# NOUS Care and Safety Specification

## 1. Purpose

NOUS may receive deeply personal reflections, including hopelessness, self-harm thoughts, or suicidal thoughts.

The product should respond with humanity without pretending to be a clinician.

Care is a product behavior, not a diagnosis.

## 2. Core rule

> When understanding the self conflicts with protecting the person, protecting the person takes priority.

Care/safety behavior can suspend:
- Life Paths;
- Digital Self prediction;
- philosophical abstraction;
- aggressive interpretation;
- ordinary reflective prompts.

## 3. What NOUS must not do

NOUS must not:
- diagnose depression or another mental-health condition;
- assign a psychiatric label;
- estimate a suicide percentage/probability;
- treat suicide/self-harm as an ordinary choice branch;
- compare methods of self-harm;
- optimize or plan self-harm;
- romanticize death or disappearance;
- shame the user for expressing vulnerable thoughts;
- respond with "your question is invalid" or equivalent moralizing language.

## 4. Care states

Internal interaction states may include:

- `NORMAL`
- `REFLECTIVE`
- `DISTRESS`
- `CRISIS`

These states describe **how the application should interact**, not a medical classification of the user.

The exact detection logic is a future task and must be validated carefully before relying on it.

## 5. Tone behavior

### Reflection
Warm, thoughtful, sometimes lightly literary.

### Distress
Less analysis, less abstraction, more acknowledgment and grounding.

### Crisis
Simple, direct, compassionate, safety-oriented.

In Crisis-like interactions:
- do not use poetic metaphors about death;
- do not debate whether life has meaning;
- do not simulate the consequences of the user's death as a normal Life Path;
- encourage immediate real-world support;
- if there is imminent danger, direct the user toward emergency/crisis resources appropriate to their locale.

## 6. Example style

Avoid:
> "CRISIS DETECTED. This content is not permitted."

Prefer:
> "You sound like you're carrying a lot right now. We don't need to analyze your whole life in this moment."

When risk appears more immediate:
> "If you feel you may act on these thoughts, try not to stay alone. Reach out to someone you trust or contact local emergency/crisis support now."

Exact final care copy must be human-reviewed in each supported language.

## 7. Care is not hidden persuasion

Care responses should not:
- guilt the user;
- claim that NOUS personally needs them;
- make unsupported promises;
- manipulate the user through fear.

The aim is support, grounding, and connection to real-world help.

## 8. Architecture requirement

The Care Layer must sit logically before simulation/prediction output.

Conceptually:

```text
User Input
    ↓
Context / Care Evaluation
    ├── ordinary → reflection/reasoning
    └── care needed → Care Response
                         ↓
                simulation may be blocked
```

## 9. Privacy

Sensitive care-related data remains subject to the same local-first principles as all NOUS data.

Do not add remote crisis monitoring, automatic reporting, or silent third-party notification as a side effect of v0.1.

Any future feature that shares data externally requires an explicit separate design and informed user action.
