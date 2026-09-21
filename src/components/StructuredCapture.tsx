import { useState } from "react";

import type { Messages } from "../i18n";
import {
  buildSaveRequest,
  hasBlankAddedItem,
  hasSubstantiveCapture,
  saveStructuredCapture,
  type CaptureDraft,
  type CaptureDraftItem,
  type SavedStructuredCapture,
  type StructuredCaptureCommandError,
} from "../app/capture";

type CapturePhase = "compose" | "review" | "saving" | "saved";

const emptyDraft = (): CaptureDraft => ({
  context: "",
  observations: [],
  thoughts: [],
});

function newDraftItem(): CaptureDraftItem {
  return { key: crypto.randomUUID(), text: "" };
}

function errorCode(error: unknown): string {
  if (typeof error === "object" && error !== null && "code" in error) {
    return String((error as StructuredCaptureCommandError).code);
  }
  return "saveFailed";
}

export function StructuredCapture({ messages }: { messages: Messages }) {
  const copy = messages.capture;
  const [phase, setPhase] = useState<CapturePhase>("compose");
  const [rawInput, setRawInput] = useState("");
  const [draft, setDraft] = useState<CaptureDraft>(emptyDraft);
  const [saved, setSaved] = useState<SavedStructuredCapture | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reset = () => {
    setRawInput("");
    setDraft(emptyDraft());
    setSaved(null);
    setError(null);
    setPhase("compose");
  };

  const updateItem = (
    kind: "observations" | "thoughts",
    key: string,
    text: string,
  ) => {
    setDraft((current) => ({
      ...current,
      [kind]: current[kind].map((item) =>
        item.key === key ? { ...item, text } : item,
      ),
    }));
  };

  const removeItem = (kind: "observations" | "thoughts", key: string) => {
    setDraft((current) => ({
      ...current,
      [kind]: current[kind].filter((item) => item.key !== key),
    }));
  };

  const addItem = (kind: "observations" | "thoughts") => {
    setDraft((current) => ({
      ...current,
      [kind]: [...current[kind], newDraftItem()],
    }));
  };

  const handleSave = async () => {
    if (!hasSubstantiveCapture(draft) || hasBlankAddedItem(draft)) return;

    setError(null);
    setPhase("saving");
    try {
      const result = await saveStructuredCapture(buildSaveRequest(draft));
      setSaved(result);
      setRawInput("");
      setDraft(emptyDraft());
      setPhase("saved");
    } catch (caught) {
      setError(errorCode(caught));
      setPhase("review");
    }
  };

  if (phase === "compose") {
    return (
      <section className="capture-card" aria-labelledby="capture-title">
        <p className="eyebrow">{copy.eyebrow}</p>
        <h1 id="capture-title">{copy.composeTitle}</h1>
        <p className="capture-intro">{copy.composeBody}</p>
        <label className="field-label" htmlFor="natural-input">
          {copy.naturalInputLabel}
        </label>
        <textarea
          id="natural-input"
          value={rawInput}
          onChange={(event) => setRawInput(event.target.value)}
          placeholder={copy.naturalInputPlaceholder}
          rows={7}
        />
        <button
          className="primary-button"
          type="button"
          disabled={rawInput.trim().length === 0}
          onClick={() => setPhase("review")}
        >
          {copy.enterReview}
        </button>
        <p className="privacy-note">{copy.temporaryNote}</p>
      </section>
    );
  }

  if (phase === "saved" && saved) {
    return (
      <section className="capture-card" aria-labelledby="saved-title">
        <p className="eyebrow">{copy.savedEyebrow}</p>
        <h1 id="saved-title">{copy.savedTitle}</h1>
        <div className="saved-records">
          {saved.situation && (
            <section>
              <h2>{copy.savedContext}</h2>
              <p>{saved.situation.description}</p>
            </section>
          )}
          {saved.observations.length > 0 && (
            <section>
              <h2>{copy.savedObservations}</h2>
              <ul>{saved.observations.map((item) => <li key={item.id}>{item.content}</li>)}</ul>
            </section>
          )}
          {saved.thoughts.length > 0 && (
            <section>
              <h2>{copy.savedThoughts}</h2>
              <ul>{saved.thoughts.map((item) => <li key={item.id}>{item.content}</li>)}</ul>
            </section>
          )}
        </div>
        <button className="primary-button" type="button" onClick={reset}>
          {copy.captureAnother}
        </button>
      </section>
    );
  }

  const canSave = hasSubstantiveCapture(draft) && !hasBlankAddedItem(draft);
  const saving = phase === "saving";
  const errorMessage = error
    ? copy.errors[error as keyof typeof copy.errors] ?? copy.errors.saveFailed
    : null;

  return (
    <section className="capture-card" aria-labelledby="review-title">
      <p className="eyebrow">{copy.reviewEyebrow}</p>
      <h1 id="review-title">{copy.reviewTitle}</h1>
      <div className="original-text">
        <span>{copy.originalText}</span>
        <p>{rawInput}</p>
        <small>{copy.originalTextNote}</small>
      </div>

      <div className="capture-fields">
        <label className="field-label" htmlFor="capture-context">
          {copy.contextPrompt}
        </label>
        <textarea
          id="capture-context"
          value={draft.context}
          onChange={(event) =>
            setDraft((current) => ({ ...current, context: event.target.value }))
          }
          rows={3}
        />
        {draft.context.length > 0 && (
          <button
            className="text-button"
            type="button"
            onClick={() => setDraft((current) => ({ ...current, context: "" }))}
          >
            {copy.removeContext}
          </button>
        )}

        <fieldset>
          <legend>{copy.observationPrompt}</legend>
          {draft.observations.map((item, index) => (
            <div className="repeatable-field" key={item.key}>
              <textarea
                aria-label={`${copy.observationItem} ${index + 1}`}
                value={item.text}
                onChange={(event) =>
                  updateItem("observations", item.key, event.target.value)
                }
                rows={2}
              />
              <button
                className="text-button"
                type="button"
                onClick={() => removeItem("observations", item.key)}
              >
                {copy.remove}
              </button>
            </div>
          ))}
          <button className="secondary-button" type="button" onClick={() => addItem("observations")}>
            {copy.addObservation}
          </button>
        </fieldset>

        <fieldset>
          <legend>{copy.thoughtPrompt}</legend>
          {draft.thoughts.map((item, index) => (
            <div className="repeatable-field" key={item.key}>
              <textarea
                aria-label={`${copy.thoughtItem} ${index + 1}`}
                value={item.text}
                onChange={(event) => updateItem("thoughts", item.key, event.target.value)}
                rows={2}
              />
              <button
                className="text-button"
                type="button"
                onClick={() => removeItem("thoughts", item.key)}
              >
                {copy.remove}
              </button>
            </div>
          ))}
          <button className="secondary-button" type="button" onClick={() => addItem("thoughts")}>
            {copy.addThought}
          </button>
        </fieldset>
      </div>

      {errorMessage && <p className="capture-error" role="alert">{errorMessage}</p>}
      {!canSave && !errorMessage && <p className="validation-note">{copy.emptyValidation}</p>}

      <div className="capture-actions">
        <button className="secondary-button" type="button" disabled={saving} onClick={reset}>
          {copy.cancel}
        </button>
        <button
          className="primary-button"
          type="button"
          disabled={!canSave || saving}
          onClick={handleSave}
        >
          {saving ? copy.saving : copy.save}
        </button>
      </div>
    </section>
  );
}
