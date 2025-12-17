use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Serialize, Deserialize)]
pub struct Provenance {
    pub origin: ProvenanceOrigin,
    pub method: ProvenanceMethod,
}

impl Provenance {
    pub fn direct_user() -> Self {
        Self {
            origin: ProvenanceOrigin::User,
            method: ProvenanceMethod::Direct,
        }
    }

    pub fn direct_agent() -> Self {
        Self {
            origin: ProvenanceOrigin::Agent,
            method: ProvenanceMethod::Direct,
        }
    }

    pub fn validate(&self) -> Result<(), ProvenanceMethodError> {
        self.method.validate()
    }
}

#[derive(Serialize, Deserialize)]
pub enum ProvenanceOrigin {
    User,
    Agent,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("invalid provenance origin: {input}")]
pub struct ParseProvenanceOriginError {
    pub input: String,
}

impl FromStr for ProvenanceOrigin {
    type Err = ParseProvenanceOriginError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "user" => Ok(Self::User),
            "agent" => Ok(Self::Agent),
            _ => Err(ParseProvenanceOriginError {
                input: s.to_string(),
            }),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ProvenanceMethod {
    Direct,
    Extracted { model: String, prompt: String },
    Summarized { model: String },
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ProvenanceMethodError {
    #[error("model name cannot be empty")]
    EmptyModel,
    #[error("prompt cannot be empty for extracted provenance")]
    EmptyPrompt,
}

impl ProvenanceMethod {
    pub fn validate(&self) -> Result<(), ProvenanceMethodError> {
        match self {
            ProvenanceMethod::Direct => Ok(()),
            ProvenanceMethod::Extracted { model, prompt } => {
                if model.trim().is_empty() {
                    return Err(ProvenanceMethodError::EmptyModel);
                }
                if prompt.trim().is_empty() {
                    return Err(ProvenanceMethodError::EmptyPrompt);
                }
                Ok(())
            }
            ProvenanceMethod::Summarized { model } => {
                if model.trim().is_empty() {
                    return Err(ProvenanceMethodError::EmptyModel);
                }
                Ok(())
            }
        }
    }
}
