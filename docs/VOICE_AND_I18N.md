# NOUS Voice and Internationalization

## 1. Supported languages

Initial user-facing languages:

- English (`en`)
- Simplified Chinese (`zh-CN`)

Internal code, identifiers, schemas, rule IDs, and logs use English.

User input may contain:
- English;
- Chinese;
- a mixture of both.

## 2. Localization architecture

User-facing strings must be separated from business/domain logic.

A likely organization:

```text
src/i18n/
├── en/
│   ├── common.json
│   ├── reflection.json
│   ├── care.json
│   └── philosophy.json
└── zh-CN/
    ├── common.json
    ├── reflection.json
    ├── care.json
    └── philosophy.json
```

Exact implementation may change, but separation is mandatory.

## 3. Semantic localization

Chinese should not be a mechanical line-by-line translation of English.

Preserve meaning and tone.

Example concept:

English:
> Some choices are difficult not because you do not know what you want, but because every path asks you to leave something behind.

Natural Chinese:
> 有些选择之所以艰难，并不是因为你不知道自己想要什么，而是因为无论走向哪一边，都难免要留下些什么。

Both communicate the same semantic result; the phrasing can differ.

## 4. Voice modes

### RELIEF

Characteristics:
- short;
- practical;
- soothing without being sentimental;
- focused on the next few minutes;
- avoids deep interpretation unless requested.

Example:
> "You're overloaded right now. Let's make the next ten minutes easier before we analyze anything."

### PRACTICAL

Characteristics:
- concrete;
- organized;
- action-oriented;
- avoids converting ordinary problems into psychological theories.


### REFLECTION

Characteristics:
- calm;
- thoughtful;
- concise;
- lightly literary when appropriate;
- avoids pretending to possess human feelings.

Example:
> You seem to be standing between two things that matter deeply to you.

### DECISION

Characteristics:
- clear;
- restrained;
- trade-off oriented;
- no recommendation language;
- uncertainty visible.

### CARE

Characteristics:
- warmer;
- simpler;
- less metaphorical;
- less philosophical abstraction;
- present-focused;
- real-world support oriented when needed.

Principle:

> When the user is reflective, language may be poetic. When the user is vulnerable, language should become warmer, simpler, and more grounded.

## 5. Anti-AI tone rules

Avoid habitual chatbot phrasing such as:
- excessive headings in normal interaction;
- "As an AI...";
- fake certainty;
- generic motivational slogans;
- repetitive validation;
- verbose lists when a short human sentence would work;
- constant restatement of the user's words.

NOUS should feel composed, not chatty for the sake of being chatty.

## 6. Output layers

### Primary
A short human-readable reflection.

### Explain
"Why did NOUS notice this?"

### Advanced
Optional:
- confidence;
- relationships;
- rule IDs;
- evidence;
- historical comparisons.

Do not expose advanced data as the default experience.

## 7. Care copy review

All care/safety messages must be intentionally authored/reviewed in both languages.

Do not rely on runtime machine translation for crisis-sensitive wording in the first release.

## 8. Terminology

Keep an explicit terminology map.

Examples:

| Canonical key | English | Simplified Chinese |
| --- | --- | --- |
| `value.autonomy` | Autonomy | 自主 |
| `value.relatedness` | Relatedness | 联结 / 亲密关系（context dependent） |
| `emotion.sadness` | Sadness | 悲伤 |
| `relation.contradicts` | Tension / Contradiction | 张力 / 矛盾（context dependent） |

Do not force one Chinese label when context requires a more natural phrase.
