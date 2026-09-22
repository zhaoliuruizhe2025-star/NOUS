import { useEffect, useState } from "react";

import type { Messages } from "../i18n";
import {
  isStructuredHistoryEmpty,
  loadStructuredHistory,
  type StructuredHistory as StructuredHistoryData,
  type StructuredHistoryCommandError,
  type StructuredHistoryThought,
  type StructuredHistoryObservation,
} from "../app/history";
import { StructuredCorrection, type CorrectableRecord } from "./StructuredCorrection";
import { StructuredDeletion } from "./StructuredDeletion";

function RecordActions({ selected, messages, onCorrect, onDelete }: {
  selected: CorrectableRecord;
  messages: Messages;
  onCorrect: (record: CorrectableRecord) => void;
  onDelete: (record: CorrectableRecord) => void;
}) {
  const copy = messages.correction;
  return (
    <div className="history-record-actions">
      {selected.record.corrected && <details className="correction-provenance">
        <summary>{copy.corrected}</summary>
        <p>{copy.priorMeaning}</p>
        <ol>{selected.kind === "situation"
          ? selected.record.corrections.map((item) => <li key={item.sequence}>
              <strong>{copy.sequence.replace("{value}", String(item.sequence))}</strong>
              <p>{copy.before}: {item.beforeDescription}</p>
              <p>{copy.after}: {item.afterDescription}</p>
              {item.note && <p>{copy.noteDisplay}: {item.note}</p>}
            </li>)
          : selected.kind === "observation"
            ? selected.record.corrections.map((item) => <li key={item.sequence}>
                <strong>{copy.sequence.replace("{value}", String(item.sequence))}</strong>
                <p>{copy.before}: {item.beforeContent}</p>
                <p>{copy.after}: {item.afterContent}</p>
                <p>{copy.beforeContext}: {item.beforeContext?.description ?? copy.noContext}</p>
                <p>{copy.afterContext}: {item.afterContext?.description ?? copy.noContext}</p>
                {item.note && <p>{copy.noteDisplay}: {item.note}</p>}
              </li>)
            : selected.record.corrections.map((item) => <li key={item.sequence}>
                <strong>{copy.sequence.replace("{value}", String(item.sequence))}</strong>
                <p>{copy.before}: {item.beforeContent}</p>
                <p>{copy.after}: {item.afterContent}</p>
                <p>{copy.beforeContext}: {item.beforeContext?.description ?? copy.noContext}</p>
                <p>{copy.afterContext}: {item.afterContext?.description ?? copy.noContext}</p>
                <p>{copy.beforeConviction}: {item.beforeSubjectiveConviction ?? copy.notReported}</p>
                <p>{copy.afterConviction}: {item.afterSubjectiveConviction ?? copy.notReported}</p>
                {item.note && <p>{copy.noteDisplay}: {item.note}</p>}
              </li>)}</ol>
      </details>}
      <button type="button" className="text-button" onClick={() => onCorrect(selected)}>{copy.open}</button>
      <button type="button" className="text-button deletion-open" onClick={() => onDelete(selected)}>{messages.deletion.open}</button>
    </div>
  );
}

function ObservationItem({ observation, contextId, messages, onCorrect, onDelete }: {
  observation: StructuredHistoryObservation;
  contextId: string | null;
  messages: Messages;
  onCorrect: (record: CorrectableRecord) => void;
  onDelete: (record: CorrectableRecord) => void;
}) {
  return <li><p>{observation.content}</p><RecordActions selected={{ kind: "observation", record: observation, contextId }} messages={messages} onCorrect={onCorrect} onDelete={onDelete} /></li>;
}

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
  contextId,
  messages,
  onCorrect,
  onDelete,
}: {
  thought: StructuredHistoryThought;
  contextId: string | null;
  messages: Messages;
  onCorrect: (record: CorrectableRecord) => void;
  onDelete: (record: CorrectableRecord) => void;
}) {
  return (
    <li>
      <p>{thought.content}</p>
      {thought.subjectiveConviction !== null && (
        <small>
          {messages.history.conviction.replace(
            "{value}",
            String(thought.subjectiveConviction),
          )}
        </small>
      )}
      <RecordActions selected={{ kind: "thought", record: thought, contextId }} messages={messages} onCorrect={onCorrect} onDelete={onDelete} />
    </li>
  );
}

export function StructuredHistory({ messages }: { messages: Messages }) {
  const copy = messages.history;
  const [state, setState] = useState<HistoryState>({ status: "loading" });
  const [loadAttempt, setLoadAttempt] = useState(0);
  const [selected, setSelected] = useState<CorrectableRecord | null>(null);
  const [deleteSelected, setDeleteSelected] = useState<CorrectableRecord | null>(null);

  function reload() {
    setSelected(null);
    setDeleteSelected(null);
    setState({ status: "loading" });
    setLoadAttempt((attempt) => attempt + 1);
  }

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
            onClick={reload}
          >
            {copy.retry}
          </button>
        </div>
      )}

      {state.status === "loaded" && selected && !deleteSelected && (
        <StructuredCorrection
          key={`${selected.kind}:${selected.record.id}:${selected.record.stateToken}`}
          selected={selected}
          contexts={state.history.contexts}
          messages={messages}
          onCancel={() => setSelected(null)}
          onStale={reload}
          onSaved={(history) => { setState({ status: "loaded", history }); setSelected(null); }}
        />
      )}

      {state.status === "loaded" && deleteSelected && !selected && (
        <StructuredDeletion
          key={`${deleteSelected.kind}:${deleteSelected.record.id}:${deleteSelected.record.stateToken}`}
          selected={deleteSelected}
          messages={messages}
          onCancel={() => setDeleteSelected(null)}
          onStale={reload}
          onDeleted={(history) => { setState({ status: "loaded", history }); setDeleteSelected(null); }}
        />
      )}

      {state.status === "loaded" && !selected && !deleteSelected && isStructuredHistoryEmpty(state.history) && (
        <p className="history-empty">{copy.empty}</p>
      )}

      {state.status === "loaded" && !selected && !deleteSelected && !isStructuredHistoryEmpty(state.history) && (
        <div className="history-records">
          {state.history.contexts.length > 0 && (
            <section aria-labelledby="history-contexts-title">
              <h2 id="history-contexts-title">{copy.contexts}</h2>
              <div className="history-context-list">
                {state.history.contexts.map((context) => (
                  <article className="history-context" key={context.id}>
                    <h3>{context.description}</h3>
                    <RecordActions selected={{ kind: "situation", record: context }} messages={messages} onCorrect={setSelected} onDelete={setDeleteSelected} />
                    {context.observations.length > 0 && (
                      <section>
                        <h4>{copy.observations}</h4>
                        <ul>
                          {context.observations.map((observation) => (
                            <ObservationItem key={observation.id} observation={observation} contextId={context.id} messages={messages} onCorrect={setSelected} onDelete={setDeleteSelected} />
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
                              contextId={context.id}
                              messages={messages}
                              onCorrect={setSelected}
                              onDelete={setDeleteSelected}
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
                  <ObservationItem key={observation.id} observation={observation} contextId={null} messages={messages} onCorrect={setSelected} onDelete={setDeleteSelected} />
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
                    contextId={null}
                    messages={messages}
                    onCorrect={setSelected}
                    onDelete={setDeleteSelected}
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
