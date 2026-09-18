use serde::{Deserialize, Serialize};

use super::ValidationError;

fn validate_required_text(field: &'static str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::EmptyText { field });
    }

    Ok(())
}

macro_rules! define_id {
    ($name:ident, $field:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
                let value = value.into();
                validate_required_text($field, &value)?;
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = ValidationError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

define_id!(SelfSubjectId, "self_subject_id");
define_id!(PersonReferenceId, "person_reference_id");
define_id!(ObservationId, "observation_id");
define_id!(SituationId, "situation_id");
define_id!(ThoughtId, "thought_id");
define_id!(EmotionId, "emotion_id");

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(super) struct RequiredText(String);

impl RequiredText {
    pub(super) fn new(
        field: &'static str,
        value: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let value = value.into();
        validate_required_text(field, &value)?;
        Ok(Self(value))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RequiredText {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new("text", value)
    }
}

impl From<RequiredText> for String {
    fn from(value: RequiredText) -> Self {
        value.0
    }
}

macro_rules! define_percentage {
    ($name:ident, $field:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "u8", into = "u8")]
        pub struct $name(u8);

        impl $name {
            pub const MINIMUM: u8 = 0;
            pub const MAXIMUM: u8 = 100;

            pub fn new(value: u8) -> Result<Self, ValidationError> {
                if value > Self::MAXIMUM {
                    return Err(ValidationError::OutOfRange {
                        field: $field,
                        minimum: Self::MINIMUM,
                        maximum: Self::MAXIMUM,
                        actual: value,
                    });
                }

                Ok(Self(value))
            }

            pub fn value(self) -> u8 {
                self.0
            }
        }

        impl TryFrom<u8> for $name {
            type Error = ValidationError;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

define_percentage!(EmotionIntensity, "emotion_intensity");
define_percentage!(ThoughtConfidence, "thought_confidence");
