use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {
    EmptyText {
        field: &'static str,
    },
    OutOfRange {
        field: &'static str,
        minimum: u8,
        maximum: u8,
        actual: u8,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyText { field } => write!(formatter, "{field} must not be empty"),
            Self::OutOfRange {
                field,
                minimum,
                maximum,
                actual,
            } => write!(
                formatter,
                "{field} must be between {minimum} and {maximum}, got {actual}"
            ),
        }
    }
}

impl Error for ValidationError {}
