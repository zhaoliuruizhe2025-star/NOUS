# NOUS Technical Architecture

## 1. Architectural goals

NOUS should be:
- local-first;
- offline-capable;
- inspectable;
- cross-platform;
- privacy-preserving;
- testable;
- modular enough to evolve from a structured reflection tool into a reasoning/simulation system.

## 2. Initial stack

Current preferred stack:

- **Desktop shell:** Tauri 2
- **UI:** React + TypeScript
- **Core/native layer:** Rust
- **Storage:** SQLite
- **Package manager:** npm unless the repository explicitly changes this
- **Development target:** Windows 11 first
- **Future supported targets:** macOS and Linux

This is an initial architecture, not permission for an agent to prematurely move all domain logic into Rust.

The TypeScript/Rust boundary should be decided incrementally based on:
- performance needs;
- data ownership;
- testability;
- code clarity;
- cross-platform behavior.

## 3. Layering

Preferred conceptual layers:

```text
┌─────────────────────────────┐
│ Presentation / Voice        │
│ React + i18n                │
├─────────────────────────────┤
│ Application Services        │
│ workflows / orchestration   │
├─────────────────────────────┤
│ Domain / Self Model         │
│ entities + domain rules     │
├─────────────────────────────┤
│ Reasoning Engine            │
│ graph / rule / later sim    │
├─────────────────────────────┤
│ Persistence                 │
│ SQLite + migrations         │
├─────────────────────────────┤
│ Tauri / OS Boundary         │
└─────────────────────────────┘
```

Care/safety is cross-cutting and can block higher-level reasoning/simulation behavior.

### 3.1 Content and interaction separation

User-response behavior has two distinct responsibilities:

```text
Input + relevant history
        |              |
        v              v
Content understanding  Interaction state signals
        |              |
        v              v
Semantic result        Interaction strategy
        |              |
        +-------> Response composition
```

- **Content understanding** identifies situations, observations, thoughts, emotions, relevant history, and possible meaning.
- **Interaction strategy** selects response mode and adjusts verbosity, questions, warmth, directness, initiative, solution priority, and reflection depth.

They should exchange explicit data and remain independently testable. Interaction state is temporary and must not automatically mutate the durable Self Model. The Care Layer can override ordinary strategy and block higher-risk output.

## 4. Repository direction

The initial scaffold may resemble:

```text
NOUS/
├── AGENTS.md
├── README.md
├── CODEX_TASK_001.md
├── docs/
├── src/
│   ├── app/
│   ├── components/
│   ├── domain/
│   ├── features/
│   ├── i18n/
│   └── styles/
├── src-tauri/
│   ├── src/
│   └── migrations/
├── tests/
└── ...
```

Do not force this exact tree if Tauri's generated project structure requires minor differences. Preserve the conceptual separation.

The Task 001 UI is temporary scaffolding. Final visual identity, navigation, typography, color, animation, and composition remain frozen until a dedicated UI/UX phase. Domain and application logic must not depend on temporary visual components.

## 5. Database

Use SQLite for local persistence.

Requirements:
- versioned schema migrations;
- transactions for multi-step updates;
- foreign-key integrity;
- historical revision tables or equivalent append-only revision semantics;
- no silent destructive schema migration;
- local application data directory by default.

Do not place the primary database in an arbitrary working-directory path where accidental deletion is likely.

## 6. Network behavior

v0.1 should not require network access at runtime.

Do not add:
- analytics;
- telemetry;
- cloud sync;
- remote crash reporting;
- remote fonts;
- remote AI calls;
- automatic external content fetches.

If development tooling uses the network to install dependencies, that is separate from runtime behavior.

## 7. OS permissions and isolation

Runtime NOUS should:
- run without administrator/root privileges;
- use only its own application data by default;
- avoid background services;
- avoid auto-start;
- avoid arbitrary shell execution;
- access external files only through explicit user selection;
- not inspect unrelated processes/files.

## 8. Performance

Normal interaction should use incremental computation.

Avoid:
- rescanning the full history on every keystroke;
- constant background analysis;
- unbounded graph traversal;
- algorithms that can loop indefinitely on graph cycles.

Expensive future analyses should be explicitly triggered or efficiently cached.

## 9. Core reasoning requirements

Graph/rule logic should be:
- deterministic where practical;
- unit-tested;
- cycle-safe;
- explainable;
- versioned when rule behavior changes.

A reasoning result should be able to return both:
- a semantic result;
- a structured reasoning trace suitable for "Why?" UI.

## 10. Backup and recovery

Before v1.0, NOUS should support:
- manual export;
- import;
- local backups;
- recovery from interrupted writes where practical.

The design should not make backup impossible.

## 11. Cross-platform policy

### Tier 1: Windows 11
Primary development and daily testing platform.

### Tier 2: macOS
Supported after Windows alpha stabilizes.

### Tier 3: Linux
Supported after Windows alpha stabilizes.

Avoid Windows-only domain code. OS-specific integration should live behind a narrow boundary.

## 12. Development prerequisites

For the preferred stack, development on Windows is expected to require:
- Git;
- Node.js LTS and npm;
- Rust stable with the MSVC toolchain;
- Microsoft C++ Build Tools;
- WebView2 runtime (normally already present on modern Windows);
- editor/IDE of choice.

Agents must not silently install or change these system-level prerequisites.

## 13. First architecture milestone

The first implementation milestone is intentionally small:

1. project builds;
2. Tauri desktop window launches;
3. React/TypeScript UI renders;
4. English/Chinese localization skeleton works;
5. a local SQLite connection can be initialized safely;
6. no external network service is required;
7. no Self Engine logic is implemented yet.

After that milestone is reviewed, domain implementation begins.

Status: **COMPLETE** at checkpoint `455e929 Complete NOUS Task 001 desktop scaffold`.

The next planned implementation task is Task 002, limited to the first six domain entities, validation, and focused tests. The planning roadmap does not itself authorize later layers, migrations, or UI changes.
