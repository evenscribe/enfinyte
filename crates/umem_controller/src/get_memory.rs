use super::{MemoryController, MemoryControllerError};
use thiserror::Error;
use umem_core::Memory;
use umem_vector_store::{VectorStore, VectorStoreError};

#[derive(Debug, Error)]
pub enum GetMemoryError {
    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("vector id cannot be empty or whitespace")]
    EmptyVectorId,
}

impl MemoryController {
    pub async fn get(id: String) -> Result<Memory, MemoryControllerError> {
        Ok(Self::get_impl(id).await?)
    }

    async fn get_impl(id: String) -> Result<Memory, GetMemoryError> {
        let vector_store = VectorStore::get_store().await?;
        Ok(vector_store.get(id.as_str()).await?)
    }
}
