use serde::{Deserialize, Serialize};

use super::{
    primitives::RequiredText, EmotionId, EmotionIntensity, ObservationId, PersonReferenceId,
    SelfSubjectId, SituationId, ThoughtConfidence, ThoughtId, ValidationError,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelfSubject {
    id: SelfSubjectId,
    display_name: RequiredText,
}

impl SelfSubject {
    pub fn new(
        id: SelfSubjectId,
        display_name: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            display_name: RequiredText::new("display_name", display_name)?,
        })
    }

    pub fn id(&self) -> &SelfSubjectId {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        self.display_name.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonReference {
    id: PersonReferenceId,
    subject_id: SelfSubjectId,
    display_name: RequiredText,
    relationship_label: RequiredText,
    context_notes: Option<RequiredText>,
}

impl PersonReference {
    pub fn new(
        id: PersonReferenceId,
        subject_id: SelfSubjectId,
        display_name: impl Into<String>,
        relationship_label: impl Into<String>,
        context_notes: Option<String>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            subject_id,
            display_name: RequiredText::new("display_name", display_name)?,
            relationship_label: RequiredText::new("relationship_label", relationship_label)?,
            context_notes: context_notes
                .map(|notes| RequiredText::new("context_notes", notes))
                .transpose()?,
        })
    }

    pub fn id(&self) -> &PersonReferenceId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }

    pub fn display_name(&self) -> &str {
        self.display_name.as_str()
    }

    pub fn relationship_label(&self) -> &str {
        self.relationship_label.as_str()
    }

    pub fn context_notes(&self) -> Option<&str> {
        self.context_notes.as_ref().map(RequiredText::as_str)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Situation {
    id: SituationId,
    subject_id: SelfSubjectId,
    description: RequiredText,
}

impl Situation {
    pub fn new(
        id: SituationId,
        subject_id: SelfSubjectId,
        description: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            subject_id,
            description: RequiredText::new("description", description)?,
        })
    }

    pub fn id(&self) -> &SituationId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }

    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    id: ObservationId,
    subject_id: SelfSubjectId,
    situation_id: Option<SituationId>,
    content: RequiredText,
}

impl Observation {
    pub fn new(
        id: ObservationId,
        subject_id: SelfSubjectId,
        situation_id: Option<SituationId>,
        content: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            subject_id,
            situation_id,
            content: RequiredText::new("content", content)?,
        })
    }

    pub fn id(&self) -> &ObservationId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }

    pub fn situation_id(&self) -> Option<&SituationId> {
        self.situation_id.as_ref()
    }

    pub fn content(&self) -> &str {
        self.content.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thought {
    id: ThoughtId,
    subject_id: SelfSubjectId,
    situation_id: Option<SituationId>,
    content: RequiredText,
    confidence: Option<ThoughtConfidence>,
}

impl Thought {
    pub fn new(
        id: ThoughtId,
        subject_id: SelfSubjectId,
        situation_id: Option<SituationId>,
        content: impl Into<String>,
        confidence: Option<ThoughtConfidence>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            subject_id,
            situation_id,
            content: RequiredText::new("content", content)?,
            confidence,
        })
    }

    pub fn id(&self) -> &ThoughtId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }

    pub fn situation_id(&self) -> Option<&SituationId> {
        self.situation_id.as_ref()
    }

    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    pub fn confidence(&self) -> Option<ThoughtConfidence> {
        self.confidence
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emotion {
    id: EmotionId,
    subject_id: SelfSubjectId,
    situation_id: Option<SituationId>,
    label: RequiredText,
    intensity: EmotionIntensity,
}

impl Emotion {
    pub fn new(
        id: EmotionId,
        subject_id: SelfSubjectId,
        situation_id: Option<SituationId>,
        label: impl Into<String>,
        intensity: EmotionIntensity,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            subject_id,
            situation_id,
            label: RequiredText::new("label", label)?,
            intensity,
        })
    }

    pub fn id(&self) -> &EmotionId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }

    pub fn situation_id(&self) -> Option<&SituationId> {
        self.situation_id.as_ref()
    }

    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    pub fn intensity(&self) -> EmotionIntensity {
        self.intensity
    }
}
