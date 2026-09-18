use std::any::TypeId;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value as JsonValue};

use super::*;

fn belief_revision() -> BeliefRevision {
    BeliefRevision::new(
        BeliefRevisionId::new("belief-revision-1").unwrap(),
        BeliefId::new("belief-1").unwrap(),
        RevisionNumber::new(1).unwrap(),
        "I can learn through practice.",
        Some(BeliefEndorsement::new(80).unwrap()),
        None,
        RevisionOrigin::InitialUserEntry,
    )
    .unwrap()
}

fn value_revision() -> ValueRevision {
    ValueRevision::new(
        ValueRevisionId::new("value-revision-1").unwrap(),
        ValueId::new("value-1").unwrap(),
        RevisionNumber::new(1).unwrap(),
        "Learning",
        Some(ValueImportance::new(90).unwrap()),
        None,
        RevisionOrigin::InitialUserEntry,
    )
    .unwrap()
}

fn assert_round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let encoded = serde_json::to_string(value).unwrap();
    assert_eq!(*value, serde_json::from_str::<T>(&encoded).unwrap());
}

#[test]
fn constructs_user_owned_commitments_and_initial_revisions() {
    let subject = SelfSubjectId::new("self-1").unwrap();
    let belief = Belief::new(BeliefId::new("belief-1").unwrap(), subject.clone());
    let value = Value::new(ValueId::new("value-1").unwrap(), subject.clone());
    let belief_state = belief_revision();
    let value_state = value_revision();

    assert_eq!(belief.subject_id(), &subject);
    assert_eq!(value.subject_id(), &subject);
    assert_eq!(belief_state.belief_id(), belief.id());
    assert_eq!(value_state.value_id(), value.id());
    assert_eq!(belief_state.id().as_str(), "belief-revision-1");
    assert_eq!(value_state.id().as_str(), "value-revision-1");
    assert_eq!(belief_state.revision_number().value(), 1);
    assert_eq!(value_state.revision_number().value(), 1);
    assert_eq!(belief_state.proposition(), "I can learn through practice.");
    assert_eq!(value_state.label(), "Learning");
    assert_eq!(belief_state.endorsement().unwrap().value(), 80);
    assert_eq!(value_state.importance().unwrap().value(), 90);
    assert_eq!(belief_state.change_note(), None);
    assert_eq!(value_state.change_note(), None);
    assert_eq!(belief_state.origin(), RevisionOrigin::InitialUserEntry);
    assert_eq!(value_state.origin(), RevisionOrigin::InitialUserEntry);
    assert_round_trip(&belief);
    assert_round_trip(&value);
    assert_round_trip(&belief_state);
    assert_round_trip(&value_state);
}

#[test]
fn validates_all_commitment_identifiers() {
    for invalid in ["", " ", "\t\r\n", "\u{3000}"] {
        assert!(BeliefId::new(invalid).is_err());
        assert!(BeliefRevisionId::new(invalid).is_err());
        assert!(ValueId::new(invalid).is_err());
        assert!(ValueRevisionId::new(invalid).is_err());
    }
    assert_round_trip(&BeliefId::new("信念-1").unwrap());
    assert_round_trip(&BeliefRevisionId::new("信念版本-1").unwrap());
    assert_round_trip(&ValueId::new("价值-1").unwrap());
    assert_round_trip(&ValueRevisionId::new("价值版本-1").unwrap());
}

#[test]
fn rejects_blank_propositions_labels_and_supplied_change_notes() {
    for blank in ["", " ", "\t\n", "\u{3000}"] {
        for (text, note) in [(blank, None), ("Valid", Some(blank.to_owned()))] {
            assert!(BeliefRevision::new(
                BeliefRevisionId::new("br").unwrap(),
                BeliefId::new("b").unwrap(),
                RevisionNumber::new(1).unwrap(),
                text,
                None,
                note.clone(),
                RevisionOrigin::InitialUserEntry,
            )
            .is_err());
            assert!(ValueRevision::new(
                ValueRevisionId::new("vr").unwrap(),
                ValueId::new("v").unwrap(),
                RevisionNumber::new(1).unwrap(),
                text,
                None,
                note,
                RevisionOrigin::InitialUserEntry,
            )
            .is_err());
        }
    }
}

