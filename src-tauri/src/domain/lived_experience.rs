use serde::{Deserialize, Serialize};

use super::{
    primitives::RequiredText, DecisionId, MemoryId, OutcomeId, SelfSubjectId, SituationId,
    ValidationError,
};

/// A retrospectively selected, user-described past experience.
///
/// The optional Situation is a typed contextual link only. This domain record does not create,
/// load, or validate ownership of a Situation and does not treat recollection as objective truth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    id: MemoryId,
    subject_id: SelfSubjectId,
    situation_id: Option<SituationId>,
    description: RequiredText,
    user_meaning: Option<RequiredText>,
}

impl Memory {
    pub fn new(
        id: MemoryId,
        subject_id: SelfSubjectId,
        situation_id: Option<SituationId>,
        description: impl Into<String>,
        user_meaning: Option<String>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            subject_id,
            situation_id,
            description: RequiredText::new("description", description)?,
            user_meaning: user_meaning
                .map(|meaning| RequiredText::new("user_meaning", meaning))
                .transpose()?,
        })
    }

    pub fn id(&self) -> &MemoryId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }

    pub fn situation_id(&self) -> Option<&SituationId> {
        self.situation_id.as_ref()
    }

    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    pub fn user_meaning(&self) -> Option<&str> {
        self.user_meaning.as_ref().map(RequiredText::as_str)
    }
}

/// A choice the current SelfSubject reports having made in optional context.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    id: DecisionId,
    subject_id: SelfSubjectId,
    situation_id: Option<SituationId>,
    description: RequiredText,
}

impl Decision {
    pub fn new(
        id: DecisionId,
        subject_id: SelfSubjectId,
        situation_id: Option<SituationId>,
        description: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            subject_id,
            situation_id,
            description: RequiredText::new("description", description)?,
        })
    }

    pub fn id(&self) -> &DecisionId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }

    pub fn situation_id(&self) -> Option<&SituationId> {
        self.situation_id.as_ref()
    }

    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}

/// What the current SelfSubject reports happened after one Decision.
///
/// The typed relationship is mandatory. Same-subject parent ownership is a later persistence
/// invariant and is deliberately not simulated by this pure domain record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    id: OutcomeId,
    subject_id: SelfSubjectId,
    decision_id: DecisionId,
    description: RequiredText,
}

impl Outcome {
    pub fn new(
        id: OutcomeId,
        subject_id: SelfSubjectId,
        decision_id: DecisionId,
        description: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            subject_id,
            decision_id,
            description: RequiredText::new("description", description)?,
        })
    }

    pub fn id(&self) -> &OutcomeId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }

    pub fn decision_id(&self) -> &DecisionId {
        &self.decision_id
    }

    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}
