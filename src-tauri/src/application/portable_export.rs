use serde::Serialize;
use serde_json::{json, Value};

use crate::{
    domain::{EvidenceRelationKind, EvidenceSource, EvidenceTarget, RevisionOrigin},
    persistence::{PersistenceError, PortableUserDataSnapshot, SqliteSelfModelRepository},
};

use super::database_backup::{publish_artifact, ArtifactError, TempArtifact};

#[derive(Debug)]
pub(crate) enum PortableExportError {
    Persistence(PersistenceError),
    Serialization,
    Artifact(ArtifactError),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepresentationCorrections {
    situations: Vec<Value>,
    observations: Vec<Value>,
    thoughts: Vec<Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PortableUserData {
    format: &'static str,
    format_version: u32,
    source_schema_version: u32,
    exported_at_ms: i64,
    subject_scope: &'static str,
    people: Vec<Value>,
    situations: Vec<Value>,
    observations: Vec<Value>,
    thoughts: Vec<Value>,
    emotions: Vec<Value>,
    beliefs: Vec<Value>,
    values: Vec<Value>,
    memories: Vec<Value>,
    decisions: Vec<Value>,
    outcomes: Vec<Value>,
    evidence_links: Vec<Value>,
    representation_corrections: RepresentationCorrections,
}

fn revision_origin_name(origin: RevisionOrigin) -> &'static str {
    match origin {
        RevisionOrigin::InitialUserEntry => "initialUserEntry",
        RevisionOrigin::UserUpdate => "userUpdate",
        RevisionOrigin::UserCorrection => "userCorrection",
    }
}

fn evidence_relation_name(relation: EvidenceRelationKind) -> &'static str {
    match relation {
        EvidenceRelationKind::Supports => "supports",
        EvidenceRelationKind::Contradicts => "contradicts",
        EvidenceRelationKind::Complicates => "complicates",
        EvidenceRelationKind::Contextualizes => "contextualizes",
    }
}

fn evidence_source(source: &EvidenceSource) -> Value {
    let (kind, id) = match source {
        EvidenceSource::Situation(id) => ("situation", id.as_str()),
        EvidenceSource::Observation(id) => ("observation", id.as_str()),
        EvidenceSource::Thought(id) => ("thought", id.as_str()),
        EvidenceSource::Emotion(id) => ("emotion", id.as_str()),
        EvidenceSource::Memory(id) => ("memory", id.as_str()),
        EvidenceSource::Decision(id) => ("decision", id.as_str()),
        EvidenceSource::Outcome(id) => ("outcome", id.as_str()),
    };
    json!({"type": kind, "id": id})
}

fn evidence_target(target: &EvidenceTarget) -> Value {
    match target {
        EvidenceTarget::BeliefRevision {
            belief_id,
            revision_id,
        } => json!({
            "type": "beliefRevision", "beliefId": belief_id.as_str(), "revisionId": revision_id.as_str()
        }),
        EvidenceTarget::ValueRevision {
            value_id,
            revision_id,
        } => json!({
            "type": "valueRevision", "valueId": value_id.as_str(), "revisionId": revision_id.as_str()
        }),
    }
}

