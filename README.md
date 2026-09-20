# NOUS

> NOUS helps you step outside your own perspective and see the person you have been becoming.

## North Star

NOUS is designed to help people develop a second perspective on themselves.

It remembers, connects, and reflects the user's own experiences so that patterns that are difficult to see from inside everyday life can become visible.

NOUS is a mirror, not an authority: it does not claim to know the user's "true self," and it does not build unconsented psychological profiles of other people.

## Status

Task 001 is complete at `455e929`, providing the runnable desktop scaffold. Task 002 is complete at `e324852`, providing the first narrow Rust Self Model domain slice. Task 003 is complete at `e970138`, providing the Belief/Value commitments and revision-history domain model.

NOUS is not intended to tell a person who they are, diagnose mental-health conditions, or decide how they should live. It aims to build an inspectable model from a user's own observations, beliefs, values, memories, thoughts, emotions, and decisions, and then help the user reflect on patterns and tensions over time.

## Product vision

NOUS is designed around five long-term capabilities:

1. **Mind Mirror** — show how the user's thinking, values, and recurring interpretations change over time.
2. **Inner Map / Belief Graph** — represent important beliefs, values, and their relationships.
3. **Philosophy Engine** — surface tensions and alternative interpretations without declaring one worldview correct.
4. **Life Paths** — explore trade-offs among possible choices and offer requested, grounded directional advice while leaving final decision authority with the user.
5. **Digital Self** — experimentally test how well an explicit self-model can anticipate responses to new situations.

A cross-cutting **Care Layer** takes priority whenever analysis would be inappropriate or potentially harmful.

The product loop is:

```text
Remember -> Connect -> Reflect -> Self-awareness
```

NOUS should be useful before it tries to be insightful. Where appropriate, responses may use `Action -> Explanation -> Reflection` as one strategy, with interaction behavior adapted to the person's current state rather than only the literal sentence.

## Core product principles

- Local-first and privacy-first.
- No external LLM or AI API is required for core functionality.
- User-facing conclusions must be explainable.
- NOUS describes, reflects, and may provide requested, grounded recommendations; the user retains final decision authority. Advice follows `docs/PRINCIPLES.md` §13.1.
- Historical self-model data is preserved rather than silently overwritten.
- Care takes priority over analysis when a user appears to be in significant distress.
- The UI supports English and Simplified Chinese.
- Internal code, identifiers, schemas, rules, and logs use English.
- Content understanding and interaction strategy are separate, testable concerns.
- The current UI is replaceable functional scaffolding; visual identity is deferred to a dedicated UI/UX phase.

See `docs/` and `AGENTS.md` before implementing features.

## Planned technical direction

- Desktop application
- Tauri 2
- React + TypeScript UI
- Rust application/core layer
- SQLite local storage
- Windows-first development, with macOS and Linux as supported targets after the Windows alpha stabilizes

The exact boundary between TypeScript and Rust may evolve after the initial scaffold. Do not redesign that boundary without documenting the reason first.

## Development

Prerequisites are listed in `docs/ARCHITECTURE.md`. From the repository root:

```powershell
npm install
npm run tauri:dev
```

Quality checks:

```powershell
npm run typecheck
npm run lint
npm test
npm run build
npm run tauri:check
npm run tauri:build
cargo test --manifest-path src-tauri/Cargo.toml
```

The Tauri SQL plugin initializes `nous.db` under the operating system's application configuration directory for `com.nous.desktop`. Task 001 creates only a technical `app_metadata` table plus the plugin's migration bookkeeping; it does not create Self Model data.

## Repository documents

- `AGENTS.md` — instructions for coding agents.
- `docs/PRODUCT.md` — product definition and user experience.
- `docs/MASTER_PLAN.md` — engineering roadmap from the foundation through Private Beta.
- `docs/TASK_SYSTEM.md` — task creation, scope, review, completion, and checkpoints.
- `docs/BACKLOG.md` — active roadmap, later candidates, post-beta features, and research ideas.
- `docs/DECISIONS.md` — accepted design decisions that agents should not casually reopen.
- `docs/PRINCIPLES.md` — non-negotiable design principles.
- `docs/SELF_MODEL.md` — conceptual model of the person.
- `docs/ARCHITECTURE.md` — technical architecture and security boundary.
- `docs/SAFETY.md` — Care Layer and high-risk interaction behavior.
- `docs/RESPONSE_POLICY.md` — chooses between relief, practical help, reflection, decision support, philosophy, and care.
- `docs/INTERACTION_MODEL.md` — adapts response form to the user's current conversational state.
- `docs/FRAMEWORKS.md` — psychology/philosophy framework library rules.
- `docs/ACADEMIC_FOUNDATION.md` — future framework provenance and humility rules.
- `docs/ONBOARDING.md` — future Bootstrap Self Model onboarding design.
- `docs/VOICE_AND_I18N.md` — tone, bilingual UI, and output style.
- `docs/ROADMAP.md` — staged implementation plan.
- `docs/EXPRESSION_LAB.md` — explicitly post-beta expression-support epic.
- `CODEX_TASK_001.md` — completed historical scaffold task.
- `CODEX_TASK_002.md` — completed historical domain-foundation task.
- `docs/TASK_003_DESIGN.md` — proposed Belief/Value revision architecture; not implementation authorization.

## License

A public open-source license has not yet been selected. Keep the repository private until the owner explicitly decides to publish it.
