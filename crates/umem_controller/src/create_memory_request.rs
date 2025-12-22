use thiserror::Error;
use typed_builder::TypedBuilder;
use umem_core::{Memory, MemoryError};

#[derive(Debug, Error)]
pub enum CreateMemoryRequestError {
    #[error("At least one of user_id, agent_id, or run_id must be set")]
    MissingContext,

    #[error("Content cannot be empty or whitespace")]
    MissingContent,

    #[error("memory validation failed with: {0}")]
    MemoryError(#[from] MemoryError),
}

#[derive(TypedBuilder)]
pub struct CreateMemoryRequest {
    #[builder(default = None)]
    user_id: Option<String>,
    #[builder(default = None)]
    agent_id: Option<String>,
    #[builder(default = None)]
    run_id: Option<String>,
    raw_content: String,
}

impl CreateMemoryRequest {
    pub fn validate(&self) -> Result<(), CreateMemoryRequestError> {
        if self.user_id.is_none() && self.agent_id.is_none() && self.run_id.is_none() {
            return Err(CreateMemoryRequestError::MissingContext);
        }

        if self.raw_content.trim().is_empty() {
            return Err(CreateMemoryRequestError::MissingContent);
        }

        Ok(())
    }

    pub fn build(self) -> Result<Memory, CreateMemoryRequestError> {
        self.validate()?;
        // TODO: update later
        Ok(Memory::gen_dummy()?)
    }
}
