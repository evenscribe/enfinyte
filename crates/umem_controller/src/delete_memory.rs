use std::sync::Arc;

use super::{MemoryController, MemoryControllerError};
use thiserror::Error;
use umem_vector_store::VectorStoreError;

#[derive(Debug, Error)]
pub enum DeleteMemoryError {
    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),
}

impl MemoryController {
    pub async fn delete(&self, id: String) -> Result<(), MemoryControllerError> {
        Ok(self.delete_impl(id).await?)
    }

    async fn delete_impl(&self, id: String) -> Result<(), DeleteMemoryError> {
        let vector_store = Arc::clone(&self.vector_store);
        Ok(vector_store.delete(id.as_str()).await?)
    }
}
