use thiserror::Error;
use typed_builder::TypedBuilder;
use umem_core::Memory;

#[derive(Debug, Error)]
pub enum CreateMemoryRequestError {
    #[error("At least one of user_id, agent_id, or run_id must be set")]
    MissingContext,

    #[error("Content cannot be empty or whitespace")]
    MissingContent,
}

#[derive(TypedBuilder)]
pub struct CreateMemoryRequest {
    #[builder(default, setter(strip_option))]
    user_id: Option<String>,
    #[builder(default, setter(strip_option))]
    agent_id: Option<String>,
    #[builder(default, setter(strip_option))]
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
        todo!()
    }
}
