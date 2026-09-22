import { useCallback, useEffect, useRef, useState } from "react";

import type { Messages } from "../i18n";
import { loadStructuredHistory, type StructuredHistoryCommandError } from "../app/history";
import {
  allCandidateSources,
  assignCandidateRole,
  canReviewCandidate,
  compareCandidateSources,
  createRefreshGate,
  projectCandidateSources,
  selectedSourceKey,
  sourceKey,
  toggleCandidateSource,
  type CandidateSource,
  type CandidateSourceSnapshot,
  type EvidenceRole,
  type SelectedCandidateSource,
} from "../app/candidate_review";

type ReviewStatus = "draft" | "checking" | "summary" | "stale";
type RefreshPurpose = "entry" | "review" | "focus";

function loadErrorCode(error: unknown): string {
  if (typeof error === "object" && error !== null && "code" in error) {
    return String((error as StructuredHistoryCommandError).code);
  }
  return "loadFailed";
}

function SourceOption({ source, selected, messages, onToggle, onRoleChange }: {
  source: CandidateSource;
  selected: SelectedCandidateSource | undefined;
  messages: Messages;
  onToggle: (source: CandidateSource) => void;
  onRoleChange: (source: CandidateSource, role: EvidenceRole | null) => void;
}) {
  const copy = messages.candidateReview;
  const isCurrentSelection = selected?.status === "current";
  return (
    <li className="candidate-source">
      <label className="candidate-source__choice">
        <input
          type="checkbox"
          checked={isCurrentSelection}
          onChange={() => onToggle(source)}
          aria-label={`${isCurrentSelection ? copy.deselectSource : copy.selectSource}: ${source.content}`}
        />
        <span className="candidate-source__kind">{copy.sourceTypes[source.recordType]}</span>
        <span className="candidate-source__content">{source.content}</span>
      </label>
      {source.linkedSituation && (
        <p className="candidate-source__context">
          {copy.linkedContext}: {source.linkedSituation.description}
        </p>
      )}
      {source.recordType === "thought" && source.subjectiveConviction !== null && (
        <p className="candidate-source__context">
          {copy.subjectiveConviction.replace("{value}", String(source.subjectiveConviction))}
        </p>
      )}
      {isCurrentSelection && (
        <label className="candidate-source__role">
          {copy.roleLabel}
          <select
            value={selected.role ?? ""}
            onChange={(event) =>
              onRoleChange(source, (event.currentTarget.value || null) as EvidenceRole | null)
            }
            aria-label={`${copy.roleLabel}: ${source.content}`}
          >
            <option value="">{copy.unclassified}</option>
            <option value="supporting">{copy.roles.supporting}</option>
            <option value="contradicting">{copy.roles.contradicting}</option>
            <option value="complicating">{copy.roles.complicating}</option>
            <option value="contextualizing">{copy.roles.contextualizing}</option>
          </select>
        </label>
      )}
      {selected?.status === "stale" && (
        <p className="candidate-source__stale">{copy.reviewCurrentSource}</p>
      )}
    </li>
  );
}

