# NOUS Beta and Release Strategy

## 1. Private beta first

Before a public open-source release, NOUS should be tested by a small trusted group.

Private Beta is a required milestone, not an optional release tactic. No public/open-source release should precede beta feedback and a separate safety/privacy review.

Initial testers:
- the project owner;
- trusted friends who knowingly choose to participate.

The purpose of the private beta is not to prove that every planned feature works.
It is to discover whether NOUS actually helps people see themselves more clearly.

## 2. Questions the beta should answer

Measure qualitatively:

- Do users want to keep using it?
- Does a reflection feel useful rather than generic?
- Does NOUS over-interpret casual comments?
- Does it sometimes feel intrusive?
- Do users experience a genuine "I had not noticed that about myself" moment?
- Are Relief / Reflection / Decision / Care modes routed appropriately?
- Does interaction adapt appropriately to frustration, anxiety, loneliness, sadness, fatigue, philosophical exploration, and distress without diagnosing the person?
- Does NOUS act usefully before trying to be insightful?
- Are questions, verbosity, warmth, directness, initiative, solution priority, and reflection depth appropriate to the moment?
- Are abstract ideas grounded in the user's real situations when relevant history is available?
- Is the Inner Map understandable?
- Do users trust the local-first privacy model?
- Is Chinese and English wording natural?

## 3. Feedback model

Future beta UI may include lightweight feedback such as:

```text
This reflection was:

[ Helpful ]
[ Too obvious ]
[ Too deep ]
[ Incorrect ]
[ Uncomfortable ]
```

Feedback is itself product research and should not automatically mutate the user's Self Model.

## 4. Open-source milestone

The first public/open-source version should follow:

```text
Prototype
→ Private beta
→ Feedback
→ Redesign
→ Safety/privacy review
→ Public release
```

Do not publish merely because the application runs.

Relationship Lens, Expression Lab, Digital Self, and other post-beta work are not prerequisites for the first beta and must not enter it without explicit roadmap promotion.

Life Paths is also not required for the first Private Beta, but this scope reduction does not remove it from the long-term NOUS product vision.

## 5. Pre-beta product and care gates

Private Beta begins only after:

- the Product Experience / UI-UX milestone replaces the Task 001 temporary scaffold with reviewed beta interaction flows, information hierarchy, visual identity, and bilingual typography;
- the Care / Safety milestone implements Care routing, distress/crisis behavior, human-reviewed bilingual care copy, safety-focused tests, and Care precedence over ordinary reflection/prediction.

These gates require separately scoped tasks; this strategy does not implement them.

## 6. Beta privacy

Private-beta participants should understand:
- the software is experimental;
- their local data remains theirs;
- the project may change;
- NOUS is not a diagnostic or medical tool.

No telemetry or automatic remote collection is required for beta feedback.

Feedback should be collected through explicit user action and treated separately from durable Self Model updates.
