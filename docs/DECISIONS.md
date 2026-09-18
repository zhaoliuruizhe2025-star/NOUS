# NOUS Initial Design Decisions

This file records decisions already made so that later agents do not repeatedly reopen them without a concrete reason.

## D001 — North Star

NOUS helps a person gain a second perspective on themselves.

The product should make the user's own patterns more visible rather than define the user's identity for them.

Status: **Accepted**

## D002 — Product loop

Core loop:

**Remember → Connect → Reflect → Self-awareness**

Prediction is secondary.

Status: **Accepted**

## D003 — Modeling subject

The only deep modeling subject in v0.1 is the current user.

Other people are `PersonReference` context only unless a future, separately designed consensual multi-user feature is approved.

Status: **Accepted**

## D004 — Local-first

Core functionality and user data are local by default and do not require an external LLM/API.

Status: **Accepted**

## D005 — First desktop direction

Initial technical direction:

- Tauri 2
- React + TypeScript
- Rust/Tauri native layer
- SQLite
- Windows-first development
- macOS/Linux later

The exact TypeScript/Rust domain boundary remains revisable after the scaffold is inspected.

Status: **Accepted with boundary still open**

## D006 — Response routing

NOUS chooses among relief, practical help, reflection, decision support, philosophy, and care rather than deeply analyzing every message.

Status: **Accepted**

## D007 — Care over analysis

In significant distress, care takes priority over philosophical analysis, simulation, and prediction.

Status: **Accepted**

## D008 — Bilingual UI

User-facing UI supports English and Simplified Chinese.

Internal code/schema identifiers use English.

Status: **Accepted**

## D009 — Useful before insightful

Immediate usefulness takes priority over depth. `Action -> Explanation -> Reflection` is one available strategy where appropriate, not a universal template or required response format. Presence, philosophy, or care may require a different structure.

Status: **Accepted**

## D010 — Separate content understanding and interaction strategy

NOUS separates what the user's material may mean from how it should respond now. Interaction strategy adapts verbosity, question frequency, warmth, directness, initiative, solution priority, and reflection depth using non-clinical conversational-state signals.

Status: **Accepted**

## D011 — Temporary UI boundary

The current interface is functional scaffolding. A dedicated Product Experience / UI-UX milestone must occur before Private Beta and cover final visual identity, interaction flows, information hierarchy, bilingual typography, and replacement of temporary surfaces. Domain logic must remain independent of the scaffold.

Status: **Accepted**

## D012 — Governed task hierarchy

Development follows:

```text
Vision -> Architecture -> Epics -> Phases/Milestones -> Tasks -> Git checkpoints
```

Completed work remains historical. Changes require new explicit tasks. A task is complete only after implementation, passing checks, architectural review, user review, and an approved Git checkpoint.

Status: **Accepted**

## D013 — Private Beta before public release

NOUS will be evaluated with a small informed Private Beta before any public/open-source release. The Product Experience / UI-UX and Care / Safety implementation milestones are pre-beta gates. Beta feedback and safety/privacy review precede public release planning.

Status: **Accepted**

## D014 — Future features remain outside the active path

Relationship Lens is perspective expansion, not third-party mind-reading. Expression Lab is a post-beta feature that helps users express feelings they already have rather than fabricate them. Digital Self remains future experimental work. These remain deferred unless explicitly promoted and assigned approved tasks.

Life Paths is deferred from the first Private Beta as a scope reduction, not removed from the long-term NOUS product vision.

Status: **Accepted**

## D015 — Self Model expansion is staged by domain boundary

Task 003 is limited to user-owned Beliefs, Values, and their append-only revision records. Belief and Value carry `SelfSubject` ownership; their revisions inherit it through the parent ID rather than duplicating a subject field. Belief endorsement is a user-entered degree of endorsement, not truth, evidence strength, prediction confidence, or system certainty. Value importance is user-entered relative salience, not truth-confidence or moral ranking.

Revision origins are limited to `InitialUserEntry`, `UserUpdate`, and `UserCorrection`. A correction is not psychological change. Future system proposals are separate traceable artifacts, not revision origins, and require user confirmation before they become canonical. Task 003 validates positive revision numbers only; Task 004 enforces cross-history uniqueness and strict sequencing unless a separately approved aggregate is introduced. A materially new commitment creates a new Belief or Value; when continuity is ambiguous, the user decides. An Observation is not Evidence, though it may later be referenced as an Evidence source and never automatically updates a commitment.

Evidence, Memory, Decision, and Outcome are deferred: Task 004 remains the persistence foundation; Task 005 groups Memory with Decision/Outcome as lived-experience records; Task 006 may introduce generic Evidence and explicit relationships after its source model is broad enough. A Thought never automatically becomes a Belief.

Status: **Accepted**

## D016 — Conversational product model and memory layers

The normal experience is a minimal conversational surface, not a manual psychological database. Raw History is remembered by default; relevant material may be represented as Structured Experience; only sufficiently supported material may enter the Durable Self Model. Everything can be remembered; not everything becomes who the person is. Bootstrap onboarding is short, conversational, and provisional; it does not produce a first-run personality report or early Self Model dashboard.

Status: **Accepted**

## D017 — Epistemic humility, correction, and directional help

NOUS distinguishes user report, concrete event details available from the user or another source with provenance preserved, user interpretation, and tentative system interpretation. It clarifies before inferring with the minimum necessary clarification. Natural-language correction preserves history while revising the current canonical understanding; `UserCorrection` remains distinct from `UserUpdate`. NOUS may be directional but not directive: a slight, reasoned leaning is allowed, while commands and winner-ranking are not. It may surface relevant patterns and self-concept discrepancies lightly and gently.

Status: **Accepted**

## D018 — Academic Foundation and experimental expression remain future work

Personal pattern comes before theory. Frameworks are lenses, not verdicts, and their evidence categories and provenance remain distinct. A future Academic Foundation is not an implementation authorization. Expression Lab remains post-beta/future and uses verified scientific principles, constrained composition, diversity tracking, and a scientific-integrity gate; its default style is restrained romanticism. Direct Self Model inspection and ambient expression are experimental future-facing surfaces.

Status: **Accepted**
