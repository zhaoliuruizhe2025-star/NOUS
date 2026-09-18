use serde::{de::DeserializeOwned, Serialize};

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
    let json = serde_json::to_string(value).expect("domain value should serialize");
    let decoded = serde_json::from_str(&json).expect("domain value should deserialize");

    assert_eq!(value, &decoded);
}

#[test]
fn constructs_valid_related_domain_records() {
    let subject = SelfSubject::new(subject_id(), "Current user").unwrap();
    let situation = Situation::new(
        situation_id(),
        subject.id().clone(),
        "Waiting for an important reply",
    )
    .unwrap();
    let observation = Observation::new(
        ObservationId::new("observation-1").unwrap(),
        subject.id().clone(),
        Some(situation.id().clone()),
        "No reply for five hours.",
    )
    .unwrap();
    let thought = Thought::new(
        ThoughtId::new("thought-1").unwrap(),
        subject.id().clone(),
        Some(situation.id().clone()),
        "They may not care about me.",
        Some(ThoughtConfidence::new(65).unwrap()),
    )
    .unwrap();
    let sadness = Emotion::new(
        EmotionId::new("emotion-1").unwrap(),
        subject.id().clone(),
        Some(situation.id().clone()),
        "sadness",
        EmotionIntensity::new(70).unwrap(),
    )
    .unwrap();
    let anxiety = Emotion::new(
        EmotionId::new("emotion-2").unwrap(),
        subject.id().clone(),
        Some(situation.id().clone()),
        "anxiety",
        EmotionIntensity::new(55).unwrap(),
    )
    .unwrap();

    assert_eq!(observation.situation_id(), Some(situation.id()));
    assert_eq!(thought.situation_id(), Some(situation.id()));
    assert_eq!(sadness.situation_id(), Some(situation.id()));
    assert_eq!(anxiety.situation_id(), Some(situation.id()));
    assert_eq!(observation.content(), "No reply for five hours.");
    assert_eq!(thought.content(), "They may not care about me.");
}

#[test]
fn constructs_thought_and_emotion_without_a_situation() {
    let thought = Thought::new(
        ThoughtId::new("thought-without-situation").unwrap(),
        subject_id(),
        None,
        "A thought can arise without a concrete situation.",
        None,
    )
    .unwrap();
    let emotion = Emotion::new(
        EmotionId::new("emotion-without-situation").unwrap(),
        subject_id(),
        None,
        "uneasy",
        EmotionIntensity::new(35).unwrap(),
    )
    .unwrap();

    assert_eq!(thought.situation_id(), None);
    assert_eq!(emotion.situation_id(), None);
}

#[test]
fn rejects_empty_and_whitespace_only_identifiers() {
    assert!(matches!(
        SelfSubjectId::new(""),
        Err(ValidationError::EmptyText {
            field: "self_subject_id"
        })
    ));
    assert!(PersonReferenceId::new("   ").is_err());
    assert!(ObservationId::new("\t\n").is_err());
    assert!(SituationId::new(" ").is_err());
    assert!(ThoughtId::new("\r\n").is_err());
    assert!(EmotionId::new("\t").is_err());
}

#[test]
fn rejects_empty_required_entity_text() {
    assert!(SelfSubject::new(subject_id(), "  ").is_err());
    assert!(PersonReference::new(
        PersonReferenceId::new("person-1").unwrap(),
        subject_id(),
        "Alex",
        " ",
        None,
    )
    .is_err());
    assert!(PersonReference::new(
        PersonReferenceId::new("person-1").unwrap(),
        subject_id(),
        "Alex",
        "friend",
        Some("\t".to_owned()),
    )
    .is_err());
    assert!(Situation::new(situation_id(), subject_id(), "\n").is_err());
    assert!(Observation::new(
        ObservationId::new("observation-1").unwrap(),
        subject_id(),
        None,
        " ",
    )
    .is_err());
    assert!(Thought::new(
        ThoughtId::new("thought-1").unwrap(),
        subject_id(),
        None,
        "\t",
        None,
    )
    .is_err());
    assert!(Emotion::new(
        EmotionId::new("emotion-1").unwrap(),
        subject_id(),
        None,
        " ",
        EmotionIntensity::new(50).unwrap(),
    )
    .is_err());
}

