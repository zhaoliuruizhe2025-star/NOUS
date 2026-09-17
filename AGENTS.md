# NOUS — Agent Instructions

These instructions apply to coding agents working in this repository.

## 1. Read before changing code

Before modifying code:

1. Read `docs/PRODUCT.md`.
2. Read `docs/PRINCIPLES.md`.
3. Read `docs/ARCHITECTURE.md`.
4. Read `docs/DECISIONS.md`.
5. Read `docs/RESPONSE_POLICY.md` when implementing user-response behavior.
6. Read the document relevant to the requested feature.
7. Read the current task specification.
6. Inspect existing tests and code before introducing new abstractions.

Do not infer the product from a short task prompt when the repository documentation already defines it.

## 2. Scope discipline

- Implement only the requested task.
- Do not build future roadmap features early.
- Do not redesign the Self Model without explicit approval.
- Do not introduce a runtime LLM/API dependency without explicit approval.
- Do not add cloud sync, telemetry, analytics, remote logging, advertisements, or account systems without explicit approval.
- Prefer small, reviewable changes.

If a requested implementation conflicts with project documents, stop and report the conflict rather than silently choosing a new direction.

## 3. Local-first security boundary

NOUS must be isolated by default.

The application must not, without explicit feature-level approval:

- require administrator/root privileges;
- modify operating-system settings;
- modify global environment variables;
- install or run background services;
- enable auto-start;
- inspect unrelated user files;
- scan the user's home directory;
- read browser history;
- inspect other applications or processes;
- execute arbitrary shell commands at runtime;
- access the network;
- upload user data;
- write outside NOUS-managed application data or a file explicitly selected by the user.

Development commands may of course compile/test the project, but runtime application behavior must respect this boundary.

## 4. Data integrity

- Historical beliefs, values, thoughts, emotions, decisions, and revisions must not be silently destroyed.
- Prefer append/revision semantics when modeling change over time.
- Use database transactions for multi-step writes.
- Migrations must be versioned.
- Destructive migrations require explicit approval and a backup/migration plan.
- Do not silently reinterpret old user data after schema changes.

## 5. Explainability

Core reasoning must produce structured reasons that can be inspected.

A user-facing conclusion such as a tension, pattern, or prediction should ultimately be traceable to:
- relevant user observations;
- model entities/relationships;
- framework rules, if applicable;
- uncertainty/limitations.

Do not implement opaque scoring that cannot be explained.

## 5.1 Response routing

Do not turn every user statement into psychological interpretation.

When implementing response behavior, distinguish at least:
- relief;
- practical help;
- reflection;
- decision support;
- philosophical exploration;
- care.

Prefer immediate, low-risk practical relief when the user is simply frustrated, overwhelmed, tired, or upset and has not asked for deep analysis.

Do not update the durable Self Model from a casual statement without sufficient evidence and/or user confirmation.

See `docs/RESPONSE_POLICY.md`.

## 5.2 Third-party modeling boundary

NOUS models the current user, not unconsenting third parties.

Other real people may be represented only as contextual references to the user's own experiences.

Do not implement features that infer or persist an unconsenting third party's:
- personality;
- attachment style;
- psychiatric state;
- beliefs/values;
- hidden motives;
- deception probability;
- future behavior probability;
- relationship outcome probability.

If a user describes another person, keep inferences centered on:
- what the user observed;
- how the user interpreted it;
- how the user felt;
- what the user did.

Do not silently turn `PersonReference` into another `SelfSubject`.

See `docs/SELF_MODEL.md` and `docs/PRINCIPLES.md`.

## 6. Prediction and decision behavior

NOUS must never output:
- "You should choose X."
- "NOUS recommends X."
- "X is the best choice."
- equivalent ranking language that turns reflection into a life recommendation.

Prediction is descriptive:
- "Your past patterns suggest you may lean toward X."
- confidence and limitations must be available;
- prediction must be distinguishable from recommendation.

Do not reduce a high-impact decision to a single winner score in the primary UI.

## 7. Mental-health and care behavior

NOUS is not a diagnostic tool.

Do not:
- diagnose depression or any psychiatric condition;
- assign psychiatric labels;
- estimate a suicide probability;
- present self-harm or suicide as an ordinary Life Path;
- optimize, simulate, or encourage self-harm;
- punish or shame vulnerable thoughts.

When Care Layer rules indicate that analysis should pause, safety/care behavior takes precedence over philosophy, simulation, and prediction.

See `docs/SAFETY.md`.

## 8. Voice and localization

- Internal identifiers are English.
- User-facing UI supports English and Simplified Chinese.
- Do not mix translated strings directly into business logic.
- Do not mechanically translate nuanced care/reflection copy.
- Reflection may be literary; care responses must become simpler, warmer, and more grounded.

See `docs/VOICE_AND_I18N.md`.

## 9. Quality requirements

For core logic:
- add tests;
- include edge cases;
- avoid hidden global mutable state;
- keep domain logic separate from presentation;
- document non-obvious algorithms;
- prefer deterministic behavior where practical;
- avoid premature optimization.

A successful build is not sufficient evidence of correctness.

## 10. Environment changes

Do not silently install system-wide dependencies or change system configuration.

If a required development dependency is missing:
1. report what is missing;
2. provide the exact prerequisite;
3. wait for explicit user approval before any system-level installation or configuration change.

## 11. Completion report

After each task, report:

1. files created/modified;
2. major implementation decisions;
3. tests/checks run and their results;
4. anything not completed;
5. any architectural or safety concern discovered;
6. suggested next task, without implementing it unless asked.
