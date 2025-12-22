use thiserror::Error;
use typed_builder::TypedBuilder;
use umem_core::Memory;

#[derive(Debug, Error)]
pub enum UpdateMemoryRequestError {
    #[error("one of vector or memory should be passed")]
    EmptyUpdate,

    #[error("vector id cannot be whitespace")]
    InvalidVectorId,
}

#[derive(TypedBuilder)]
pub struct UpdateMemoryRequest {
    pub vector_id: String,
    #[builder(default = None)]
    pub vector: Option<Vec<f32>>,
    #[builder(default = None)]
    pub memory: Option<Memory>,
}

impl UpdateMemoryRequest {
    pub fn validate(&self) -> Result<(), UpdateMemoryRequestError> {
        if self.vector.is_none() && self.memory.is_none() {
            return Err(UpdateMemoryRequestError::EmptyUpdate);
        }
        if self.vector_id.trim().is_empty() {
            return Err(UpdateMemoryRequestError::InvalidVectorId);
        }
        Ok(())
    }
}
