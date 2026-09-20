use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value as JsonValue};

use super::*;

fn subject_id() -> SelfSubjectId {
    SelfSubjectId::new("self-1").unwrap()
}

fn belief_target() -> EvidenceTarget {
    EvidenceTarget::BeliefRevision {
        belief_id: BeliefId::new("belief-1").unwrap(),
        revision_id: BeliefRevisionId::new("belief-revision-1").unwrap(),
    }
}

fn value_target() -> EvidenceTarget {
    EvidenceTarget::ValueRevision {
        value_id: ValueId::new("value-1").unwrap(),
        revision_id: ValueRevisionId::new("value-revision-1").unwrap(),
    }
}

fn assert_round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let encoded = serde_json::to_string(value).unwrap();
    let decoded = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value, &decoded);
}

fn link(
    id: &str,
    source: EvidenceSource,
    target: EvidenceTarget,
    relationship: EvidenceRelationKind,
) -> Result<EvidenceLink, ValidationError> {
    EvidenceLink::new(
        EvidenceLinkId::new(id).unwrap(),
        subject_id(),
        source,
        target,
        relationship,
        EvidenceProvenance::UserAuthored,
        None,
    )
}

fn polarized_sources() -> Vec<EvidenceSource> {
    vec![
        EvidenceSource::Observation(ObservationId::new("observation-1").unwrap()),
        EvidenceSource::Thought(ThoughtId::new("thought-1").unwrap()),
        EvidenceSource::Memory(MemoryId::new("memory-1").unwrap()),
        EvidenceSource::Decision(DecisionId::new("decision-1").unwrap()),
        EvidenceSource::Outcome(OutcomeId::new("outcome-1").unwrap()),
    ]
}

#[test]
fn evidence_link_id_validates_and_serializes_as_a_string() {
    let id = EvidenceLinkId::new("证据-link-1").unwrap();
    assert_eq!(id.as_str(), "证据-link-1");
    assert_eq!(serde_json::to_value(&id).unwrap(), json!("证据-link-1"));
    assert_round_trip(&id);

    for blank in ["", " ", "\t\r\n", "\u{3000}"] {
        assert!(EvidenceLinkId::new(blank).is_err());
    }
}

#[test]
fn every_approved_source_kind_target_triplet_constructs_and_round_trips() {
    let relationships = [
        EvidenceRelationKind::Supports,
        EvidenceRelationKind::Contradicts,
        EvidenceRelationKind::Complicates,
        EvidenceRelationKind::Contextualizes,
    ];

    for source in polarized_sources() {
        for target in [belief_target(), value_target()] {
            for relationship in relationships {
                let evidence = link(
                    "evidence-matrix",
                    source.clone(),
                    target.clone(),
                    relationship,
                )
                .unwrap();
                assert_eq!(evidence.source(), &source);
                assert_eq!(evidence.target(), &target);
                assert_eq!(evidence.relationship(), relationship);
                assert_eq!(evidence.provenance(), EvidenceProvenance::UserAuthored);
                assert_round_trip(&evidence);
            }
        }
    }

    for source in [
        EvidenceSource::Situation(SituationId::new("situation-1").unwrap()),
        EvidenceSource::Emotion(EmotionId::new("emotion-1").unwrap()),
    ] {
        for target in [belief_target(), value_target()] {
            let evidence = link(
                "evidence-context",
                source.clone(),
                target.clone(),
                EvidenceRelationKind::Contextualizes,
            )
            .unwrap();
            assert_round_trip(&evidence);
        }
    }
}

#[test]
fn situation_and_emotion_reject_non_contextual_relationships_for_both_targets() {
    let cases = [
        (
            EvidenceSource::Situation(SituationId::new("situation-1").unwrap()),
            EvidenceRelationKind::Supports,
        ),
        (
            EvidenceSource::Situation(SituationId::new("situation-1").unwrap()),
            EvidenceRelationKind::Contradicts,
        ),
        (
            EvidenceSource::Emotion(EmotionId::new("emotion-1").unwrap()),
            EvidenceRelationKind::Supports,
        ),
        (
            EvidenceSource::Emotion(EmotionId::new("emotion-1").unwrap()),
            EvidenceRelationKind::Complicates,
        ),
    ];

    for (source, relationship) in cases {
        for target in [belief_target(), value_target()] {
            assert!(matches!(
                link("evidence-invalid", source.clone(), target, relationship),
                Err(ValidationError::InvalidEvidenceRelation { .. })
            ));
        }
    }
}

#[test]
fn targets_preserve_revision_family_identity_and_semantics() {
    let belief = link(
        "evidence-belief",
        EvidenceSource::Observation(ObservationId::new("observation-1").unwrap()),
        belief_target(),
        EvidenceRelationKind::Contradicts,
    )
    .unwrap();
    let value = link(
        "evidence-value",
        EvidenceSource::Decision(DecisionId::new("decision-1").unwrap()),
        value_target(),
        EvidenceRelationKind::Contradicts,
    )
    .unwrap();

    assert!(matches!(
        belief.target(),
        EvidenceTarget::BeliefRevision { .. }
    ));
    assert!(matches!(
        value.target(),
        EvidenceTarget::ValueRevision { .. }
    ));
    assert_round_trip(&belief);
    assert_round_trip(&value);
}

