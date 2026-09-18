# NOUS UI/UX Boundary

## 1. Current status

The current Task 001 interface is a temporary functional scaffold.

It is NOT the approved visual identity of NOUS.

The visual identity is frozen until the dedicated UI/UX phase in the master plan.

## 2. Until a dedicated UI/UX design phase

Coding agents may create only the minimum UI required to verify a feature.

They must not independently finalize:

- brand identity;
- typography system;
- color palette;
- animation language;
- icon style;
- final navigation structure;
- final page composition;
- visual metaphors;
- dark-mode design;
- final Chinese/English typographic treatment.

## 3. Allowed temporary UI

Allowed:
- basic buttons;
- basic inputs;
- temporary forms;
- simple navigation placeholders;
- labels needed for feature verification;
- test/debug surfaces.

## 4. Dedicated pre-beta design phase

The Product Experience / UI-UX milestone must occur before Private Beta. It will replace the Task 001 scaffold and define:

- final visual identity;
- interaction flows for approved workflows;
- information hierarchy;
- bilingual English/Simplified Chinese typography and layout behavior;
- replacement of temporary navigation, forms, and verification surfaces.

The work will be designed separately and may be documented in future files such as:

```text
docs/UI_UX.md
docs/DESIGN_SYSTEM.md
```

Until those documents exist, functional scaffolding must remain easy to replace.

## 5. Separation requirement

Domain logic must not depend on temporary visual components.

A future redesign should not require rewriting the Self Model.

This document constrains implementation but does not itself authorize UI work. UI/UX ideas remain outside active tasks unless explicitly scoped.
