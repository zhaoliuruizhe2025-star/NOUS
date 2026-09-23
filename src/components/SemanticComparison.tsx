import { useCallback, useEffect, useRef, useState } from "react";

import { loadStructuredHistory, type StructuredHistoryCommandError } from "../app/history";
import {
  addSource,
  answerComparability,
  answerExperience,
  answerMeaning,
  changeComparisonFrame,
  changeSourceType,
  chooseAnchor,
  createComparisonRefreshGate,
  emptyComparison,
  mayRestoreReviewedSummary,
  projectComparisonSources,
  readyForComparison,
  reconcileComparison,
  removeSource,
  selectionId,
  type Comparability,
  type ComparisonDraft,
  type ComparisonSource,
  type ExperienceRelation,
  type MeaningRelation,
} from "../app/semantic_comparison";
import type { Messages } from "../i18n";

type ReviewStatus = "draft" | "checking" | "reviewed" | "stale";
type RefreshPurpose = "entry" | "review" | "focus";
type LoadError = "subjectInvariant" | "dataInconsistent" | "storageUnavailable" | "loadFailed";

function loadErrorCode(error: unknown): LoadError {
  const code = typeof error === "object" && error !== null && "code" in error
    ? String((error as StructuredHistoryCommandError).code)
    : "loadFailed";
  return code === "subjectInvariant" || code === "dataInconsistent" || code === "storageUnavailable"
    ? code
    : "loadFailed";
}

function SourceContext({ source, copy }: {
  source: ComparisonSource;
  copy: Messages["semanticComparison"];
}) {
  return (
    <p className="semantic-source-context">
      {source.linkedSituation
        ? `${copy.linkedSituation}: ${source.linkedSituation.description}`
        : copy.noLinkedSituation}
    </p>
  );
}

