import { useEffect, useState } from "react";

import type { Messages } from "../i18n";
import {
  isStructuredHistoryEmpty,
  loadStructuredHistory,
  type StructuredHistory as StructuredHistoryData,
  type StructuredHistoryCommandError,
  type StructuredHistoryThought,
} from "../app/history";

type HistoryState =
  | { status: "loading" }
  | { status: "loaded"; history: StructuredHistoryData }
  | { status: "error"; code: string };

function errorCode(error: unknown): string {
  if (typeof error === "object" && error !== null && "code" in error) {
    return String((error as StructuredHistoryCommandError).code);
  }
  return "loadFailed";
}

function ThoughtItem({
  thought,
  convictionCopy,
}: {
  thought: StructuredHistoryThought;
  convictionCopy: string;
}) {
  return (
    <li>
      <p>{thought.content}</p>
      {thought.subjectiveConviction !== null && (
        <small>
          {convictionCopy.replace(
            "{value}",
            String(thought.subjectiveConviction),
          )}
        </small>
      )}
    </li>
  );
}

export function StructuredHistory({ messages }: { messages: Messages }) {
  const copy = messages.history;
  const [state, setState] = useState<HistoryState>({ status: "loading" });
  const [loadAttempt, setLoadAttempt] = useState(0);

  useEffect(() => {
    let active = true;
    loadStructuredHistory()
      .then((history) => {
        if (active) setState({ status: "loaded", history });
      })
      .catch((error: unknown) => {
        if (active) setState({ status: "error", code: errorCode(error) });
      });
    return () => {
      active = false;
    };
  }, [loadAttempt]);

  const errorMessage =
    state.status === "error"
      ? copy.errors[state.code as keyof typeof copy.errors] ?? copy.errors.loadFailed
      : null;

  return (
    <section className="history-card" aria-labelledby="history-title">
      <p className="eyebrow">{copy.eyebrow}</p>
      <h1 id="history-title">{copy.title}</h1>
      <p className="history-scope">{copy.scope}</p>
      <p className="history-order">{copy.order}</p>

      {state.status === "loading" && <p role="status">{copy.loading}</p>}

      {state.status === "error" && (
        <div className="history-error" role="alert">
          <p>{errorMessage}</p>
          <button
            className="secondary-button"
            type="button"
            onClick={() => {
              setState({ status: "loading" });
              setLoadAttempt((attempt) => attempt + 1);
            }}
          >
            {copy.retry}
          </button>
        </div>
      )}

      {state.status === "loaded" && isStructuredHistoryEmpty(state.history) && (
        <p className="history-empty">{copy.empty}</p>
      )}

      {state.status === "loaded" && !isStructuredHistoryEmpty(state.history) && (
        <div className="history-records">
          {state.history.contexts.length > 0 && (
            <section aria-labelledby="history-contexts-title">
              <h2 id="history-contexts-title">{copy.contexts}</h2>
              <div className="history-context-list">
                {state.history.contexts.map((context) => (
                  <article className="history-context" key={context.id}>
                    <h3>{context.description}</h3>
                    {context.observations.length > 0 && (
                      <section>
                        <h4>{copy.observations}</h4>
                        <ul>
                          {context.observations.map((observation) => (
                            <li key={observation.id}>{observation.content}</li>
                          ))}
                        </ul>
                      </section>
                    )}
                    {context.thoughts.length > 0 && (
                      <section>
                        <h4>{copy.thoughts}</h4>
                        <ul>
                          {context.thoughts.map((thought) => (
                            <ThoughtItem
                              key={thought.id}
                              thought={thought}
                              convictionCopy={copy.conviction}
                            />
                          ))}
                        </ul>
                      </section>
                    )}
                  </article>
                ))}
              </div>
            </section>
          )}

          {state.history.standaloneObservations.length > 0 && (
            <section>
              <h2>{copy.standaloneObservations}</h2>
              <ul>
                {state.history.standaloneObservations.map((observation) => (
                  <li key={observation.id}>{observation.content}</li>
                ))}
              </ul>
            </section>
          )}

          {state.history.standaloneThoughts.length > 0 && (
            <section>
              <h2>{copy.standaloneThoughts}</h2>
              <ul>
                {state.history.standaloneThoughts.map((thought) => (
                  <ThoughtItem
                    key={thought.id}
                    thought={thought}
                    convictionCopy={copy.conviction}
                  />
                ))}
              </ul>
            </section>
          )}
        </div>
      )}
    </section>
  );
}
