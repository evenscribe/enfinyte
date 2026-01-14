use std::sync::Arc;

use super::{MemoryController, MemoryControllerError};
use thiserror::Error;
use umem_core::Memory;
use umem_vector_store::VectorStoreError;

#[derive(Debug, Error)]
pub enum GetMemoryError {
    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("vector id cannot be empty or whitespace")]
    EmptyVectorId,
}

impl MemoryController {
    pub async fn get(&self, id: String) -> Result<Memory, MemoryControllerError> {
        Ok(self.get_impl(id).await?)
    }

    async fn get_impl(&self, id: String) -> Result<Memory, GetMemoryError> {
        let vector_store = Arc::clone(&self.vector_store);
        Ok(vector_store.get(id.as_str()).await?)
    }
}
