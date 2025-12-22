use serde::{Deserialize, Serialize};

use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("invalid lifecycle state: {input}")]
pub struct ParseLifecycleStateError {
    pub input: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum LifecycleState {
    #[default]
    Active,
    Archived,
}

impl FromStr for LifecycleState {
    type Err = ParseLifecycleStateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(ParseLifecycleStateError {
                input: s.to_string(),
            }),
        }
    }
}

impl LifecycleState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "Active",
            Self::Archived => "Archived",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, LifecycleState::Active)
    }

    pub fn is_archived(&self) -> bool {
        matches!(self, LifecycleState::Archived)
    }
}
