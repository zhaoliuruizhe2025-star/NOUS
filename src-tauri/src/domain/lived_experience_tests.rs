use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value as JsonValue};

use super::*;

fn subject_id() -> SelfSubjectId {
    SelfSubjectId::new("self-1").expect("fixture subject id should be valid")
}

fn situation_id() -> SituationId {
    SituationId::new("situation-1").expect("fixture situation id should be valid")
}

fn assert_round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let encoded = serde_json::to_string(value).expect("domain value should serialize");
    let decoded = serde_json::from_str(&encoded).expect("domain value should deserialize");

    assert_eq!(value, &decoded);
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
fn lived_experience_ids_validate_and_serialize_as_strings() {
    let memory_id = MemoryId::new("memory-记忆-1").unwrap();
    let decision_id = DecisionId::new("decision-决定-1").unwrap();
    let outcome_id = OutcomeId::new("outcome-结果-1").unwrap();

    assert_eq!(memory_id.as_str(), "memory-记忆-1");
    assert_eq!(decision_id.as_str(), "decision-决定-1");
    assert_eq!(outcome_id.as_str(), "outcome-结果-1");
    assert_eq!(
        serde_json::to_value(&memory_id).unwrap(),
        json!("memory-记忆-1")
    );
    assert_eq!(
        serde_json::to_value(&decision_id).unwrap(),
        json!("decision-决定-1")
    );
    assert_eq!(
        serde_json::to_value(&outcome_id).unwrap(),
        json!("outcome-结果-1")
    );
    assert_round_trip(&memory_id);
    assert_round_trip(&decision_id);
    assert_round_trip(&outcome_id);

    for blank in ["", " ", "\t\r\n", "\u{3000}"] {
        assert!(MemoryId::new(blank).is_err());
        assert!(DecisionId::new(blank).is_err());
        assert!(OutcomeId::new(blank).is_err());
    }
}

#[test]
fn memory_preserves_optional_context_meaning_and_unicode() {
    let without_optional_values = Memory::new(
        MemoryId::new("memory-none").unwrap(),
        subject_id(),
        None,
        "I remembered learning to ride a bicycle.",
        None,
    )
    .unwrap();
    assert_eq!(without_optional_values.situation_id(), None);
    assert_eq!(without_optional_values.user_meaning(), None);

    let with_situation = Memory::new(
        MemoryId::new("memory-situation").unwrap(),
        subject_id(),
        Some(situation_id()),
        "小时候第一次独自上学。",
        None,
    )
    .unwrap();
    assert_eq!(with_situation.situation_id(), Some(&situation_id()));
    assert_eq!(with_situation.user_meaning(), None);

    let with_meaning = Memory::new(
        MemoryId::new("memory-meaning").unwrap(),
        subject_id(),
        None,
        "A conversation I still remember.",
        Some("  I learned 我可以 ask for help.  ".to_owned()),
    )
    .unwrap();
    assert_eq!(with_meaning.situation_id(), None);
    assert_eq!(
        with_meaning.user_meaning(),
        Some("  I learned 我可以 ask for help.  ")
    );

    let with_both = Memory::new(
        MemoryId::new("memory-both").unwrap(),
        subject_id(),
        Some(situation_id()),
        "毕业那天 felt both proud and uncertain 🌱",
        Some("It reminded me that endings can also be beginnings.".to_owned()),
    )
    .unwrap();
    assert_eq!(with_both.id().as_str(), "memory-both");
    assert_eq!(with_both.subject_id(), &subject_id());
    assert_eq!(with_both.situation_id(), Some(&situation_id()));
    assert_eq!(
        with_both.description(),
        "毕业那天 felt both proud and uncertain 🌱"
    );
    assert_eq!(
        with_both.user_meaning(),
        Some("It reminded me that endings can also be beginnings.")
    );

    for memory in [
        without_optional_values,
        with_situation,
        with_meaning,
        with_both,
    ] {
        assert_round_trip(&memory);
    }
}

#[test]
fn memory_rejects_blank_description_and_supplied_meaning() {
    for blank in ["", " ", "\t\n", "\u{3000}"] {
        assert!(Memory::new(
            MemoryId::new("memory-description").unwrap(),
            subject_id(),
            None,
            blank,
            None,
        )
        .is_err());
        assert!(Memory::new(
            MemoryId::new("memory-meaning").unwrap(),
            subject_id(),
            None,
            "A valid description",
            Some(blank.to_owned()),
        )
        .is_err());
    }
}

