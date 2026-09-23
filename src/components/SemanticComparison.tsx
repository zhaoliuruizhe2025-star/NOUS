import { useCallback, useEffect, useRef, useState } from "react";

import { loadStructuredHistory, type StructuredHistoryCommandError } from "../app/history";
import {
  candidateRequestIsCurrent,
  changedScopeSources,
  deriveCandidate,
  dismissCandidate,
  editClassification,
  editX,
  failCandidateLoad,
  finishChecking,
  handoffFromReviewedComparison,
  readyForCandidateAnalysis,
  restoreCheckedResult,
  returnToGrounding,
  staleCandidate,
  startCandidateAttempt,
  startChecking,
  syncComparisonEdit,
  type CandidateState,
} from "../app/thought_recurrence_candidate";
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
import { SystemCandidateReview } from "./SystemCandidateReview";

type ReviewStatus = "draft" | "checking" | "reviewed" | "stale";
type RefreshPurpose = "entry" | "review" | "focus" | "candidateAnalyze";
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
  const [candidate, setCandidate] = useState<CandidateState>({ kind: "closed" });
  const draftRef = useRef(draft);
  const candidateRef = useRef(candidate);
  const candidateAttemptRef = useRef(0);
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

  function publishCandidate(next: CandidateState) {
    candidateRef.current = next;
    setCandidate(next);
  }

  function edit(update: (current: ComparisonDraft) => ComparisonDraft) {
    const previous = draftRef.current;
    const next = update(previous);
    if (next === previous) return;
    editVersionRef.current += 1;
    publishCandidate(syncComparisonEdit(candidateRef.current, previous, next));
    publishDraft(next);
    publishStatus("draft");
  }

  const refresh = useCallback(async (purpose: RefreshPurpose) => {
    const gate = gateRef.current;
    const request = gate.start();
    const editVersion = editVersionRef.current;
    const beforeCandidate = candidateRef.current;
    const canAnalyze = purpose === "candidateAnalyze" && readyForCandidateAnalysis(beforeCandidate);
    if (purpose === "candidateAnalyze" && !canAnalyze) return;
    if (canAnalyze) {
      publishCandidate(startChecking(beforeCandidate, "analyze", request));
    } else if (beforeCandidate.kind === "proposed" || beforeCandidate.kind === "zeroResult" ||
        beforeCandidate.kind === "checking") {
      publishCandidate(startChecking(beforeCandidate, "focus", request));
    }
    const candidateAtRequest = candidateRef.current;
    const checkIdentity = candidateAtRequest.kind !== "closed" && candidateAtRequest.kind !== "stale"
      ? { attemptId: candidateAtRequest.attemptId, revision: candidateAtRequest.revision }
      : null;
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
      const currentCandidate = candidateRef.current;
      let candidateSourceChanged = false;
      let changedCandidateIds: readonly string[] = [];
      if (checkIdentity && currentCandidate.kind !== "closed" && currentCandidate.kind !== "stale" &&
          currentCandidate.attemptId === checkIdentity.attemptId && currentCandidate.revision === checkIdentity.revision) {
        const changed = changedScopeSources(currentCandidate.handoff, currentSources);
        if (changed) {
          candidateSourceChanged = true;
          changedCandidateIds = changed.affectedIds;
          publishCandidate(staleCandidate(changed.reason, changed.affectedIds));
        }
        else if (checkIdentity && candidateRequestIsCurrent(currentCandidate, request, checkIdentity.attemptId, checkIdentity.revision)) {
          publishCandidate(currentCandidate.purpose === "analyze"
            ? finishChecking(currentCandidate, deriveCandidate(currentCandidate, request))
            : restoreCheckedResult(currentCandidate));
        }
      }
      if (editVersion !== editVersionRef.current) return;
      const reconciled = reconcileComparison(draftRef.current, currentSources);
      // Task 015 normally detects state-token changes. Also revoke an answer if
      // the defensive content/context check found a change without a new token.
      const changedIds = new Set(changedCandidateIds);
      const anchorChanged = reconciled.draft.anchorId !== null && changedIds.has(reconciled.draft.anchorId);
      const nextDraft: ComparisonDraft = candidateSourceChanged ? {
        ...reconciled.draft,
        selectedRecords: reconciled.draft.selectedRecords.map((selected) =>
          selected.status === "current" && changedIds.has(selected.source.id)
            ? { status: "stale" as const, recordType: selected.source.recordType, id: selected.source.id, reason: "sourceChanged" as const }
            : selected),
        anchorId: anchorChanged ? null : reconciled.draft.anchorId,
        comparisons: anchorChanged ? {} : Object.fromEntries(Object.entries(reconciled.draft.comparisons)
          .filter(([id]) => !changedIds.has(id))),
      } : reconciled.draft;
      draftRef.current = nextDraft;
      setDraft(nextDraft);
      setSources(currentSources);
      setLoadError(null);
      setLoading(false);
      const nextStatus: ReviewStatus = reconciled.changed || candidateSourceChanged
        ? "stale"
        : mayRestoreReviewedSummary(
          nextDraft, restoreReview, reconciled.changed, editVersion, editVersionRef.current,
        )
          ? "reviewed"
          : "draft";
      reviewStatusRef.current = nextStatus;
      setReviewStatus(nextStatus);
    } catch (error) {
      if (!gate.isCurrent(request)) return;
      const currentCandidate = candidateRef.current;
      if (checkIdentity && candidateRequestIsCurrent(currentCandidate, request, checkIdentity.attemptId, checkIdentity.revision)) {
        publishCandidate(failCandidateLoad(currentCandidate, loadErrorCode(error)));
      }
      if (editVersion !== editVersionRef.current) return;
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
    publishCandidate({ kind: "closed" });
    publishDraft(emptyComparison());
    publishStatus("draft");
  }

  function leaveComparison() {
    gateRef.current.dispose();
    publishCandidate({ kind: "closed" });
    onClose();
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
          {draft.sourceType === "thought" && (
            <button type="button" className="secondary-button" onClick={() => {
              const handoff = handoffFromReviewedComparison(draftRef.current);
              if (handoff) publishCandidate(startCandidateAttempt(handoff, ++candidateAttemptRef.current));
            }}>{messages.systemCandidate.begin}</button>
          )}
          <button type="button" className="secondary-button" onClick={() => publishStatus("draft")}>{copy.continueEditing}</button>
        </section>
      )}

      <SystemCandidateReview
        state={candidate}
        messages={messages}
        onXChange={(value) => publishCandidate(editX(candidateRef.current, value))}
        onClassificationChange={(id, answer) => publishCandidate(editClassification(candidateRef.current, id, answer))}
        onAnalyze={() => {
          if (readyForCandidateAnalysis(candidateRef.current)) void refresh("candidateAnalyze");
        }}
        onDismiss={() => publishCandidate(dismissCandidate(candidateRef.current))}
        onReturn={() => publishCandidate(returnToGrounding(candidateRef.current))}
        onClose={() => publishCandidate({ kind: "closed" })}
      />

      <div className="semantic-actions">
        <button
          type="button"
          className="primary-button"
          disabled={loading || loadError !== null || reviewStatus === "checking" || !ready}
          onClick={() => void refresh("review")}
        >{copy.review}</button>
        <button type="button" className="secondary-button" onClick={clearComparison}>{copy.clear}</button>
        <button type="button" className="text-button" onClick={leaveComparison}>{copy.close}</button>
      </div>
    </section>
  );
}
