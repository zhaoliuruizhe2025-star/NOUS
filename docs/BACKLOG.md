# NOUS Backlog

## 1. Purpose

This document separates committed active-roadmap work from ideas that are not authorized for implementation.

Listing an item here does not create a task. Promotion requires an explicit planning decision, an approved scope, and a task specification under `docs/TASK_SYSTEM.md`.

## 2. Active roadmap

These outcomes are on the path from the current foundation to Private Beta:

- v0.2 master planning checkpoint;
- Task 002 Self Model foundation (complete);
- Task 003 Belief/Value commitments and revision history (complete at `e970138`);
- Task 004 local persistence and application services for approved entities;
- Task 005 Memory, Decision, and Outcome;
- Task 006 Evidence foundation and explicit relationships;
- basic Remember workflows;
- Phase C explicit connections, evidence-grounded Pattern/Candidate reasoning, model validation, counterevidence, context/time scope, and faithful reasoning traces under `docs/REASONING_MODEL.md` and `docs/TASK_012_DESIGN.md`; transient user-proposed Candidate Review is complete at Task 013. Task 015 is complete at `f937936` with strictly transient user-assisted semantic comparison. `docs/TASK_016_DESIGN.md` records the Owner-approved architecture direction: Task 015 output does not authorize direct system Candidate generation; a complete deterministic User-Grounded Candidate System v1 mechanism is the preferred next design, with a separate Candidate Product Test required after any approved implementation. Autonomous psychological-semantic generation and v1 implementation remain unauthorized; Task 017 has not started. A local semantic model remains a future optional research direction, not a current requirement;
- Phase D response alignment, relevant Self Model selection, requested advice from validated reasoning, explainability, adaptive interaction, and bilingual semantic consistency under `docs/REASONING_MODEL.md`;
- Mirror, correction, and evidence views;
- Product Experience / UI-UX work that replaces the Task 001 scaffold before beta;
- Care routing, distress/crisis behavior, bilingual human-reviewed care copy, and safety-focused tests;
- export/recovery, privacy, and beta-readiness work;
- Private Beta with a small informed group.

The ordering and task splits are governed by `docs/MASTER_PLAN.md`. Only a dedicated task specification authorizes implementation.

## 3. Later candidate work

Potential work that may follow evidence from the active roadmap:

- richer curated Framework Library;
- expanded Inner Map exploration;
- Life Paths trade-off exploration, deferred from the first Private Beta as scope reduction rather than removed from the long-term product vision;
- cross-platform hardening beyond the beta target;
- richer import/export and research export;
- optional on-device language assistance, subject to a separate privacy and architecture decision.

These candidates are not current commitments.

Life Paths remains a long-term NOUS capability. Its position here means only that it is not required for the first Private Beta.

## 4. Post-beta features

The following must not silently enter the active development path:

- **Relationship Lens** — perspective expansion around the user's relationships, not third-party mind-reading;
- **Expression Lab** — helps users express feelings they already have across relationship and intellectual contexts;
- **Digital Self** — experimental prediction with calibration and explainability;
- consensual shared relationship spaces;
- optional encrypted sync;
- mobile companion;
- external AI-provider integrations;
- broad public/open-source release work beyond beta feedback and review.

See `docs/RELATIONSHIP_LENS.md` and `docs/EXPRESSION_LAB.md` for feature boundaries.

## 5. Research ideas

Research items require evidence and may never become product features:

- evaluation methods for “second perspective” usefulness;
- qualitative calibration of Pattern claims and research into prediction uncertainty, without a numeric Pattern/Candidate confidence score;
- measuring when reflection feels helpful, obvious, intrusive, or overly deep;
- transparent multi-framework disagreement;
- methods for distinguishing stable patterns from temporary state;
- safe, non-clinical interaction-state adaptation;
- private, local evaluation of personalized expression style;
- how much structure improves self-awareness without making the product burdensome.
- whether, and under what evidence/consent conditions, NOUS should reuse strategies that previously helped the person in similar situations. Decision/Outcome history may support this later, but no active behavior is approved.

## 6. Promotion rule

To move an item toward implementation:

1. record the product reason and evidence;
2. identify architecture, privacy, safety, and data implications;
3. decide whether it belongs before or after Private Beta;
4. update the master plan and decisions if its priority changes;
5. create a bounded task specification;
6. obtain explicit approval before implementation begins.
