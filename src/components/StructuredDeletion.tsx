import { useState } from "react";

import { confirmDeletion } from "../app/deletion";
import type { StructuredHistory } from "../app/history";
import type { Messages } from "../i18n";
import type { CorrectableRecord } from "./StructuredCorrection";

export function StructuredDeletion({ selected, messages, onCancel, onDeleted, onStale }: {
  selected: CorrectableRecord;
  messages: Messages;
  onCancel: () => void;
  onDeleted: (history: StructuredHistory) => void;
  onStale: () => void;
}) {
  const copy = messages.deletion;
  const [deleting, setDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [stale, setStale] = useState(false);

  async function confirm() {
    if (deleting || stale) return;
    setDeleting(true);
    setError(null);
    try {
      const history = await confirmDeletion({
        recordType: selected.kind,
        targetId: selected.record.id,
        expectedStateToken: selected.record.stateToken,
      });
      onDeleted(history);
    } catch (failure) {
      const code = typeof failure === "object" && failure !== null && "code" in failure
        ? String(failure.code)
        : "deleteFailed";
      setError(copy.errors[code as keyof typeof copy.errors] ?? copy.errors.deleteFailed);
      if (code === "staleDelete") setStale(true);
    } finally {
      setDeleting(false);
    }
  }

  return <section className="deletion-confirmation" aria-labelledby="deletion-title">
    <h2 id="deletion-title">{copy.title}</h2>
    <p>{copy.consequence}</p>
    <blockquote>{selected.kind === "situation" ? selected.record.description : selected.record.content}</blockquote>
    {error && <p role="alert" className="capture-error">{error}</p>}
    {stale && <button type="button" className="secondary-button" onClick={onStale}>{copy.reload}</button>}
    <div className="capture-actions">
      <button type="button" className="secondary-button" onClick={onCancel} disabled={deleting}>{copy.cancel}</button>
      <button type="button" className="danger-button" onClick={() => void confirm()} disabled={deleting || stale}>
        {deleting ? copy.deleting : copy.confirm}
      </button>
    </div>
  </section>;
}