export function CandidateReview({ messages, onClose }: {
  messages: Messages;
  onClose: () => void;
}) {
  const copy = messages.candidateReview;
  const [snapshot, setSnapshot] = useState<CandidateSourceSnapshot | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [candidateText, setCandidateText] = useState("");
  const [claimScope, setClaimScope] = useState("");
  const [selectedSources, setSelectedSources] = useState<SelectedCandidateSource[]>([]);
  const [reviewStatus, setReviewStatus] = useState<ReviewStatus>("draft");
  const [reviewInvalid, setReviewInvalid] = useState(false);
  const selectedRef = useRef(selectedSources);
  const reviewStatusRef = useRef(reviewStatus);
  const draftRevisionRef = useRef(0);
  const gateRef = useRef(createRefreshGate());

  function setSelection(next: SelectedCandidateSource[]) {
    selectedRef.current = next;
    setSelectedSources(next);
  }

  function setStatus(next: ReviewStatus) {
    reviewStatusRef.current = next;
    setReviewStatus(next);
  }

  function editDraft() {
    draftRevisionRef.current += 1;
    setReviewInvalid(false);
    setStatus("draft");
  }

  const refresh = useCallback(async (purpose: RefreshPurpose) => {
    const gate = gateRef.current;
    const request = gate.start();
    const draftRevision = draftRevisionRef.current;
    const summaryRequested =
      purpose === "review" ||
      reviewStatusRef.current === "summary" ||
      reviewStatusRef.current === "checking";
    if (purpose === "entry") setLoading(true);
    if (summaryRequested) {
      reviewStatusRef.current = "checking";
      setReviewStatus("checking");
    }
    try {
      const history = await loadStructuredHistory();
      if (!gate.isCurrent(request)) return;
      const currentSnapshot = projectCandidateSources(history);
      const comparison = compareCandidateSources(selectedRef.current, currentSnapshot);
      selectedRef.current = comparison.selectedSources;
      setSelectedSources(comparison.selectedSources);
      setSnapshot(currentSnapshot);
      setLoadError(null);
      setLoading(false);
      const nextStatus: ReviewStatus = comparison.stale
        ? "stale"
        : draftRevision === draftRevisionRef.current && summaryRequested
          ? "summary"
          : "draft";
      reviewStatusRef.current = nextStatus;
      setReviewStatus(nextStatus);
    } catch (error) {
      if (!gate.isCurrent(request)) return;
      setSnapshot(null);
      setLoadError(loadErrorCode(error));
      setLoading(false);
      reviewStatusRef.current = "draft";
      setReviewStatus("draft");
    }
  }, []);

  useEffect(() => {
    gateRef.current = createRefreshGate();
    const entryLoad = window.setTimeout(() => void refresh("entry"), 0);
    const onFocus = () => void refresh("focus");
    window.addEventListener("focus", onFocus);
    return () => {
      window.clearTimeout(entryLoad);
      window.removeEventListener("focus", onFocus);
      gateRef.current.dispose();
    };
  }, [refresh]);

  function toggleSource(source: CandidateSource) {
    setSelection(toggleCandidateSource(selectedRef.current, source));
    editDraft();
  }

  function changeRole(source: CandidateSource, role: EvidenceRole | null) {
    setSelection(assignCandidateRole(selectedRef.current, source, role));
    editDraft();
  }

  function removeStale(recordType: string, id: string) {
    setSelection(selectedRef.current.filter((selected) =>
      selected.status !== "stale" || selected.recordType !== recordType || selected.id !== id,
    ));
    editDraft();
  }

  function clearReview() {
    draftRevisionRef.current += 1;
    setCandidateText("");
    setClaimScope("");
    setSelection([]);
    setReviewInvalid(false);
    setStatus("draft");
  }

  function reviewSummary() {
    if (!canReviewCandidate(candidateText, claimScope, selectedRef.current)) {
      setReviewInvalid(true);
      return;
    }
    setReviewInvalid(false);
    void refresh("review");
  }

  const sourceOption = (source: CandidateSource) => (
    <SourceOption
      key={sourceKey(source.recordType, source.id)}
      source={source}
      selected={selectedSources.find((selected) =>
        selectedSourceKey(selected) === sourceKey(source.recordType, source.id),
      )}
      messages={messages}
      onToggle={toggleSource}
      onRoleChange={changeRole}
    />
  );

  return (
    <section className="candidate-card" aria-labelledby="candidate-title">
      <p className="eyebrow">{copy.eyebrow}</p>
      <h1 id="candidate-title">{copy.title}</h1>
      <p className="candidate-intro">{copy.intro}</p>
      <p className="candidate-boundary">{copy.notSaved}</p>

      <div className="candidate-fields">
        <label htmlFor="candidate-text">{copy.candidateLabel}</label>
        <textarea
          id="candidate-text"
          value={candidateText}
          onChange={(event) => {
            setCandidateText(event.currentTarget.value);
            editDraft();
          }}
          placeholder={copy.candidatePlaceholder}
          rows={3}
        />
        <p className="candidate-guidance">{copy.wordingGuidance}</p>
        <label htmlFor="candidate-scope">{copy.claimScopeLabel}</label>
        <textarea
          id="candidate-scope"
          value={claimScope}
          onChange={(event) => {
            setClaimScope(event.currentTarget.value);
            editDraft();
          }}
          placeholder={copy.claimScopePlaceholder}
          rows={2}
        />
        <p className="candidate-guidance">{copy.scopeGuidance}</p>
      </div>

      <section className="candidate-browser" aria-labelledby="candidate-sources-title">
        <h2 id="candidate-sources-title">{copy.sourcesTitle}</h2>
        <p>{copy.roleMeaning}</p>
        {loading && <p role="status">{copy.loading}</p>}
        {loadError && (
          <div role="alert" className="candidate-error">
            <p>{copy.errors[loadError as keyof typeof copy.errors] ?? copy.errors.loadFailed}</p>
            <button type="button" className="secondary-button" onClick={() => void refresh("entry")}>{copy.retry}</button>
          </div>
        )}
        {snapshot && allCandidateSources(snapshot).length === 0 && <p>{copy.noSources}</p>}
        {snapshot && allCandidateSources(snapshot).length > 0 && (
          <div className="candidate-groups">
            {snapshot.contexts.map((group) => (
              <section key={group.situation.id} className="candidate-group">
                <h3>{copy.situationHeading}</h3>
                <p className="candidate-context-note">{copy.situationDualRole}</p>
                <ul>{sourceOption(group.situation)}</ul>
                {group.observations.length > 0 && (
                  <><h4>{copy.observationHeading}</h4><ul>{group.observations.map(sourceOption)}</ul></>
                )}
                {group.thoughts.length > 0 && (
                  <><h4>{copy.thoughtHeading}</h4><ul>{group.thoughts.map(sourceOption)}</ul></>
                )}
              </section>
            ))}
            {snapshot.standaloneObservations.length > 0 && (
              <section className="candidate-group">
                <h3>{copy.standaloneObservations}</h3>
                <ul>{snapshot.standaloneObservations.map(sourceOption)}</ul>
              </section>
            )}
            {snapshot.standaloneThoughts.length > 0 && (
              <section className="candidate-group">
                <h3>{copy.standaloneThoughts}</h3>
                <ul>{snapshot.standaloneThoughts.map(sourceOption)}</ul>
              </section>
            )}
          </div>
        )}
      </section>

      {selectedSources.some((selected) => selected.status === "stale") && (
        <div className="candidate-error" role="alert">
          <p>{copy.staleNotice}</p>
          <ul>{selectedSources.filter((selected) => selected.status === "stale").map((selected) => {
            if (selected.status !== "stale") return null;
            return <li key={sourceKey(selected.recordType, selected.id)}>
              <span>{copy.sourceTypes[selected.recordType]}: {copy[selected.reason]}</span>
              <button type="button" className="text-button" onClick={() => removeStale(selected.recordType, selected.id)}>{copy.removeUnavailable}</button>
            </li>;
          })}</ul>
        </div>
      )}

      {reviewInvalid && <p role="alert" className="candidate-error">{copy.reviewInvalid}</p>}
      {reviewStatus === "checking" && <p role="status">{copy.checking}</p>}
      {reviewStatus === "summary" && !loadError && canReviewCandidate(candidateText, claimScope, selectedSources) && (
        <section className="candidate-summary" aria-labelledby="candidate-summary-title">
          <h2 id="candidate-summary-title">{copy.summaryTitle}</h2>
          <h3>{copy.candidateLabel}</h3>
          <p>{candidateText}</p>
          <h3>{copy.claimScopeLabel}</h3>
          <p>{claimScope}</p>
          <h3>{copy.analysisScope}</h3>
          <p>{copy.analysisScopeExplanation}</p>
          <ul>{selectedSources.map((selected) => {
            if (selected.status !== "current") return null;
            const source = selected.source;
            return <li key={sourceKey(source.recordType, source.id)}>
              <strong>{copy.sourceTypes[source.recordType]} — {selected.role ? copy.roles[selected.role] : copy.unclassified}</strong>
              <p>{source.content}</p>
              {source.linkedSituation && <p className="candidate-source__context">{copy.linkedContext}: {source.linkedSituation.description}</p>}
              {source.recordType === "thought" && source.subjectiveConviction !== null && <p className="candidate-source__context">{copy.subjectiveConviction.replace("{value}", String(source.subjectiveConviction))}</p>}
            </li>;
          })}</ul>
          <h3>{copy.limitationsTitle}</h3>
          <ul className="candidate-limitations">
            <li>{copy.limitedSelection}</li>
            <li>{copy.dependence}</li>
            <li>{copy.noChronology}</li>
            <li>{copy.noConclusion}</li>
          </ul>
          <button type="button" className="secondary-button" onClick={() => setStatus("draft")}>{copy.continueEditing}</button>
        </section>
      )}

      <div className="candidate-actions">
        <button type="button" className="primary-button" disabled={loading || !!loadError || reviewStatus === "checking" || !canReviewCandidate(candidateText, claimScope, selectedSources)} onClick={reviewSummary}>{copy.reviewSummary}</button>
        <button type="button" className="secondary-button" onClick={clearReview}>{copy.clearReview}</button>
        <button type="button" className="text-button" onClick={onClose}>{copy.close}</button>
      </div>
    </section>
  );
}
