import { useState } from "react";

import { createDatabaseBackup, exportUserData } from "../app/data_portability";
import type { Messages } from "../i18n";

type Action = "export" | "backup";
type State = { kind: "idle" } | { kind: "working"; action: Action }
  | { kind: "completed"; action: Action } | { kind: "error"; code: string };

export function DataPortability({ messages }: { messages: Messages }) {
  const [state, setState] = useState<State>({ kind: "idle" });
  const copy = messages.portability;

  async function run(action: Action) {
    setState({ kind: "working", action });
    try {
      const result = action === "export" ? await exportUserData() : await createDatabaseBackup();
      setState(result.status === "cancelled" ? { kind: "idle" } : { kind: "completed", action });
    } catch (error) {
      const code = typeof error === "object" && error !== null && "code" in error
        && typeof error.code === "string" ? error.code : `${action}Failed`;
      setState({ kind: "error", code });
    }
  }

  const errorCopy = copy.errors as Record<string, string>;
  return <section className="portability-card" aria-labelledby="portability-title">
    <p className="eyebrow">{copy.eyebrow}</p>
    <h1 id="portability-title">{copy.title}</h1>
    <div className="portability-actions">
      <div>
        <h2>{copy.exportAction}</h2>
        <p>{copy.exportExplanation}</p>
        <button type="button" disabled={state.kind === "working"} onClick={() => void run("export")}>
          {copy.exportAction}
        </button>
      </div>
      <div>
        <h2>{copy.backupAction}</h2>
        <p>{copy.backupExplanation}</p>
        <button type="button" disabled={state.kind === "working"} onClick={() => void run("backup")}>
          {copy.backupAction}
        </button>
      </div>
    </div>
    <p className="portability-privacy">{copy.privacy}</p>
    <p role="status">
      {state.kind === "working" && (state.action === "export" ? copy.workingExport : copy.workingBackup)}
      {state.kind === "completed" && (state.action === "export" ? copy.exportComplete : copy.backupComplete)}
      {state.kind === "error" && (errorCopy[state.code] ?? copy.errors.exportFailed)}
    </p>
  </section>;
}
