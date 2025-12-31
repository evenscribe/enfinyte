use super::{MemoryController, MemoryControllerError};
use thiserror::Error;
use umem_vector_store::{VectorStore, VectorStoreError};

#[derive(Debug, Error)]
pub enum DeleteMemoryError {
    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),
}

impl MemoryController {
    pub async fn delete(id: String) -> Result<(), MemoryControllerError> {
        Ok(Self::delete_impl(id).await?)
    }

    async fn delete_impl(id: String) -> Result<(), DeleteMemoryError> {
        let vector_store = VectorStore::get_store().await?;
        Ok(vector_store.delete(id.as_str()).await?)
    }
}
