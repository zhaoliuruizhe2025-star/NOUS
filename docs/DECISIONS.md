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
