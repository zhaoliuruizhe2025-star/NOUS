import type { Messages } from "../i18n";
import {
  readyForCandidateAnalysis,
  type CandidateState,
  type ClassificationAnswer,
  type PresentExperienceRelation,
  type ThoughtRecurrenceCandidate,
} from "../app/thought_recurrence_candidate";

type Copy = Messages["systemCandidate"];

function ExperienceList({ relations, copy }: { relations: readonly PresentExperienceRelation[]; copy: Copy }) {
  return (
    <ul>
      {relations.map((relation) => (
        <li key={relation.thoughtId}>
          {relation.anchorId} ↔ {relation.thoughtId}: {copy.experience[relation.relation]}
        </li>
      ))}
    </ul>
  );
}

function Proposal({ payload, copy }: { payload: ThoughtRecurrenceCandidate; copy: Copy }) {
  return (
    <section className="system-candidate-result" aria-labelledby="system-candidate-result-title">
      <h3 id="system-candidate-result-title">{copy.proposalTitle}</h3>
      <p>{copy.conclusion}</p>
      <p className="system-candidate-limit">{copy.atLeastTwoOnly}</p>
      <h4>{copy.whyTitle}</h4>
      <p>{copy.proofIntro}</p>
      <ExperienceList relations={payload.proofEdges} copy={copy} />
      {payload.presentWithoutDistinctExperienceIds.length > 0 && (
        <>
          <p>{copy.presentWithoutDistinct}</p>
          <ExperienceList
            relations={payload.presentExperienceRelations.filter((relation) =>
              relation.relation !== "differentExperiences")}
            copy={copy}
          />
        </>
      )}
      <h4>{copy.limitsTitle}</h4>
      {payload.absentIds.length > 0 && (
        <p>{copy.absentSummary.replace("{count}", String(payload.absentIds.length))}: {payload.absentIds.join(", ")}</p>
      )}
      {payload.cannotTellIds.length > 0 && (
        <p>{copy.cannotTellSummary.replace("{count}", String(payload.cannotTellIds.length))}: {payload.cannotTellIds.join(", ")}</p>
      )}
      <p>{copy.scopeLimit}</p>
      <p>{copy.truthLimit}</p>
      <p>{copy.dependenceLimit}</p>
      <p>{copy.timeLimit}</p>
      <p>{copy.invalidation}</p>
      <p className="system-candidate-meta">{copy.mechanism}: {payload.mechanism.id} v{payload.mechanism.version} · {copy.expressionVersion} {payload.mechanism.expressionVersion}</p>
    </section>
  );
}

export function SystemCandidateReview({ state, messages, onXChange, onClassificationChange, onAnalyze, onDismiss, onReturn, onClose }: {
  state: CandidateState;
  messages: Messages;
  onXChange: (x: string) => void;
  onClassificationChange: (id: string, answer: ClassificationAnswer) => void;
  onAnalyze: () => void;
  onDismiss: () => void;
  onReturn: () => void;
  onClose: () => void;
}) {
  if (state.kind === "closed") return null;
  const copy = messages.systemCandidate;
  if (state.kind === "stale") {
    return (
      <section className="system-candidate-card" aria-labelledby="system-candidate-title">
        <h2 id="system-candidate-title">{copy.title}</h2>
        <p role="alert">{copy.stale}: {state.affectedIds.join(", ")}</p>
        <button type="button" className="secondary-button" onClick={onClose}>{copy.returnToComparison}</button>
      </section>
    );
  }

  const ready = readyForCandidateAnalysis(state);
  const { handoff } = state;
  return (
    <section className="system-candidate-card" aria-labelledby="system-candidate-title">
      <p className="eyebrow">{copy.eyebrow}</p>
      <h2 id="system-candidate-title">{copy.title}</h2>
      <p>{copy.intro}</p>
      <p className="system-candidate-limit">{copy.notSaved}</p>

      <h3>{copy.frameLabel}</h3>
      <blockquote className="system-candidate-user-text">{handoff.comparisonFrame}</blockquote>
      <label className="system-candidate-x" htmlFor="system-candidate-x">
        <strong>{copy.xLabel}</strong>
        <textarea
          id="system-candidate-x"
          rows={3}
          value={state.x}
          onChange={(event) => onXChange(event.currentTarget.value)}
        />
      </label>
      <p>{copy.xGuidance}</p>
      <h3>{copy.scopeTitle}</h3>
      <p>{copy.scopeIntro}</p>
      <ul className="system-candidate-sources">
        {handoff.selectedThoughts.map((source) => (
          <li key={source.id}>
            <strong>{source.id === handoff.anchorId ? copy.anchor : copy.thought}: {source.id}</strong>
            <p className="system-candidate-user-text">{source.content}</p>
            <p>{source.linkedSituation
              ? `${copy.context}: ${source.linkedSituation.description}`
              : copy.noContext}</p>
            <label>
              {copy.classificationQuestion}
              <select
                value={state.classifications[source.id] ?? ""}
                onChange={(event) => onClassificationChange(source.id, (event.currentTarget.value || null) as ClassificationAnswer)}
              >
                <option value="">{copy.classifications.unanswered}</option>
                <option value="present">{copy.classifications.present}</option>
                <option value="absent">{copy.classifications.absent}</option>
                <option value="cannotTell">{copy.classifications.cannotTell}</option>
              </select>
            </label>
          </li>
        ))}
      </ul>

      {state.kind === "checking" && <p role="status">{copy.checking}</p>}
      {state.kind === "loadError" && <p role="alert">{copy.errors[state.code]}</p>}
      {state.kind === "zeroResult" && (
        <section className="system-candidate-result" aria-labelledby="system-candidate-zero-title">
          <h3 id="system-candidate-zero-title">{copy.zeroTitle}</h3>
          <p>{copy.zeroReasons[state.reason]}</p>
          <p>{copy.zeroLimit}</p>
        </section>
      )}
      {state.kind === "proposed" && <Proposal payload={state.payload} copy={copy} />}
      {state.kind === "dismissed" && <p role="status">{copy.dismissed}</p>}
      {state.kind === "draft" && !ready && <p>{copy.incomplete}</p>}

      <div className="system-candidate-actions">
        <button type="button" className="primary-button" disabled={!ready || state.kind === "checking"} onClick={onAnalyze}>{copy.analyze}</button>
        {(state.kind === "proposed" || state.kind === "zeroResult" || state.kind === "loadError") && (
          <button type="button" className="secondary-button" onClick={onReturn}>{copy.returnToGrounding}</button>
        )}
        {state.kind === "proposed" && <button type="button" className="secondary-button" onClick={onDismiss}>{copy.dismiss}</button>}
        <button type="button" className="text-button" onClick={onClose}>{copy.close}</button>
      </div>
    </section>
  );
}