#[test]
fn user_authored_note_preserves_unicode_and_rejects_blank_values() {
    let evidence = EvidenceLink::new(
        EvidenceLinkId::new("evidence-含义").unwrap(),
        SelfSubjectId::new("self-用户").unwrap(),
        EvidenceSource::Memory(MemoryId::new("memory-经历").unwrap()),
        value_target(),
        EvidenceRelationKind::Supports,
        EvidenceProvenance::UserAuthored,
        Some("This choice reflects 自主与 responsibility 🌱".to_owned()),
    )
    .unwrap();

    assert_eq!(evidence.id().as_str(), "evidence-含义");
    assert_eq!(evidence.subject_id().as_str(), "self-用户");
    assert_eq!(
        evidence.user_note(),
        Some("This choice reflects 自主与 responsibility 🌱")
    );
    assert_round_trip(&evidence);

    for blank in ["", " ", "\t\n", "\u{3000}"] {
        assert!(EvidenceLink::new(
            EvidenceLinkId::new("evidence-blank").unwrap(),
            subject_id(),
            EvidenceSource::Observation(ObservationId::new("observation-1").unwrap()),
            belief_target(),
            EvidenceRelationKind::Supports,
            EvidenceProvenance::UserAuthored,
            Some(blank.to_owned()),
        )
        .is_err());
    }
}

#[test]
fn distinct_ids_allow_duplicate_assertions_without_weighting_behavior() {
    let source = EvidenceSource::Memory(MemoryId::new("memory-1").unwrap());
    let target = belief_target();
    let first = link(
        "evidence-1",
        source.clone(),
        target.clone(),
        EvidenceRelationKind::Supports,
    )
    .unwrap();
    let second = link("evidence-2", source, target, EvidenceRelationKind::Supports).unwrap();

    assert_ne!(first.id(), second.id());
    assert_eq!(first.source(), second.source());
    assert_eq!(first.target(), second.target());
    assert_eq!(first.relationship(), second.relationship());
    assert_eq!(first.provenance(), second.provenance());
}

#[test]
fn deserialization_revalidates_triplets_and_rejects_unapproved_variants() {
    let valid = serde_json::to_value(
        link(
            "evidence-1",
            EvidenceSource::Situation(SituationId::new("situation-1").unwrap()),
            belief_target(),
            EvidenceRelationKind::Contextualizes,
        )
        .unwrap(),
    )
    .unwrap();

    let mut illegal_triplet = valid.clone();
    illegal_triplet["relationship"] = json!("Supports");
    assert!(serde_json::from_value::<EvidenceLink>(illegal_triplet).is_err());

    let mut person_source = valid.clone();
    person_source["source"] = json!({"type": "PersonReference", "id": "person-1"});
    assert!(serde_json::from_value::<EvidenceLink>(person_source).is_err());

    let mut person_target = valid.clone();
    person_target["target"] = json!({"type": "PersonReference", "id": "person-1"});
    assert!(serde_json::from_value::<EvidenceLink>(person_target).is_err());

    let mut anchor_target = valid.clone();
    anchor_target["target"] = json!({"type": "Belief", "revision": {"id": "belief-1"}});
    assert!(serde_json::from_value::<EvidenceLink>(anchor_target).is_err());

    let mut unapproved_provenance = valid.clone();
    unapproved_provenance["provenance"] = json!("SystemProposed");
    assert!(serde_json::from_value::<EvidenceLink>(unapproved_provenance).is_err());

    let mut blank_source_id = valid.clone();
    blank_source_id["source"]["id"] = json!(" ");
    assert!(serde_json::from_value::<EvidenceLink>(blank_source_id).is_err());
}

#[test]
fn serialized_shape_has_no_scoring_inference_third_party_or_timestamp_fields() {
    let value = serde_json::to_value(
        link(
            "evidence-1",
            EvidenceSource::Thought(ThoughtId::new("thought-1").unwrap()),
            belief_target(),
            EvidenceRelationKind::Complicates,
        )
        .unwrap(),
    )
    .unwrap();
    let object = value.as_object().unwrap();

    for required in [
        "id",
        "subject_id",
        "source",
        "target",
        "relationship",
        "provenance",
        "user_note",
    ] {
        assert!(object.contains_key(required));
    }
    for forbidden in [
        "confidence",
        "weight",
        "score",
        "inferred",
        "reasoning",
        "rule_version",
        "person_reference",
        "automatic_confirmation",
        "created_at_ms",
        "occurred_at",
        "effective_at",
    ] {
        assert!(!object.contains_key(forbidden));
        let mut tampered: JsonValue = value.clone();
        tampered[forbidden] = json!(1);
        assert!(serde_json::from_value::<EvidenceLink>(tampered).is_err());
    }
}
