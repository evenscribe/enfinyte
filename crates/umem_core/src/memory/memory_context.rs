use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MemoryContextError {
    #[error("context must contain at least one identifier")]
    EmptyContext,

    #[error("{field} must not be empty or whitespace")]
    EmptyField { field: &'static str },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryContext {
    user_id: Option<String>,
    agent_id: Option<String>,
    run_id: Option<String>,
}

impl MemoryContext {
    pub fn new(
        user_id: Option<String>,
        agent_id: Option<String>,
        run_id: Option<String>,
    ) -> Result<Self, MemoryContextError> {
        let user_id = normalize("user_id", user_id)?;
        let agent_id = normalize("agent_id", agent_id)?;
        let run_id = normalize("run_id", run_id)?;

        if user_id.is_none() && agent_id.is_none() && run_id.is_none() {
            return Err(MemoryContextError::EmptyContext);
        }

        Ok(Self {
            user_id,
            agent_id,
            run_id,
        })
    }

    pub fn validate(&self) -> Result<(), MemoryContextError> {
        if self.user_id.is_none() && self.agent_id.is_none() && self.run_id.is_none() {
            return Err(MemoryContextError::EmptyContext);
        }

        Ok(())
    }

    pub fn for_user(user_id: impl Into<String>) -> Result<Self, MemoryContextError> {
        Self::new(Some(user_id.into()), None, None)
    }

    pub fn for_agent(agent_id: impl Into<String>) -> Result<Self, MemoryContextError> {
        Self::new(None, Some(agent_id.into()), None)
    }

    pub fn for_run(run_id: impl Into<String>) -> Result<Self, MemoryContextError> {
        Self::new(None, None, Some(run_id.into()))
    }

    pub fn is_partial(&self) -> bool {
        let count = self.user_id.is_some() as u8
            + self.agent_id.is_some() as u8
            + self.run_id.is_some() as u8;

        count < 3
    }

    pub fn has_user(&self) -> bool {
        self.user_id.is_some()
    }

    pub fn has_agent(&self) -> bool {
        self.agent_id.is_some()
    }

    pub fn has_run(&self) -> bool {
        self.run_id.is_some()
    }

    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    pub fn agent_id(&self) -> Option<&str> {
        self.agent_id.as_deref()
    }

    pub fn run_id(&self) -> Option<&str> {
        self.run_id.as_deref()
    }
}

fn normalize(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, MemoryContextError> {
    match value {
        None => Ok(None),
        Some(v) => {
            let v = v.trim();
            if v.is_empty() {
                Err(MemoryContextError::EmptyField { field })
            } else {
                Ok(Some(v.to_string()))
            }
        }
    }
}
