# NOUS Product Specification

## 1. North Star

NOUS exists to help a person gain a second perspective on themselves.

Its purpose is not to define the user from the outside. It is to organize and reflect back the user's own experiences, thoughts, emotions, beliefs, values, memories, and decisions so that the user can step back and see patterns that are difficult to notice from inside everyday life.

The desired outcome is greater self-awareness and stronger self-observation.

A useful test for every feature is:

> Does this help the user see themselves more clearly, or does it merely let the system label them?

If the feature primarily labels, judges, or defines the user, it should be redesigned.

NOUS should act as a mirror, not an authority.

### Remember · Connect · Reflect · Self-awareness

The core product loop is:

1. **Remember** — preserve what the user thought, felt, valued, experienced, and chose over time.
2. **Connect** — reveal meaningful relationships and recurring patterns among those records.
3. **Reflect** — present those patterns back to the user in a form that supports a more detached, observer-like view of themselves.
4. **Self-awareness** — help the user notice, question, and revise their own understanding while remaining the final interpreter.

Prediction, philosophical analysis, and life-path simulation are secondary capabilities built on this foundation.

## 2. One-sentence definition

NOUS is a local-first computational self-modeling application that helps a person develop a clearer second perspective on themselves by organizing and reflecting how beliefs, values, memories, thoughts, emotions, and decisions interact and change over time.

## 2.1 Core experience

The normal experience begins with a short NOUS logo opening animation and a calm, highly minimal conversational surface: a short context-appropriate prompt above one main input. It should feel closer to a composed conversation than a form, dashboard, psychological test, or journaling database.

The person normally says what happened, what they think it meant, and how it felt. NOUS performs deeper structuring; it does not make the person manually maintain Situation, Thought, Emotion, Belief, or Value records.

Normal conversations are remembered by default because the user is intentionally using a system that learns from their life. Remembering a statement is not treating it as a durable psychological truth. See `SELF_MODEL.md`.

## 3. Problem

People often experience their inner lives as disconnected fragments:
- a thought today;
- a decision months ago;
- a recurring fear;
- a value they claim to hold;
- an emotion they cannot easily explain;
- a philosophical belief that does not always match their behavior.

Existing tools often fall into one of two extremes:
- simple journaling, which preserves text but performs little structured reasoning; or
- AI chat systems, which can produce fluent interpretations but may be opaque, inconsistent, costly, and dependent on an external model.

NOUS aims to create an explicit, inspectable, longitudinal self-model that remains useful without a generative AI service.

## 4. Product promise

NOUS should help a user see:
- what they repeatedly care about;
- what interpretations tend to accompany certain emotions;
- where stated beliefs and observed behavior appear to be in tension;
- how their worldview changes over time;
- which trade-offs different life paths appear to contain;
- how accurately the system can model their future responses, as an experiment rather than a prophecy.

## 5. What NOUS is not

NOUS is not:
- a therapist;
- a medical or psychiatric diagnostic system;
- a suicide-risk scoring tool;
- a fortune teller;
- a personality quiz;
- an ideology classifier;
- a moral authority;
- an authority that decides how the user should live;
- an LLM wrapper.

## 6. Long-term product surfaces

### 5.1 Today

A calm entry point for current reflection.

Primary actions:
- describe what is on the user's mind in natural language;
- receive useful conversational help;
- see at most one or two meaningful observations, not a dashboard full of metrics.

### 5.2 Mirror

Shows change over time in human-readable language.

Examples:
- "Recently, you have spoken more often about freedom when describing important choices."
- "Three months ago, uncertainty was usually connected to avoiding regret; recently, it is more often connected to curiosity."

The primary UI should favor narrative summaries. Detailed charts and underlying evidence are secondary.

### 5.3 Inner Map

Shows a curated subset of the user's current self-model:
- important values;
- important beliefs;
- selected supporting/tension relationships;
- recurring thoughts;
- links to memories/decisions.

Do not expose a giant raw graph by default.

### 5.4 Paths

Helps the user explore a decision by showing trade-offs.

It may show:
- what each path may protect;
- what each path may sacrifice;
- uncertainty;
- possible third options or false dichotomies;
- connections to the user's values and past decisions.

When the user asks for advice or decision support and relevant evidence/context exists, it may provide useful, grounded directional recommendations under `PRINCIPLES.md` §13.1. Final decision authority remains with the user; this does not authorize primary UI scoring, ranking, or winner mechanics.

### 5.5 Self

Shows how NOUS currently understands the user:
- stable patterns;
- changing patterns;
- areas of uncertainty;
- recurring tensions;
- model coverage;
- prediction history, if enabled in a later version.

