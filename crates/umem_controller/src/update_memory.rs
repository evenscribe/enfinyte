use std::sync::Arc;

use super::{MemoryController, MemoryControllerError};
use thiserror::Error;
use typed_builder::TypedBuilder;
use umem_ai::EmbeddingModelError;
use umem_core::Memory;
use umem_vector_store::VectorStoreError;

#[derive(Debug, Error)]
pub enum UpdateMemoryRequestError {
    #[error("vector or memory should be passed")]
    EmptyUpdate,

    #[error("vector id cannot be empty or whitespace")]
    EmptyVectorId,
}

#[derive(Debug, Error)]
pub enum UpdateMemoryError {
    #[error("update request --build-- failed with: {0}")]
    UpdateMemoryRequestError(#[from] UpdateMemoryRequestError),

    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("embedder action failed with: {0}")]
    EmbeddingModelError(#[from] EmbeddingModelError),
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
            return Err(UpdateMemoryRequestError::EmptyVectorId);
        }
        Ok(())
    }
}

impl MemoryController {
    pub async fn update(&self, request: UpdateMemoryRequest) -> Result<(), MemoryControllerError> {
        Ok(self.update_impl(request).await?)
    }

    async fn update_impl(&self, request: UpdateMemoryRequest) -> Result<(), UpdateMemoryError> {
        let vector_store = Arc::clone(&self.vector_store);

        request.validate()?;

        let UpdateMemoryRequest {
            vector_id,
            vector,
            memory,
        } = request;

        Ok(vector_store
            .update(&vector_id, vector.as_deref(), memory.as_ref())
            .await?)
    }
}