#[test]
fn decision_preserves_optional_context_and_unicode_and_rejects_blank_description() {
    let without_situation = Decision::new(
        DecisionId::new("decision-none").unwrap(),
        subject_id(),
        None,
        "I chose to wait before replying.",
    )
    .unwrap();
    assert_eq!(without_situation.situation_id(), None);

    let simplified_chinese = Decision::new(
        DecisionId::new("decision-zh").unwrap(),
        subject_id(),
        None,
        "我决定先听完对方的想法。",
    )
    .unwrap();
    assert_eq!(simplified_chinese.description(), "我决定先听完对方的想法。");

    let with_situation = Decision::new(
        DecisionId::new("decision-context").unwrap(),
        subject_id(),
        Some(situation_id()),
        "我决定接受 the new role 🚀",
    )
    .unwrap();
    assert_eq!(with_situation.id().as_str(), "decision-context");
    assert_eq!(with_situation.subject_id(), &subject_id());
    assert_eq!(with_situation.situation_id(), Some(&situation_id()));
    assert_eq!(with_situation.description(), "我决定接受 the new role 🚀");

    assert_round_trip(&without_situation);
    assert_round_trip(&simplified_chinese);
    assert_round_trip(&with_situation);

    for blank in ["", " ", "\t\n", "\u{3000}"] {
        assert!(Decision::new(
            DecisionId::new("decision-blank").unwrap(),
            subject_id(),
            None,
            blank,
        )
        .is_err());
    }
}

#[test]
fn outcome_preserves_required_decision_and_unicode_and_rejects_blank_description() {
    for (id, description) in [
        ("outcome-en", "I received a reply the next day."),
        ("outcome-zh", "后来我收到了录取通知。"),
        ("outcome-mixed", "The move was difficult, 但我适应了 🌏"),
    ] {
        let outcome = Outcome::new(
            OutcomeId::new(id).unwrap(),
            subject_id(),
            DecisionId::new("decision-1").unwrap(),
            description,
        )
        .unwrap();
        assert_eq!(outcome.id().as_str(), id);
        assert_eq!(outcome.subject_id(), &subject_id());
        assert_eq!(outcome.decision_id().as_str(), "decision-1");
        assert_eq!(outcome.description(), description);
        assert_round_trip(&outcome);
    }

    for blank in ["", " ", "\t\n", "\u{3000}"] {
        assert!(Outcome::new(
            OutcomeId::new("outcome-blank").unwrap(),
            subject_id(),
            DecisionId::new("decision-1").unwrap(),
            blank,
        )
        .is_err());
    }
}

#[test]
fn lived_experience_deserialization_cannot_bypass_validation_or_add_fields() {
    let memory = serde_json::to_value(
        Memory::new(
            MemoryId::new("memory-1").unwrap(),
            subject_id(),
            Some(situation_id()),
            "A remembered experience",
            Some("My own meaning".to_owned()),
        )
        .unwrap(),
    )
    .unwrap();
    let decision = serde_json::to_value(
        Decision::new(
            DecisionId::new("decision-1").unwrap(),
            subject_id(),
            Some(situation_id()),
            "I made a choice",
        )
        .unwrap(),
    )
    .unwrap();
    let outcome = serde_json::to_value(
        Outcome::new(
            OutcomeId::new("outcome-1").unwrap(),
            subject_id(),
            DecisionId::new("decision-1").unwrap(),
            "What I later reported happened",
        )
        .unwrap(),
    )
    .unwrap();

    for blank in [json!(""), json!(" \t\n"), json!("\u{3000}")] {
        for field in [
            "id",
            "subject_id",
            "situation_id",
            "description",
            "user_meaning",
        ] {
            assert_invalid_field::<Memory>(&memory, field, blank.clone());
        }
        for field in ["id", "subject_id", "situation_id", "description"] {
            assert_invalid_field::<Decision>(&decision, field, blank.clone());
        }
        for field in ["id", "subject_id", "decision_id", "description"] {
            assert_invalid_field::<Outcome>(&outcome, field, blank.clone());
        }
    }

    for (value, field) in [
        (&memory, "person_reference_id"),
        (&decision, "person_reference_id"),
        (&outcome, "person_reference_id"),
        (&memory, "created_at_ms"),
        (&decision, "created_at_ms"),
        (&outcome, "created_at_ms"),
    ] {
        assert!(value.get(field).is_none());
    }
    assert_invalid_field::<Memory>(&memory, "person_reference_id", json!("person-1"));
    assert_invalid_field::<Decision>(&decision, "person_reference_id", json!("person-1"));
    assert_invalid_field::<Outcome>(&outcome, "person_reference_id", json!("person-1"));
    assert_invalid_field::<Memory>(&memory, "created_at_ms", json!(1));
    assert_invalid_field::<Decision>(&decision, "created_at_ms", json!(1));
    assert_invalid_field::<Outcome>(&outcome, "created_at_ms", json!(1));
}