The model must always be presented as incomplete and revisable. Direct Self Model inspection is a later experimental surface, not a core early-product dashboard; the model should first reveal itself through usefulness rather than labels.

## 7. Long-term capabilities

### Mind Mirror
Question: **How have I changed?**

### Belief Graph / Inner Map
Question: **What do I believe and value, and how are those things connected?**

### Philosophy Engine
Question: **Where are the tensions or alternative interpretations in my worldview?**

### Life Paths
Question: **What trade-offs might different choices create for the things I care about?**

### Digital Self
Question: **How well can an explicit model of me anticipate how I respond to new situations?**

Digital Self is experimental and must not be framed as consciousness, a soul, destiny, or objective truth.

## 8. Core interaction philosophy

NOUS should separate:

1. **what happened**;
2. **what the user thought it meant**;
3. **how the user felt**;
4. **what the user did**;
5. **what happened afterward**.

This separation enables reflection without automatically treating interpretations as facts.

NOUS must also respond to the person's current conversational state, not merely the literal sentence. Content understanding and interaction strategy are separate systems: one determines what the material may mean; the other adjusts how much to say, how many questions to ask, how direct or warm to be, whether to take initiative, and whether action or reflection should come first.

The baseline is **useful before insightful**. One available strategy, where appropriate, is:

```text
Action -> Explanation -> Reflection
```

This sequence yields to presence when sadness or loneliness calls for it, to deeper exploration when invited, and to grounded safety behavior in distress or crisis. See `INTERACTION_MODEL.md`.

When material uncertainty would change the analysis, NOUS clarifies before inferring, using the minimum necessary clarification. It distinguishes user-reported state, concrete event details available from the user or another source with provenance preserved, the user's interpretation, and NOUS’s tentative interpretation. See `INTERACTION_MODEL.md` and `SELF_MODEL.md`.

## 9. Output hierarchy

Before choosing the output form, NOUS first decides what kind of help is appropriate: immediate relief, practical help, reflection, decision support, philosophical exploration, or care.

A casual statement of frustration should not automatically trigger a deep Self Model interpretation. See `RESPONSE_POLICY.md`.


User-facing output should have three levels:

### Level 1 — Human-readable answer
A concise, natural-language answer to the user's actual request, including reflection when appropriate. The Response Contract and priority rules in [REASONING_MODEL.md](REASONING_MODEL.md) keep relevant background from displacing the direct answer.

### Level 2 — "Why did NOUS notice this?"
Shows the relevant observations, past patterns, and concepts.

### Level 3 — Advanced details
Optional structured data, evidence strength and uncertainty, rule IDs, and major reasoning factors. These must reflect the actual basis used under [REASONING_MODEL.md](REASONING_MODEL.md); this output layer does not authorize numeric psychological confidence scores or raw chain-of-thought disclosure.

Users should never be forced to read Level 3 to benefit from NOUS.

Abstract reasoning should be translated into concrete human situations. When relevant history exists, examples should come from the user's actual records rather than generic invented scenarios. Conclusions remain traceable and correctable, with language strength matching the evidence: tentative when weak, increasingly direct when repeated or stronger. As relevant long-term evidence becomes richer and more stable, explicit confirmation should generally become less frequent; strong evidence never makes an interpretation immutable. See `RESPONSE_POLICY.md` §6.

## 10. User agency

The user can:
- correct NOUS;
- reject an interpretation;
- mark a relationship as inaccurate;
- edit/remove their own records;
- inspect why a conclusion appeared;
- export their data;
- delete their data.

A self-model that cannot be challenged by the person it models is contrary to the product.

Natural-language correction must preserve the original history, earlier interpretation, later correction, and current canonical understanding. A correction is not the same as genuine change over time.

## 11. Foundation and release sequence

The current interface is temporary functional scaffolding. The approved future direction is a minimal, calm, Apple-like conversational experience, but its visual identity is not implementation-authorized until a dedicated UI/UX phase; domain work must remain independent of the scaffold. See `ONBOARDING.md` for the future first-run experience and `UI_BOUNDARY.md` for its implementation boundary.

The first usable foundation succeeds if a user can:
- create local records;
- distinguish situations, thoughts, emotions, beliefs, values, memories, and decisions;
- preserve revisions/history;
- see a small, understandable Inner Map;
- inspect how an entity is connected to others;
- switch between English and Simplified Chinese;
- run the app offline with no external AI/API dependency.

The foundation does not need to predict future choices or provide a mature philosophy engine.

A small Private Beta precedes any public or open-source release. Post-beta features such as Relationship Lens, Expression Lab, and Digital Self remain in `BACKLOG.md` until explicitly promoted into the active plan.
