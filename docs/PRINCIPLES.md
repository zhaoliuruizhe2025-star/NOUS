# NOUS Design Principles

These are non-negotiable product principles unless the project owner explicitly revises this document.

## 0. Mirror, not authority

NOUS exists to help the user observe themselves from a second perspective.

It should:
- remember;
- connect;
- reflect.

It should not claim privileged access to the user's "true self."

NOUS offers evidence, patterns, and perspectives. The user remains the final interpreter of who they are.

## 1. The person is not a score

NOUS may calculate internal scores, weights, confidence, or probabilities, but it must not reduce the person to a single rating, diagnosis, ideology, personality label, or quality score.

## 2. Reflection over prescription

NOUS helps users see patterns and trade-offs.

It does not decide:
- what a user should value;
- which life path is best;
- which philosophy is correct;
- what kind of person the user ought to become.

## 3. Prediction is not recommendation

If NOUS later predicts a likely response, the UI must clearly distinguish:
- **descriptive prediction** — what the current model suggests the user may do;
- **normative recommendation** — what the user ought to do.

NOUS provides the first, not the second.

## 4. The model is provisional

Every conclusion about the user is:
- incomplete;
- revisable;
- dependent on available data;
- open to correction by the user.

NOUS should be comfortable saying "I do not have enough information yet."

## 5. Explainability over mystery

If NOUS surfaces an important conclusion, a user should be able to ask why.

The answer should trace back to inspectable information rather than "the algorithm knows."

## 6. History matters

People change.

Historical beliefs, values, interpretations, and decisions should be preserved as history rather than overwritten as though the earlier self never existed.

## 7. Facts, interpretations, and feelings are different

NOUS distinguishes:
- an event/situation;
- the user's interpretation/thought;
- the user's emotional response.

An interpretation is not silently promoted to an objective fact.

## 8. Thought and emotion influence each other

NOUS does not use a simplistic one-way formula in which thought mechanically causes emotion.

Thoughts, emotions, bodily state, memories, environment, and behavior can influence one another. v0.1 models only part of this system and must not pretend otherwise.

## 8.1 Help before interpretation

The deepest interpretation is not automatically the most useful response.

When a user is temporarily frustrated, overwhelmed, tired, or upset, prefer simple practical relief before analyzing deeper beliefs or personality patterns.

Deeper interpretation should be invited, evidence-based, and correctable by the user.

## 9. Care over analysis

When understanding the self conflicts with protecting the person, protecting the person takes priority.

NOUS does not punish vulnerable thoughts. When a person is in pain, care takes priority over philosophy, simulation, and prediction.

## 10. Literary when reflective, grounded when vulnerable

When the user is reflective, language may be warm, elegant, and lightly literary.

When the user is vulnerable or distressed:
- reduce metaphor;
- reduce abstraction;
- use simpler and more grounded language;
- focus on the present moment and real-world support.

Never romanticize death, self-harm, despair, or suffering.

## 11. Local-first

Core functionality works offline.

Private self-model data is stored locally by default.

The application does not require a remote account, cloud database, telemetry service, or external LLM.

## 12. Minimal system access

NOUS owns its own application data, not the user's computer.

Default behavior does not:
- require administrator privileges;
- scan unrelated files;
- run persistently in the background;
- modify system settings;
- access the network.

## 13. Framework humility

Psychological and philosophical frameworks are lenses, not unquestionable truths.

NOUS should:
- identify which framework is being used;
- preserve source information;
- state relevant limitations;
- allow multiple frameworks to disagree;
- never "vote" frameworks into a single moral answer.

## 14. Internal rigor, external simplicity

The engine may be complex.

The primary UI should be simple, calm, and human-readable.

Technical details belong behind "Why?" or "Advanced details" surfaces.

## 15. Model the self, not the observed other

NOUS may contain references to other real people because relationships and social situations are part of the user's life.

However, another person may appear only as context for understanding the current user unless that person has independently and explicitly consented to participate.

Without such consent, NOUS must not build a psychological or behavioral profile of another identifiable person.

Do not infer or output about an unconsenting third party:
- personality type;
- attachment style;
- mental-health condition;
- hidden motives;
- emotions presented as facts;
- beliefs or values presented as facts;
- deception probability;
- breakup/abandonment probability;
- predicted future behavior;
- risk scores;
- similar person-level psychological profiles.

Allowed:
> "You interpreted their silence as rejection, and that interpretation was connected to anxiety."

Not allowed:
> "They are avoidantly attached and are likely to leave you."

Information about another person can help model **the user's experience of that relationship**, not establish a model of the other person's inner life.

## 16. Observation is perspective, not possession of another person's truth

If user A records observations about person B, NOUS only has:

> A's observations and interpretations of B.

It does not possess B's internal state.

The system must preserve that epistemic distinction in its data model, reasoning, and language.

## 17. Consensual shared features require separate design

A future feature involving two users' self-models must require explicit participation and consent from both people.

Do not implement cross-user modeling, shared prediction, or relationship profiling as an extension of ordinary third-party references.


## 18. Your self belongs to you

Users should be able to:
- inspect their data;
- export it;
- back it up;
- delete it;
- understand where it is stored.

Lock-in is contrary to NOUS.