#[test]
fn commitment_numeric_bounds_are_validated() {
    for number in [0, 100] {
        assert_eq!(BeliefEndorsement::new(number).unwrap().value(), number);
        assert_eq!(ValueImportance::new(number).unwrap().value(), number);
        assert_round_trip(&BeliefEndorsement::new(number).unwrap());
        assert_round_trip(&ValueImportance::new(number).unwrap());
    }
    for number in [101, u8::MAX] {
        assert!(BeliefEndorsement::new(number).is_err());
        assert!(ValueImportance::new(number).is_err());
    }
    assert_eq!(
        RevisionNumber::new(0),
        Err(ValidationError::ZeroRevisionNumber)
    );
    assert_eq!(
        ValidationError::ZeroRevisionNumber.to_string(),
        "revision_number must be positive"
    );
    for number in [1, 2, u32::MAX] {
        assert_eq!(RevisionNumber::new(number).unwrap().value(), number);
        assert_round_trip(&RevisionNumber::new(number).unwrap());
    }
}

#[test]
fn semantic_numeric_types_are_distinct() {
    assert_ne!(
        TypeId::of::<BeliefEndorsement>(),
        TypeId::of::<ValueImportance>()
    );
    assert_ne!(
        TypeId::of::<BeliefEndorsement>(),
        TypeId::of::<ThoughtConfidence>()
    );
    assert_ne!(
        TypeId::of::<ValueImportance>(),
        TypeId::of::<ThoughtConfidence>()
    );
}

#[test]
fn preserves_unicode_content_and_optional_unquantified_states() {
    for text in [
        "Learning matters",
        "学习很重要。",
        "Learning 学习 matters 🌱",
    ] {
        let belief = BeliefRevision::new(
            BeliefRevisionId::new("br").unwrap(),
            BeliefId::new("b").unwrap(),
            RevisionNumber::new(1).unwrap(),
            text,
            None,
            Some(text.to_owned()),
            RevisionOrigin::InitialUserEntry,
        )
        .unwrap();
        let value = ValueRevision::new(
            ValueRevisionId::new("vr").unwrap(),
            ValueId::new("v").unwrap(),
            RevisionNumber::new(1).unwrap(),
            text,
            None,
            Some(text.to_owned()),
            RevisionOrigin::InitialUserEntry,
        )
        .unwrap();
        assert_eq!(belief.proposition(), text);
        assert_eq!(value.label(), text);
        assert_eq!(belief.endorsement(), None);
        assert_eq!(value.importance(), None);
        assert_eq!(belief.change_note(), Some(text));
        assert_eq!(value.change_note(), Some(text));
        assert_round_trip(&belief);
        assert_round_trip(&value);
    }
}

#[test]
fn revision_origins_round_trip_and_reject_inference() {
    for (origin, name) in [
        (RevisionOrigin::InitialUserEntry, "InitialUserEntry"),
        (RevisionOrigin::UserUpdate, "UserUpdate"),
        (RevisionOrigin::UserCorrection, "UserCorrection"),
    ] {
        assert_eq!(serde_json::to_value(origin).unwrap(), json!(name));
        assert_round_trip(&origin);
    }
    assert_ne!(RevisionOrigin::UserUpdate, RevisionOrigin::UserCorrection);
    assert!(serde_json::from_value::<RevisionOrigin>(json!("SystemInference")).is_err());
}

#[test]
fn later_updates_and_corrections_leave_prior_snapshots_unchanged() {
    let initial_belief = belief_revision();
    let initial_value = value_revision();
    let belief_before = serde_json::to_value(&initial_belief).unwrap();
    let value_before = serde_json::to_value(&initial_value).unwrap();
    let mut beliefs = vec![initial_belief];
    let mut values = vec![initial_value];
    for (number, origin) in [
        (2, RevisionOrigin::UserUpdate),
        (3, RevisionOrigin::UserCorrection),
    ] {
        beliefs.push(
            BeliefRevision::new(
                BeliefRevisionId::new(format!("br-{number}")).unwrap(),
                beliefs[0].belief_id().clone(),
                RevisionNumber::new(number).unwrap(),
                "Practice helps me learn with support.",
                Some(BeliefEndorsement::new(0).unwrap()),
                Some("User-authored change".to_owned()),
                origin,
            )
            .unwrap(),
        );
        values.push(
            ValueRevision::new(
                ValueRevisionId::new(format!("vr-{number}")).unwrap(),
                values[0].value_id().clone(),
                RevisionNumber::new(number).unwrap(),
                "Learning with others",
                Some(ValueImportance::new(100).unwrap()),
                Some("User-authored change".to_owned()),
                origin,
            )
            .unwrap(),
        );
    }
    assert_eq!(serde_json::to_value(&beliefs[0]).unwrap(), belief_before);
    assert_eq!(serde_json::to_value(&values[0]).unwrap(), value_before);
    assert_eq!(beliefs[1].origin(), RevisionOrigin::UserUpdate);
    assert_eq!(beliefs[2].origin(), RevisionOrigin::UserCorrection);
    assert_eq!(values[1].origin(), RevisionOrigin::UserUpdate);
    assert_eq!(values[2].origin(), RevisionOrigin::UserCorrection);
    assert_round_trip(&beliefs);
    assert_round_trip(&values);
}