export function SemanticComparison({ messages, onClose }: {
  messages: Messages;
  onClose: () => void;
}) {
  const copy = messages.semanticComparison;
  const [draft, setDraft] = useState<ComparisonDraft>(emptyComparison);
  const [sources, setSources] = useState<ComparisonSource[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<LoadError | null>(null);
  const [reviewStatus, setReviewStatus] = useState<ReviewStatus>("draft");
  const draftRef = useRef(draft);
  const reviewStatusRef = useRef(reviewStatus);
  const editVersionRef = useRef(0);
  const gateRef = useRef(createComparisonRefreshGate());

  function publishDraft(next: ComparisonDraft) {
    draftRef.current = next;
    setDraft(next);
  }

  function publishStatus(next: ReviewStatus) {
    reviewStatusRef.current = next;
    setReviewStatus(next);
  }

  function edit(update: (current: ComparisonDraft) => ComparisonDraft) {
    const next = update(draftRef.current);
    if (next === draftRef.current) return;
    editVersionRef.current += 1;
    publishDraft(next);
    publishStatus("draft");
  }

  const refresh = useCallback(async (purpose: RefreshPurpose) => {
    const gate = gateRef.current;
    const request = gate.start();
    const editVersion = editVersionRef.current;
    const restoreReview = purpose === "review"
      || reviewStatusRef.current === "reviewed"
      || reviewStatusRef.current === "checking";
    if (purpose === "entry") setLoading(true);
    if (restoreReview) {
      reviewStatusRef.current = "checking";
      setReviewStatus("checking");
    }
    try {
      const history = await loadStructuredHistory();
      if (!gate.isCurrent(request)) return;
      const currentSources = projectComparisonSources(history);
      const reconciled = reconcileComparison(draftRef.current, currentSources);
      draftRef.current = reconciled.draft;
      setDraft(reconciled.draft);
      setSources(currentSources);
      setLoadError(null);
      setLoading(false);
      const nextStatus: ReviewStatus = reconciled.changed
        ? "stale"
        : mayRestoreReviewedSummary(
          reconciled.draft, restoreReview, reconciled.changed, editVersion, editVersionRef.current,
        )
          ? "reviewed"
          : "draft";
      reviewStatusRef.current = nextStatus;
      setReviewStatus(nextStatus);
    } catch (error) {
      if (!gate.isCurrent(request)) return;
      setSources(null);
      setLoadError(loadErrorCode(error));
      setLoading(false);
      reviewStatusRef.current = "draft";
      setReviewStatus("draft");
    }
  }, []);

  useEffect(() => {
    gateRef.current = createComparisonRefreshGate();
    const entryLoad = window.setTimeout(() => void refresh("entry"), 0);
    const onFocus = () => void refresh("focus");
    window.addEventListener("focus", onFocus);
    return () => {
      window.clearTimeout(entryLoad);
      window.removeEventListener("focus", onFocus);
      gateRef.current.dispose();
    };
  }, [refresh]);

  function clearComparison() {
    editVersionRef.current += 1;
    publishDraft(emptyComparison());
    publishStatus("draft");
  }

  const available = sources?.filter((source) => source.recordType === draft.sourceType) ?? [];
  const anchor = draft.selectedRecords.find((selected) =>
    selected.status === "current" && selected.source.id === draft.anchorId,
  );
  const anchorSource = anchor?.status === "current" ? anchor.source : null;
  const compared = draft.selectedRecords.filter((selected) =>
    selected.status === "current" && selected.source.id !== draft.anchorId,
  );
  const ready = readyForComparison(draft);

  return (
    <section className="semantic-card" aria-labelledby="semantic-title">
      <p className="eyebrow">{copy.eyebrow}</p>
      <h1 id="semantic-title">{copy.title}</h1>
      <p className="semantic-intro">{copy.intro}</p>
      <p className="semantic-boundary">{copy.notSaved}</p>

      <fieldset className="semantic-mode">
        <legend>{copy.modeLabel}</legend>
        {(["thought", "observation"] as const).map((type) => (
          <label key={type}>
            <input
              type="radio"
              name="semantic-source-type"
              value={type}
              checked={draft.sourceType === type}
              onChange={() => edit((current) => changeSourceType(current, type))}
            />
            {copy.sourceTypes[type]}
          </label>
        ))}
      </fieldset>

      <div className="semantic-frame">
        <label htmlFor="semantic-frame">{copy.frameLabel}</label>
        <textarea
          id="semantic-frame"
          value={draft.comparisonFrame}
          onChange={(event) => edit((current) => changeComparisonFrame(current, event.currentTarget.value))}
          rows={2}
        />
        <p>{copy.frameGuidance}</p>
      </div>

      <section className="semantic-browser" aria-labelledby="semantic-sources-title">
        <h2 id="semantic-sources-title">{copy.sourcesTitle}</h2>
        {loading && <p role="status">{copy.loading}</p>}
        {loadError && (
          <div className="semantic-error" role="alert">
            <p>{copy.errors[loadError]}</p>
            <button type="button" className="secondary-button" onClick={() => void refresh("entry")}>{copy.retry}</button>
          </div>
        )}
        {!loading && !loadError && draft.sourceType === null && <p>{copy.chooseMode}</p>}
        {!loading && !loadError && draft.sourceType !== null && available.length === 0 && (
          <p>{draft.sourceType === "thought" ? copy.noThoughts : copy.noObservations}</p>
        )}
        {!loading && !loadError && draft.sourceType !== null && available.length > 0 && (
          <ul className="semantic-source-list">
            {available.map((source) => {
              const selected = draft.selectedRecords.find((item) => selectionId(item) === source.id);
              const checked = selected?.status === "current";
              return (
                <li key={`${source.recordType}:${source.id}`} className="semantic-source">
                  <label className="semantic-source-choice">
                    <input
                      type="checkbox"
                      checked={checked}
                      onChange={() => edit((current) => checked
                        ? removeSource(current, source.id)
                        : addSource(current, source))}
                    />
                    <span>{source.content}</span>
                  </label>
                  <SourceContext source={source} copy={copy} />
                  {checked && (
                    <label className="semantic-anchor-choice">
                      <input
                        type="radio"
                        name="semantic-anchor"
                        checked={draft.anchorId === source.id}
                        onChange={() => edit((current) => chooseAnchor(current, source.id))}
                      />
                      {copy.anchorLabel}
                    </label>
                  )}
                </li>
              );
            })}
          </ul>
        )}
        {draft.selectedRecords.some((selected) => selected.status === "stale") && (
          <div className="semantic-error" role="alert">
            <p>{copy.staleNotice}</p>
            <ul>{draft.selectedRecords.filter((selected) => selected.status === "stale").map((selected) => (
              <li key={`${selected.status === "stale" ? selected.recordType : ""}:${selectionId(selected)}`}>
                {copy.sourceChanged}
                <button type="button" className="text-button" onClick={() => edit((current) => removeSource(current, selectionId(selected)))}>{copy.removeStale}</button>
              </li>
            ))}</ul>
          </div>
        )}
        {draft.selectedRecords.length === 1 && <p>{copy.needAnother}</p>}
      </section>

      {anchorSource && compared.length > 0 && !loadError && (
        <section className="semantic-questions" aria-labelledby="semantic-questions-title">
          <h2 id="semantic-questions-title">{copy.questionsTitle}</h2>
          <p>{copy.anchorIntro}</p>
          <blockquote>{anchorSource.content}</blockquote>
          <SourceContext source={anchorSource} copy={copy} />
          {compared.map((selected) => {
            if (selected.status !== "current") return null;
            const source = selected.source;
            const judgment = draft.comparisons[source.id];
            if (!judgment) return null;
            return (
              <div className="semantic-question" key={source.id}>
                <p className="semantic-relation-title">{copy.anchorRelation}</p>
                <p className="semantic-compared-content">{source.content}</p>
                <SourceContext source={source} copy={copy} />
                <label>
                  {copy.comparabilityQuestion}
                  <select
                    value={judgment.comparability ?? ""}
                    onChange={(event) => edit((current) => answerComparability(current, source.id, event.currentTarget.value as Comparability))}
                  >
                    <option value="" disabled>{copy.unanswered}</option>
                    <option value="comparable">{copy.comparability.comparable}</option>
                    <option value="notComparable">{copy.comparability.notComparable}</option>
                    <option value="cannotTell">{copy.comparability.cannotTell}</option>
                  </select>
                </label>
                {judgment.comparability === "comparable" && (
                  <label>
                    {copy.meaningQuestion}
                    <select
                      value={judgment.meaningRelation ?? ""}
                      onChange={(event) => edit((current) => answerMeaning(current, source.id, event.currentTarget.value as MeaningRelation))}
                    >
                      <option value="" disabled>{copy.unanswered}</option>
                      <option value="similar">{copy.meaning.similar}</option>
                      <option value="materiallyDifferent">{copy.meaning.materiallyDifferent}</option>
                      <option value="cannotTell">{copy.meaning.cannotTell}</option>
                    </select>
                  </label>
                )}
                <label>
                  {copy.experienceQuestion}
                  <select
                    value={judgment.experienceRelation ?? ""}
                    onChange={(event) => edit((current) => answerExperience(current, source.id, event.currentTarget.value as ExperienceRelation))}
                  >
                    <option value="" disabled>{copy.unanswered}</option>
                    <option value="sameOrPossiblySame">{copy.experience.sameOrPossiblySame}</option>
                    <option value="differentExperiences">{copy.experience.differentExperiences}</option>
                    <option value="cannotTell">{copy.experience.cannotTell}</option>
                  </select>
                </label>
              </div>
            );
          })}
        </section>
      )}

      {reviewStatus === "checking" && <p role="status">{copy.checking}</p>}
      {reviewStatus === "stale" && <p className="semantic-error" role="alert">{copy.summaryInvalid}</p>}
      {reviewStatus === "draft" && draft.selectedRecords.length >= 2 && !ready && !loadError && (
        <p className="semantic-guidance">{copy.incomplete}</p>
      )}

      {reviewStatus === "reviewed" && !loadError && ready && anchorSource && (
        <section className="semantic-summary" aria-labelledby="semantic-summary-title">
          <h2 id="semantic-summary-title">{copy.summaryTitle}</h2>
          <h3>{copy.frameLabel}</h3>
          <p className="semantic-user-text">{draft.comparisonFrame}</p>
          <h3>{copy.sourceTypeLabel}</h3>
          <p>{draft.sourceType === "thought" ? copy.sourceTypes.thought : copy.sourceTypes.observation}</p>
          <h3>{copy.anchorLabel}</h3>
          <p className="semantic-user-text">{anchorSource.content}</p>
          <SourceContext source={anchorSource} copy={copy} />
          <h3>{copy.comparisonsTitle}</h3>
          <ul>{compared.map((selected) => {
            if (selected.status !== "current") return null;
            const source = selected.source;
            const judgment = draft.comparisons[source.id];
            if (!judgment) return null;
            return (
              <li key={source.id}>
                <strong>{copy.anchorRelation}</strong>
                <p className="semantic-user-text">{source.content}</p>
                <SourceContext source={source} copy={copy} />
                <p>{copy.comparabilityLabel}: {judgment.comparability === null ? copy.unanswered : copy.comparability[judgment.comparability]}</p>
                {judgment.comparability === "comparable" && (
                  <p>{copy.meaningLabel}: {judgment.meaningRelation === null ? copy.unanswered : copy.meaning[judgment.meaningRelation]}</p>
                )}
                <p>{copy.experienceLabel}: {judgment.experienceRelation === null ? copy.unanswered : copy.experience[judgment.experienceRelation]}</p>
              </li>
            );
          })}</ul>
          <p className="semantic-limitations">{copy.limitations}</p>
          <button type="button" className="secondary-button" onClick={() => publishStatus("draft")}>{copy.continueEditing}</button>
        </section>
      )}

      <div className="semantic-actions">
        <button
          type="button"
          className="primary-button"
          disabled={loading || loadError !== null || reviewStatus === "checking" || !ready}
          onClick={() => void refresh("review")}
        >{copy.review}</button>
        <button type="button" className="secondary-button" onClick={clearComparison}>{copy.clear}</button>
        <button type="button" className="text-button" onClick={onClose}>{copy.close}</button>
      </div>
    </section>
  );
}
