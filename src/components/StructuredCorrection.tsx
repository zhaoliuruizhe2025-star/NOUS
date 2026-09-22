import { useState } from "react";

import { confirmCorrection, type CorrectionRequest } from "../app/correction";
import type {
  StructuredHistory,
  StructuredHistoryContext,
  StructuredHistoryObservation,
  StructuredHistoryThought,
} from "../app/history";
import type { Messages } from "../i18n";

export type CorrectableRecord =
  | { kind: "situation"; record: StructuredHistoryContext }
  | { kind: "observation"; record: StructuredHistoryObservation; contextId: string | null }
  | { kind: "thought"; record: StructuredHistoryThought; contextId: string | null };

export function StructuredCorrection({
  selected,
  contexts,
  messages,
  onCancel,
  onSaved,
  onStale,
}: {
  selected: CorrectableRecord;
  contexts: StructuredHistoryContext[];
  messages: Messages;
  onCancel: () => void;
  onSaved: (history: StructuredHistory) => void;
  onStale: () => void;
}) {
  const copy = messages.correction;
  const [text, setText] = useState(
    selected.kind === "situation" ? selected.record.description : selected.record.content,
  );
  const [contextId, setContextId] = useState(
    selected.kind === "situation" ? "" : selected.contextId ?? "",
  );
  const [conviction, setConviction] = useState(
    selected.kind === "thought" && selected.record.subjectiveConviction !== null
      ? String(selected.record.subjectiveConviction)
      : "",
  );
  const [note, setNote] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [stale, setStale] = useState(false);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (saving || stale) return;
    const convictionNumber = conviction.trim() === "" ? null : Number(conviction);
    if (
      text.trim() === "" ||
      (selected.kind === "thought" && convictionNumber !== null &&
        (!Number.isInteger(convictionNumber) || convictionNumber < 0 || convictionNumber > 100))
    ) {
      setError(copy.errors.invalidCorrection);
      return;
    }
    const common = {
      targetId: selected.record.id,
      expectedStateToken: selected.record.stateToken,
      note: note.trim() === "" ? null : note,
    };
    let request: CorrectionRequest;
    if (selected.kind === "situation") {
      request = { recordType: "situation", ...common, description: text };
    } else if (selected.kind === "observation") {
      request = {
        recordType: "observation", ...common, content: text,
        context: contextId ? { kind: "existing", situationId: contextId } : { kind: "none" },
      };
    } else {
      request = {
        recordType: "thought", ...common, content: text,
        context: contextId ? { kind: "existing", situationId: contextId } : { kind: "none" },
        subjectiveConviction: convictionNumber === null
          ? { kind: "notReported" }
          : { kind: "reported", value: convictionNumber },
      };
    }
    setSaving(true);
    setError(null);
    try {
      const history = await confirmCorrection(request);
      onSaved(history);
    } catch (failure) {
      const code = typeof failure === "object" && failure !== null && "code" in failure
        ? String(failure.code)
        : "saveFailed";
      setError(copy.errors[code as keyof typeof copy.errors] ?? copy.errors.saveFailed);
      if (code === "staleCorrection") setStale(true);
    } finally {
      setSaving(false);
    }
  }

  return (
    <form className="correction-form" onSubmit={(event) => void submit(event)}>
      <h2>{copy.title}</h2>
      <p>{copy.explanation}</p>
      <label htmlFor="correction-content">{selected.kind === "situation" ? copy.contextText : copy.recordText}</label>
      <textarea id="correction-content" value={text} onChange={(event) => setText(event.target.value)} rows={4} />
      {selected.kind !== "situation" && (
        <>
          <label htmlFor="correction-context">{copy.context}</label>
          <select id="correction-context" value={contextId} onChange={(event) => setContextId(event.target.value)}>
            <option value="">{copy.noContext}</option>
            {contexts.map((context) => <option key={context.id} value={context.id}>{context.description}</option>)}
          </select>
        </>
      )}
      {selected.kind === "thought" && (
        <>
          <label htmlFor="correction-conviction">{copy.conviction}</label>
          <input id="correction-conviction" type="number" min="0" max="100" value={conviction} onChange={(event) => setConviction(event.target.value)} />
        </>
      )}
      <label htmlFor="correction-note">{copy.note}</label>
      <textarea id="correction-note" value={note} onChange={(event) => setNote(event.target.value)} rows={2} />
      {error && <p role="alert" className="capture-error">{error}</p>}
      {stale && <button type="button" className="secondary-button" onClick={onStale}>{copy.reload}</button>}
      <div className="capture-actions">
        <button type="button" className="secondary-button" onClick={onCancel} disabled={saving}>{copy.cancel}</button>
        <button type="submit" className="primary-button" disabled={saving || stale}>{saving ? copy.saving : copy.confirm}</button>
      </div>
    </form>
  );
}
