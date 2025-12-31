use super::{MemoryController, MemoryControllerError};
use thiserror::Error;
use umem_core::{Memory, MemoryContext, MemoryContextError, Query};
use umem_vector_store::{VectorStore, VectorStoreError};

#[derive(Debug, Error)]
pub enum ListMemoryError {
    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("memory context action failed with: {0}")]
    MemoryContextError(#[from] MemoryContextError),
}

impl MemoryController {
    pub async fn list_for_user(user_id: String) -> Result<Vec<Memory>, MemoryControllerError> {
        Ok(Self::list_for_user_impl(user_id).await?)
    }

    async fn list_for_user_impl(user_id: String) -> Result<Vec<Memory>, ListMemoryError> {
        let vector_store = VectorStore::get_store().await?;
        let query = Query::builder()
            .context(MemoryContext::for_user(user_id)?)
            .limit(1000)
            .build();

        Ok(vector_store.list(query).await?)
    }

    pub async fn list_with_context(
        context: MemoryContext,
    ) -> Result<Vec<Memory>, MemoryControllerError> {
        Ok(Self::list_with_context_impl(context).await?)
    }

    async fn list_with_context_impl(
        context: MemoryContext,
    ) -> Result<Vec<Memory>, ListMemoryError> {
        let vector_store = VectorStore::get_store().await?;
        let query = Query::builder().context(context).limit(1000).build();

        Ok(vector_store.list(query).await?)
    }
}