#[test]
fn local_revisions_do_not_require_persisted_parents_or_sequence_state() {
    // No parent or initial state is constructed. Equal positive numbers are locally valid;
    // history-level uniqueness and sequencing belong to Task 004.
    for origin in [RevisionOrigin::UserUpdate, RevisionOrigin::UserCorrection] {
        let belief = BeliefRevision::new(
            BeliefRevisionId::new("br").unwrap(),
            BeliefId::new("unresolved-belief").unwrap(),
            RevisionNumber::new(7).unwrap(),
            "A commitment",
            None,
            None,
            origin,
        )
        .unwrap();
        let value = ValueRevision::new(
            ValueRevisionId::new("vr").unwrap(),
            ValueId::new("unresolved-value").unwrap(),
            RevisionNumber::new(7).unwrap(),
            "Learning",
            None,
            None,
            origin,
        )
        .unwrap();
        assert_eq!(belief.revision_number().value(), 7);
        assert_eq!(value.revision_number().value(), 7);
    }
}

fn assert_invalid_field<T: DeserializeOwned>(valid: &JsonValue, field: &str, invalid: JsonValue) {
    let mut changed = valid.clone();
    changed[field] = invalid;
    assert!(
        serde_json::from_value::<T>(changed).is_err(),
        "accepted invalid {field}"
    );
}

#[test]
fn deserialization_cannot_bypass_revision_invariants() {
    let belief = serde_json::to_value(belief_revision()).unwrap();
    let value = serde_json::to_value(value_revision()).unwrap();
    for blank in [json!(""), json!(" \t\n"), json!("\u{3000}")] {
        for field in ["id", "belief_id", "proposition", "change_note"] {
            assert_invalid_field::<BeliefRevision>(&belief, field, blank.clone());
        }
        for field in ["id", "value_id", "label", "change_note"] {
            assert_invalid_field::<ValueRevision>(&value, field, blank.clone());
        }
    }
    for invalid in [
        json!(0),
        json!(-1),
        json!(1.5),
        json!(4294967296_u64),
        JsonValue::Null,
    ] {
        assert_invalid_field::<BeliefRevision>(&belief, "revision_number", invalid.clone());
        assert_invalid_field::<ValueRevision>(&value, "revision_number", invalid);
    }
    for invalid in [json!(101), json!(256), json!(-1), json!(0.5), json!("50")] {
        assert_invalid_field::<BeliefRevision>(&belief, "endorsement", invalid.clone());
        assert_invalid_field::<ValueRevision>(&value, "importance", invalid);
    }
    for invalid in [json!("SystemInference"), json!(""), JsonValue::Null] {
        assert_invalid_field::<BeliefRevision>(&belief, "origin", invalid.clone());
        assert_invalid_field::<ValueRevision>(&value, "origin", invalid);
    }
}

#[test]
fn ownership_is_on_anchors_only_and_extra_ownership_fields_are_rejected() {
    let subject = SelfSubjectId::new("self-1").unwrap();
    let belief =
        serde_json::to_value(Belief::new(BeliefId::new("b").unwrap(), subject.clone())).unwrap();
    let value = serde_json::to_value(Value::new(ValueId::new("v").unwrap(), subject)).unwrap();
    assert_eq!(belief, json!({"id": "b", "subject_id": "self-1"}));
    assert_eq!(value, json!({"id": "v", "subject_id": "self-1"}));
    for field in ["id", "subject_id"] {
        assert_invalid_field::<Belief>(&belief, field, json!(" "));
        assert_invalid_field::<Value>(&value, field, json!(" "));
    }
    let belief_state = serde_json::to_value(belief_revision()).unwrap();
    let value_state = serde_json::to_value(value_revision()).unwrap();
    for field in ["subject_id", "person_reference_id"] {
        assert!(belief_state.get(field).is_none());
        assert!(value_state.get(field).is_none());
        assert_invalid_field::<BeliefRevision>(&belief_state, field, json!("other"));
        assert_invalid_field::<ValueRevision>(&value_state, field, json!("other"));
    }
    assert_invalid_field::<Belief>(&belief, "person_reference_id", json!("person-1"));
    assert_invalid_field::<Value>(&value, "person_reference_id", json!("person-1"));
}
