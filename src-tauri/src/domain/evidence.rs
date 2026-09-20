use serde::{Deserialize, Deserializer, Serialize};

use super::{
    primitives::RequiredText, BeliefId, BeliefRevisionId, DecisionId, EmotionId, EvidenceLinkId,
    MemoryId, ObservationId, OutcomeId, SelfSubjectId, SituationId, ThoughtId, ValidationError,
    ValueId, ValueRevisionId,
};

/// A closed set of user-owned records that may be cited by an explicit EvidenceLink.
///
/// Source variants preserve epistemic type: an Observation remains a user-reported observation,
/// while a Thought remains an interpretation. Neither becomes objective proof by being linked.
///
/// `PersonReference` is deliberately not representable:
/// ```compile_fail
/// use nous_lib::domain::{EvidenceSource, PersonReferenceId};
/// let person = PersonReferenceId::new("person-1").unwrap();
/// let _ = EvidenceSource::PersonReference(person);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", deny_unknown_fields)]
pub enum EvidenceSource {
    Observation(ObservationId),
    Thought(ThoughtId),
    Emotion(EmotionId),
    Situation(SituationId),
    Memory(MemoryId),
    Decision(DecisionId),
    Outcome(OutcomeId),
}

impl EvidenceSource {
    fn kind_name(&self) -> &'static str {
        match self {
            Self::Observation(_) => "Observation",
            Self::Thought(_) => "Thought",
            Self::Emotion(_) => "Emotion",
            Self::Situation(_) => "Situation",
            Self::Memory(_) => "Memory",
            Self::Decision(_) => "Decision",
            Self::Outcome(_) => "Outcome",
        }
    }
}

/// An exact commitment revision, including its ownership anchor.
///
/// Evidence never targets a Belief or Value anchor by itself. The anchor ID is part of the
/// approved domain identity of the revision target and later enables structural ownership checks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "revision", deny_unknown_fields)]
pub enum EvidenceTarget {
    BeliefRevision {
        belief_id: BeliefId,
        revision_id: BeliefRevisionId,
    },
    ValueRevision {
        value_id: ValueId,
        revision_id: ValueRevisionId,
    },
}

impl EvidenceTarget {
    fn kind_name(&self) -> &'static str {
        match self {
            Self::BeliefRevision { .. } => "BeliefRevision",
            Self::ValueRevision { .. } => "ValueRevision",
        }
    }
}

/// The complete approved evidential relationship vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceRelationKind {
    /// The user explicitly asserts that the source bears in favor of the exact
    /// revision target without turning that assertion into proof.
    Supports,
    /// For a belief revision, the source conflicts with its proposition. For a
    /// value revision, the source conflicts or is in tension with its expressed
    /// orientation or priority; this never means that a value is "false."
    Contradicts,
    /// The user explicitly asserts that the source makes the exact revision's
    /// meaning or application less straightforward.
    Complicates,
    /// The user explicitly supplies the source as context for the exact
    /// revision without asserting support or conflict.
    Contextualizes,
}

impl EvidenceRelationKind {
    fn name(self) -> &'static str {
        match self {
            Self::Supports => "Supports",
            Self::Contradicts => "Contradicts",
            Self::Complicates => "Complicates",
            Self::Contextualizes => "Contextualizes",
        }
    }
}

/// Provenance for a durable EvidenceLink created through deliberate user intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceProvenance {
    UserAuthored,
}

/// An immutable user-authored assertion that one typed source bears on one exact commitment
/// revision. It is not proof, a score, a weight, an inference result, or a target mutation.
///
/// Distinct IDs may record the same semantic assertion. Repetition carries no additional
/// evidential weight, confidence, or automatic corroboration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EvidenceLink {
    id: EvidenceLinkId,
    subject_id: SelfSubjectId,
    source: EvidenceSource,
    target: EvidenceTarget,
    relationship: EvidenceRelationKind,
    provenance: EvidenceProvenance,
    user_note: Option<RequiredText>,
}

impl EvidenceLink {
    pub fn new(
        id: EvidenceLinkId,
        subject_id: SelfSubjectId,
        source: EvidenceSource,
        target: EvidenceTarget,
        relationship: EvidenceRelationKind,
        provenance: EvidenceProvenance,
        user_note: Option<String>,
    ) -> Result<Self, ValidationError> {
        if !Self::is_legal_triplet(&source, relationship, &target) {
            return Err(ValidationError::InvalidEvidenceRelation {
                source: source.kind_name(),
                relationship: relationship.name(),
                target: target.kind_name(),
            });
        }

        Ok(Self {
            id,
            subject_id,
            source,
            target,
            relationship,
            provenance,
            user_note: user_note
                .map(|note| RequiredText::new("user_note", note))
                .transpose()?,
        })
    }

    fn is_legal_triplet(
        source: &EvidenceSource,
        relationship: EvidenceRelationKind,
        target: &EvidenceTarget,
    ) -> bool {
        matches!(
            (source, relationship, target),
            (
                EvidenceSource::Observation(_)
                    | EvidenceSource::Thought(_)
                    | EvidenceSource::Memory(_)
                    | EvidenceSource::Decision(_)
                    | EvidenceSource::Outcome(_),
                EvidenceRelationKind::Supports
                    | EvidenceRelationKind::Contradicts
                    | EvidenceRelationKind::Complicates
                    | EvidenceRelationKind::Contextualizes,
                EvidenceTarget::BeliefRevision { .. } | EvidenceTarget::ValueRevision { .. },
            ) | (
                EvidenceSource::Situation(_) | EvidenceSource::Emotion(_),
                EvidenceRelationKind::Contextualizes,
                EvidenceTarget::BeliefRevision { .. } | EvidenceTarget::ValueRevision { .. },
            )
        )
    }

    pub fn id(&self) -> &EvidenceLinkId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }

    pub fn source(&self) -> &EvidenceSource {
        &self.source
    }

    pub fn target(&self) -> &EvidenceTarget {
        &self.target
    }

    pub fn relationship(&self) -> EvidenceRelationKind {
        self.relationship
    }

    pub fn provenance(&self) -> EvidenceProvenance {
        self.provenance
    }

    pub fn user_note(&self) -> Option<&str> {
        self.user_note.as_ref().map(RequiredText::as_str)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceLinkData {
    id: EvidenceLinkId,
    subject_id: SelfSubjectId,
    source: EvidenceSource,
    target: EvidenceTarget,
    relationship: EvidenceRelationKind,
    provenance: EvidenceProvenance,
    user_note: Option<String>,
}

impl<'de> Deserialize<'de> for EvidenceLink {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = EvidenceLinkData::deserialize(deserializer)?;
        Self::new(
            data.id,
            data.subject_id,
            data.source,
            data.target,
            data.relationship,
            data.provenance,
            data.user_note,
        )
        .map_err(serde::de::Error::custom)
    }
}