pub(crate) fn serialize_portable_user_data(
    mut snapshot: PortableUserDataSnapshot,
    exported_at_ms: i64,
) -> Result<Vec<u8>, PortableExportError> {
    if exported_at_ms < 0 {
        return Err(PortableExportError::Serialization);
    }
    snapshot
        .history
        .situations
        .sort_by(|a, b| a.value.id().as_str().cmp(b.value.id().as_str()));
    snapshot
        .history
        .observations
        .sort_by(|a, b| a.value.id().as_str().cmp(b.value.id().as_str()));
    snapshot
        .history
        .thoughts
        .sort_by(|a, b| a.value.id().as_str().cmp(b.value.id().as_str()));

    let mut situation_corrections = Vec::new();
    let situations = snapshot.history.situations.iter().map(|record| {
        for correction in &record.corrections {
            situation_corrections.push(json!({
                "targetId": record.value.id().as_str(),
                "sequence": correction.sequence,
                "priorInaccurateRepresentation": {"description": correction.before_description},
                "replacementRepresentation": {"description": correction.after_description},
                "userNote": correction.user_note,
                "recordedAtMs": correction.recorded_at_ms
            }));
        }
        json!({"id": record.value.id().as_str(), "description": record.value.description(), "savedAtMs": record.created_at_ms})
    }).collect();
    let mut observation_corrections = Vec::new();
    let observations = snapshot.history.observations.iter().map(|record| {
        for correction in &record.corrections {
            observation_corrections.push(json!({
                "targetId": record.value.id().as_str(),
                "sequence": correction.sequence,
                "priorInaccurateRepresentation": {"content": correction.before_content, "situationId": correction.before_situation_id.as_ref().map(|id| id.as_str())},
                "replacementRepresentation": {"content": correction.after_content, "situationId": correction.after_situation_id.as_ref().map(|id| id.as_str())},
                "userNote": correction.user_note,
                "recordedAtMs": correction.recorded_at_ms
            }));
        }
        json!({"id": record.value.id().as_str(), "situationId": record.value.situation_id().map(|id| id.as_str()), "content": record.value.content(), "savedAtMs": record.created_at_ms})
    }).collect();
    let mut thought_corrections = Vec::new();
    let thoughts = snapshot.history.thoughts.iter().map(|record| {
        for correction in &record.corrections {
            thought_corrections.push(json!({
                "targetId": record.value.id().as_str(),
                "sequence": correction.sequence,
                "priorInaccurateRepresentation": {"content": correction.before_content, "situationId": correction.before_situation_id.as_ref().map(|id| id.as_str()), "subjectiveConviction": correction.before_confidence.map(|v| v.value())},
                "replacementRepresentation": {"content": correction.after_content, "situationId": correction.after_situation_id.as_ref().map(|id| id.as_str()), "subjectiveConviction": correction.after_confidence.map(|v| v.value())},
                "userNote": correction.user_note,
                "recordedAtMs": correction.recorded_at_ms
            }));
        }
        json!({"id": record.value.id().as_str(), "situationId": record.value.situation_id().map(|id| id.as_str()), "content": record.value.content(), "subjectiveConviction": record.value.confidence().map(|v| v.value()), "savedAtMs": record.created_at_ms})
    }).collect();

    let document = PortableUserData {
        format: "nous-user-data",
        format_version: 1,
        source_schema_version: 5,
        exported_at_ms,
        subject_scope: "singleCurrentUser",
        people: snapshot.people.iter().map(|r| json!({"id": r.value.id().as_str(), "displayName": r.value.display_name(), "relationshipLabel": r.value.relationship_label(), "contextNotes": r.value.context_notes(), "savedAtMs": r.saved_at_ms})).collect(),
        situations,
        observations,
        thoughts,
        emotions: snapshot.emotions.iter().map(|r| json!({"id": r.value.id().as_str(), "situationId": r.value.situation_id().map(|id| id.as_str()), "label": r.value.label(), "intensity": r.value.intensity().value(), "savedAtMs": r.saved_at_ms})).collect(),
        beliefs: snapshot.beliefs.iter().map(|b| json!({"id": b.record.value.id().as_str(), "savedAtMs": b.record.saved_at_ms, "revisions": b.revisions.iter().map(|r| json!({"id": r.value.id().as_str(), "revisionNumber": r.value.revision_number().value(), "proposition": r.value.proposition(), "endorsement": r.value.endorsement().map(|v| v.value()), "changeNote": r.value.change_note(), "origin": revision_origin_name(r.value.origin()), "savedAtMs": r.saved_at_ms})).collect::<Vec<_>>() })).collect(),
        values: snapshot.values.iter().map(|v| json!({"id": v.record.value.id().as_str(), "savedAtMs": v.record.saved_at_ms, "revisions": v.revisions.iter().map(|r| json!({"id": r.value.id().as_str(), "revisionNumber": r.value.revision_number().value(), "label": r.value.label(), "importance": r.value.importance().map(|v| v.value()), "changeNote": r.value.change_note(), "origin": revision_origin_name(r.value.origin()), "savedAtMs": r.saved_at_ms})).collect::<Vec<_>>() })).collect(),
        memories: snapshot.memories.iter().map(|r| json!({"id": r.value.id().as_str(), "situationId": r.value.situation_id().map(|id| id.as_str()), "description": r.value.description(), "userMeaning": r.value.user_meaning(), "savedAtMs": r.saved_at_ms})).collect(),
        decisions: snapshot.decisions.iter().map(|r| json!({"id": r.value.id().as_str(), "situationId": r.value.situation_id().map(|id| id.as_str()), "description": r.value.description(), "savedAtMs": r.saved_at_ms})).collect(),
        outcomes: snapshot.outcomes.iter().map(|r| json!({"id": r.value.id().as_str(), "decisionId": r.value.decision_id().as_str(), "description": r.value.description(), "savedAtMs": r.saved_at_ms})).collect(),
        evidence_links: snapshot.evidence_links.iter().map(|r| json!({"id": r.value.id().as_str(), "source": evidence_source(r.value.source()), "target": evidence_target(r.value.target()), "relation": evidence_relation_name(r.value.relationship()), "provenance": "userAuthored", "userNote": r.value.user_note(), "savedAtMs": r.saved_at_ms})).collect(),
        representation_corrections: RepresentationCorrections { situations: situation_corrections, observations: observation_corrections, thoughts: thought_corrections },
    };
    serde_json::to_vec_pretty(&document).map_err(|_| PortableExportError::Serialization)
}

