use serde::{Deserialize, Serialize};

use super::{
    primitives::RequiredText, BeliefEndorsement, BeliefId, BeliefRevisionId, RevisionNumber,
    SelfSubjectId, ValidationError, ValueId, ValueImportance, ValueRevisionId,
};

/// Provenance of a canonical user-authored revision, never a system inference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevisionOrigin {
    /// The first canonical user-authored state.
    InitialUserEntry,
    /// The user's actual commitment changed over time.
    UserUpdate,
    /// The stored representation was incorrect; this is not psychological change.
    UserCorrection,
}

/// Identity and ownership of a durable user-endorsed proposition.
/// Content lives in revisions; this anchor does not require a stored initial revision.
///
/// A contextual person reference cannot own a Belief:
/// ```compile_fail
/// use nous_lib::domain::{Belief, BeliefId, PersonReferenceId};
/// Belief::new(BeliefId::new("belief-1").unwrap(), PersonReferenceId::new("person-1").unwrap());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Belief {
    id: BeliefId,
    subject_id: SelfSubjectId,
}

impl Belief {
    pub fn new(id: BeliefId, subject_id: SelfSubjectId) -> Self {
        Self { id, subject_id }
    }

    pub fn id(&self) -> &BeliefId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }
}

/// Immutable local snapshot. Ownership is inherited through `belief_id`.
/// Parent existence, initial-revision existence and history sequencing are not checked here.
/// Record later changes as new revisions; semantic continuity requires the user's judgment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeliefRevision {
    id: BeliefRevisionId,
    belief_id: BeliefId,
    revision_number: RevisionNumber,
    proposition: RequiredText,
    endorsement: Option<BeliefEndorsement>,
    change_note: Option<RequiredText>,
    origin: RevisionOrigin,
}

impl BeliefRevision {
    pub fn new(
        id: BeliefRevisionId,
        belief_id: BeliefId,
        revision_number: RevisionNumber,
        proposition: impl Into<String>,
        endorsement: Option<BeliefEndorsement>,
        change_note: Option<String>,
        origin: RevisionOrigin,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            belief_id,
            revision_number,
            proposition: RequiredText::new("proposition", proposition)?,
            endorsement,
            change_note: change_note
                .map(|note| RequiredText::new("change_note", note))
                .transpose()?,
            origin,
        })
    }

    pub fn id(&self) -> &BeliefRevisionId {
        &self.id
    }

    pub fn belief_id(&self) -> &BeliefId {
        &self.belief_id
    }

    pub fn revision_number(&self) -> RevisionNumber {
        self.revision_number
    }

    pub fn proposition(&self) -> &str {
        self.proposition.as_str()
    }

    /// User-reported endorsement, not truth probability or system confidence.
    pub fn endorsement(&self) -> Option<BeliefEndorsement> {
        self.endorsement
    }

    pub fn change_note(&self) -> Option<&str> {
        self.change_note.as_ref().map(RequiredText::as_str)
    }

    pub fn origin(&self) -> RevisionOrigin {
        self.origin
    }
}

/// Identity and ownership of an enduring orientation or priority.
/// Label and importance live in revisions, not in this anchor.
///
/// A contextual person reference cannot own a Value:
/// ```compile_fail
/// use nous_lib::domain::{Value, ValueId, PersonReferenceId};
/// Value::new(ValueId::new("value-1").unwrap(), PersonReferenceId::new("person-1").unwrap());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Value {
    id: ValueId,
    subject_id: SelfSubjectId,
}

impl Value {
    pub fn new(id: ValueId, subject_id: SelfSubjectId) -> Self {
        Self { id, subject_id }
    }

    pub fn id(&self) -> &ValueId {
        &self.id
    }

    pub fn subject_id(&self) -> &SelfSubjectId {
        &self.subject_id
    }
}

/// Immutable local snapshot. Ownership is inherited through `value_id`.
/// Parent existence, initial-revision existence and history sequencing are not checked here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueRevision {
    id: ValueRevisionId,
    value_id: ValueId,
    revision_number: RevisionNumber,
    label: RequiredText,
    importance: Option<ValueImportance>,
    change_note: Option<RequiredText>,
    origin: RevisionOrigin,
}

impl ValueRevision {
    pub fn new(
        id: ValueRevisionId,
        value_id: ValueId,
        revision_number: RevisionNumber,
        label: impl Into<String>,
        importance: Option<ValueImportance>,
        change_note: Option<String>,
        origin: RevisionOrigin,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            value_id,
            revision_number,
            label: RequiredText::new("label", label)?,
            importance,
            change_note: change_note
                .map(|note| RequiredText::new("change_note", note))
                .transpose()?,
            origin,
        })
    }

    pub fn id(&self) -> &ValueRevisionId {
        &self.id
    }

    pub fn value_id(&self) -> &ValueId {
        &self.value_id
    }

    pub fn revision_number(&self) -> RevisionNumber {
        self.revision_number
    }

    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// User-reported salience, not moral worth or system ranking.
    pub fn importance(&self) -> Option<ValueImportance> {
        self.importance
    }

    pub fn change_note(&self) -> Option<&str> {
        self.change_note.as_ref().map(RequiredText::as_str)
    }

    pub fn origin(&self) -> RevisionOrigin {
        self.origin
    }
}
