use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("invalid memory kind: {input}")]
pub struct ParseMemoryKindError {
    pub input: String,
}

impl FromStr for MemoryKind {
    type Err = ParseMemoryKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "semantic" => Ok(Self::Semantic),
            "episodic" => Ok(Self::Episodic),
            "procedural" => Ok(Self::Procedural),
            "instruction" | "directive" => Ok(Self::Instruction),
            "relational" | "relation" => Ok(Self::Relational),
            "working" => Ok(Self::Working),
            "prospective" | "future" => Ok(Self::Prospective),
            s => Err(ParseMemoryKindError {
                input: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum MemoryKind {
    #[default]
    Semantic, // general knowledge
    Episodic,    // tempo-spatial
    Procedural,  // skill, habits, etc.
    Instruction, // explicit directives
    Relational,  // people, entities
    Working,     // temp current working memory
    Prospective, // future plans, etc.
}

impl MemoryKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            MemoryKind::Semantic => "Semantic",
            MemoryKind::Episodic => "Episodic",
            MemoryKind::Procedural => "Procedural",
            MemoryKind::Instruction => "Instruction",
            MemoryKind::Relational => "Relational",
            MemoryKind::Working => "Working",
            MemoryKind::Prospective => "Prospective",
        }
    }

    pub const fn all() -> &'static [MemoryKind] {
        &[
            MemoryKind::Semantic,
            MemoryKind::Episodic,
            MemoryKind::Procedural,
            MemoryKind::Instruction,
            MemoryKind::Relational,
            MemoryKind::Working,
            MemoryKind::Prospective,
        ]
    }

    pub fn is_transient(&self) -> bool {
        matches!(self, MemoryKind::Working)
    }
}