pub(crate) async fn export_user_data_to_path(
    repository: &SqliteSelfModelRepository,
    destination: &std::path::Path,
    exported_at_ms: i64,
) -> Result<(), PortableExportError> {
    let snapshot = repository
        .load_portable_user_data_snapshot()
        .await
        .map_err(PortableExportError::Persistence)?;
    let bytes = serialize_portable_user_data(snapshot, exported_at_ms)?;
    let artifact = TempArtifact::new(destination, "json").map_err(PortableExportError::Artifact)?;
    artifact
        .write_bytes(&bytes)
        .map_err(PortableExportError::Artifact)?;
    publish_artifact(&artifact, destination).map_err(PortableExportError::Artifact)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Belief, BeliefId, BeliefRevision, BeliefRevisionId, EvidenceLink, EvidenceLinkId,
        EvidenceProvenance, Observation, ObservationId, RevisionNumber, SelfSubjectId, Situation,
        SituationId, Thought, ThoughtConfidence, ThoughtId,
    };
    use crate::persistence::{
        HistorySituationRecord, HistoryThoughtRecord, SavedBelief, SavedRecord,
        SituationCorrectionRecord, StructuredHistoryRecords, ThoughtCorrectionRecord,
    };

    #[test]
    fn empty_json_v1_has_every_collection_without_bootstrap_metadata() {
        let snapshot = PortableUserDataSnapshot {
            history: StructuredHistoryRecords {
                situations: vec![],
                observations: vec![],
                thoughts: vec![],
            },
            people: vec![],
            emotions: vec![],
            beliefs: vec![],
            values: vec![],
            memories: vec![],
            decisions: vec![],
            outcomes: vec![],
            evidence_links: vec![],
        };
        let bytes = serialize_portable_user_data(snapshot, 123).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["format"], "nous-user-data");
        assert_eq!(value["formatVersion"], 1);
        assert_eq!(value["sourceSchemaVersion"], 5);
        assert_eq!(value["exportedAtMs"], 123);
        assert_eq!(value["subjectScope"], "singleCurrentUser");
        for name in [
            "people",
            "situations",
            "observations",
            "thoughts",
            "emotions",
            "beliefs",
            "values",
            "memories",
            "decisions",
            "outcomes",
            "evidenceLinks",
        ] {
            assert_eq!(value[name], json!([]), "{name}");
        }
        assert_eq!(value["representationCorrections"]["situations"], json!([]));
        assert!(value.get("subject").is_none());
        assert!(!String::from_utf8(bytes).unwrap().contains("stateToken"));
    }

    #[test]
    fn current_values_revisions_evidence_and_inaccurate_prior_values_are_distinct() {
        let subject = SelfSubjectId::new("self").unwrap();
        let situation_id = SituationId::new("s").unwrap();
        let observation_id = ObservationId::new("o").unwrap();
        let thought_id = ThoughtId::new("t").unwrap();
        let belief_id = BeliefId::new("b").unwrap();
        let revision_id = BeliefRevisionId::new("br").unwrap();
        let situation = Situation::new(situation_id.clone(), subject.clone(), "Thursday").unwrap();
        let observation = Observation::new(
            observation_id.clone(),
            subject.clone(),
            Some(situation_id.clone()),
            "They said they were busy",
        )
        .unwrap();
        let thought = Thought::new(
            thought_id.clone(),
            subject.clone(),
            None,
            "I worried",
            Some(ThoughtConfidence::new(60).unwrap()),
        )
        .unwrap();
        let belief = Belief::new(belief_id.clone(), subject.clone());
        let revision = BeliefRevision::new(
            revision_id.clone(),
            belief_id.clone(),
            RevisionNumber::new(1).unwrap(),
            "I can ask",
            None,
            None,
            RevisionOrigin::InitialUserEntry,
        )
        .unwrap();
        let evidence = EvidenceLink::new(
            EvidenceLinkId::new("link").unwrap(),
            subject,
            EvidenceSource::Observation(observation_id),
            EvidenceTarget::BeliefRevision {
                belief_id,
                revision_id,
            },
            EvidenceRelationKind::Supports,
            EvidenceProvenance::UserAuthored,
            None,
        )
        .unwrap();
        let snapshot = PortableUserDataSnapshot {
            history: StructuredHistoryRecords {
                situations: vec![HistorySituationRecord {
                    value: situation,
                    created_at_ms: 3,
                    state_token: "secret-state-token".into(),
                    corrections: vec![SituationCorrectionRecord {
                        sequence: 1,
                        before_description: "Tuesday".into(),
                        after_description: "Thursday".into(),
                        user_note: Some("Wrong day".into()),
                        recorded_at_ms: 12,
                    }],
                }],
                observations: vec![crate::persistence::HistoryObservationRecord {
                    value: observation,
                    created_at_ms: 4,
                    state_token: "secret-state-token".into(),
                    corrections: vec![],
                }],
                thoughts: vec![HistoryThoughtRecord {
                    value: thought,
                    created_at_ms: 5,
                    state_token: "secret-state-token".into(),
                    corrections: vec![ThoughtCorrectionRecord {
                        sequence: 1,
                        before_content: "I panicked".into(),
                        after_content: "I worried".into(),
                        before_situation_id: None,
                        after_situation_id: None,
                        before_confidence: Some(ThoughtConfidence::new(90).unwrap()),
                        after_confidence: Some(ThoughtConfidence::new(60).unwrap()),
                        user_note: None,
                        recorded_at_ms: 13,
                    }],
                }],
            },
            people: vec![],
            emotions: vec![],
            beliefs: vec![SavedBelief {
                record: SavedRecord {
                    value: belief,
                    saved_at_ms: 6,
                },
                revisions: vec![SavedRecord {
                    value: revision,
                    saved_at_ms: 7,
                }],
            }],
            values: vec![],
            memories: vec![],
            decisions: vec![],
            outcomes: vec![],
            evidence_links: vec![SavedRecord {
                value: evidence,
                saved_at_ms: 8,
            }],
        };
        let bytes = serialize_portable_user_data(snapshot, 20).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["situations"][0]["description"], "Thursday");
        assert_eq!(value["thoughts"][0]["subjectiveConviction"], 60);
        assert_eq!(
            value["beliefs"][0]["revisions"][0]["origin"],
            "initialUserEntry"
        );
        assert_eq!(
            value["evidenceLinks"][0]["source"],
            json!({"type":"observation","id":"o"})
        );
        assert_eq!(
            value["evidenceLinks"][0]["target"],
            json!({"type":"beliefRevision","beliefId":"b","revisionId":"br"})
        );
        assert_eq!(
            value["representationCorrections"]["situations"][0]["priorInaccurateRepresentation"]
                ["description"],
            "Tuesday"
        );
        assert_eq!(
            value["representationCorrections"]["thoughts"][0]["priorInaccurateRepresentation"]
                ["subjectiveConviction"],
            90
        );
        assert_eq!(
            value["representationCorrections"]["thoughts"][0]["recordedAtMs"],
            13
        );
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("secret-state-token"));
        assert!(!text.contains("beforeStateToken"));
        assert!(!text.contains("displayName\": \"Self"));
    }
}