#[test]
fn validates_numeric_boundaries() {
    assert_eq!(EmotionIntensity::new(0).unwrap().value(), 0);
    assert_eq!(EmotionIntensity::new(100).unwrap().value(), 100);
    assert!(matches!(
        EmotionIntensity::new(101),
        Err(ValidationError::OutOfRange {
            field: "emotion_intensity",
            minimum: 0,
            maximum: 100,
            actual: 101,
        })
    ));

    assert_eq!(ThoughtConfidence::new(0).unwrap().value(), 0);
    assert_eq!(ThoughtConfidence::new(100).unwrap().value(), 100);
    assert!(ThoughtConfidence::new(101).is_err());
}

#[test]
fn supports_chinese_english_and_mixed_language_content() {
    let subject = SelfSubject::new(subject_id(), "李明 Li Ming").unwrap();
    let person = PersonReference::new(
        PersonReferenceId::new("person-朋友").unwrap(),
        subject.id().clone(),
        "小雨 Xiaoyu",
        "朋友 / friend",
        Some("大学时认识的朋友。We still talk weekly.".to_owned()),
    )
    .unwrap();
    let situation = Situation::new(
        SituationId::new("situation-考试").unwrap(),
        subject.id().clone(),
        "明天有考试，但我还没有复习完。",
    )
    .unwrap();
    let thought = Thought::new(
        ThoughtId::new("thought-混合").unwrap(),
        subject.id().clone(),
        Some(situation.id().clone()),
        "I can finish 一章 at a time.",
        None,
    )
    .unwrap();

    assert_eq!(subject.display_name(), "李明 Li Ming");
    assert_eq!(person.relationship_label(), "朋友 / friend");
    assert_eq!(situation.description(), "明天有考试，但我还没有复习完。");
    assert_eq!(thought.content(), "I can finish 一章 at a time.");
}

#[test]
fn person_reference_serialization_contains_context_not_a_psychological_profile() {
    let person = PersonReference::new(
        PersonReferenceId::new("person-1").unwrap(),
        subject_id(),
        "Alex",
        "friend",
        Some("Met through a shared class.".to_owned()),
    )
    .unwrap();

    let json = serde_json::to_value(&person).unwrap();
    let object = json.as_object().unwrap();

    assert_eq!(
        object.keys().map(String::as_str).collect::<Vec<_>>(),
        vec![
            "context_notes",
            "display_name",
            "id",
            "relationship_label",
            "subject_id",
        ]
    );
    for forbidden in [
        "beliefs",
        "values",
        "emotions",
        "diagnosis",
        "personality_traits",
        "motives",
        "future_behavior_predictions",
    ] {
        assert!(!object.contains_key(forbidden));
    }
}

#[test]
fn domain_entities_round_trip_through_json() {
    let subject = SelfSubject::new(subject_id(), "Current user").unwrap();
    let person = PersonReference::new(
        PersonReferenceId::new("person-1").unwrap(),
        subject.id().clone(),
        "Alex",
        "friend",
        None,
    )
    .unwrap();
    let situation = Situation::new(
        situation_id(),
        subject.id().clone(),
        "A mixed-language situation 情境",
    )
    .unwrap();
    let observation = Observation::new(
        ObservationId::new("observation-1").unwrap(),
        subject.id().clone(),
        Some(situation.id().clone()),
        "Observed detail 观察",
    )
    .unwrap();
    let thought = Thought::new(
        ThoughtId::new("thought-1").unwrap(),
        subject.id().clone(),
        Some(situation.id().clone()),
        "A tentative interpretation",
        Some(ThoughtConfidence::new(40).unwrap()),
    )
    .unwrap();
    let emotion = Emotion::new(
        EmotionId::new("emotion-1").unwrap(),
        subject.id().clone(),
        Some(situation.id().clone()),
        "期待",
        EmotionIntensity::new(60).unwrap(),
    )
    .unwrap();

    assert_round_trip(&subject);
    assert_round_trip(&person);
    assert_round_trip(&situation);
    assert_round_trip(&observation);
    assert_round_trip(&thought);
    assert_round_trip(&emotion);
}

#[test]
fn deserialization_cannot_bypass_domain_validation() {
    assert!(serde_json::from_str::<SelfSubject>(r#"{"id":"self-1","display_name":" "}"#).is_err());
    assert!(serde_json::from_str::<EmotionIntensity>("101").is_err());
    assert!(serde_json::from_str::<ThoughtConfidence>("101").is_err());
}
