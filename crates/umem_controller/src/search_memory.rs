use super::{MemoryController, MemoryControllerError};
use std::sync::Arc;
use thiserror::Error;
use tokio::{sync::AcquireError, task::JoinError};
use typed_builder::TypedBuilder;
use umem_core::{Memory, MemoryContext, MemoryContextError, Query};
use umem_embeddings::{Embedder, EmbedderBase, EmbedderError};
use umem_refine::{RefineError, Segmenter};
use umem_rerank::{RerankError, Reranker};
use umem_vector_store::{VectorStore, VectorStoreError};

#[derive(Debug, Error)]
pub enum SearchMemoryError {
    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("memory context action failed with: {0}")]
    MemoryContextError(#[from] MemoryContextError),

    #[error("embedder action failed with: {0}")]
    EmbedderError(#[from] EmbedderError),

    #[error("umem refine action failed with: {0}")]
    RefineError(#[from] RefineError),

    #[error("parallel search semaphore failed with: {0}")]
    AcquireError(#[from] AcquireError),

    #[error("parallel search join failed with: {0}")]
    JoinError(#[from] JoinError),

    #[error("rerank action failed with: {0}")]
    RerankError(#[from] RerankError),
}

#[derive(TypedBuilder, Default)]
pub struct SearchMemoryOptions {
    #[builder(default = None)]
    pub embedder: Option<Arc<dyn EmbedderBase + Send + Sync>>,
}

impl MemoryController {
    pub async fn search_for_user(
        user_id: String,
        query: String,
        options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, MemoryControllerError> {
        Ok(Self::search_for_user_impl(user_id, query, options).await?)
    }

    async fn search_for_user_impl(
        user_id: String,
        query: String,
        options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, SearchMemoryError> {
        let options = options.unwrap_or_default();
        let vector_store = VectorStore::get_store().await?;
        let embedder = options.embedder.unwrap_or(Embedder::get_embedder().await?);

        let vector = embedder.generate_embedding(query.as_str()).await?;

        let query = Query::builder()
            .vector(vector)
            .context(MemoryContext::for_user(user_id)?)
            .limit(1000)
            .build();

        Ok(vector_store.search(query).await?)
    }

    pub async fn search_with_context(
        context: MemoryContext,
        query: String,
        options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, MemoryControllerError> {
        Ok(Self::search_with_context_impl(context, query, options).await?)
    }

    async fn search_with_context_impl(
        context: MemoryContext,
        query: String,
        options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, SearchMemoryError> {
        let options = options.unwrap_or_default();
        let vector_store = VectorStore::get_store().await?;
        let embedder = options.embedder.unwrap_or(Embedder::get_embedder().await?);

        let vector = embedder.generate_embedding(query.as_str()).await?;

        let query = Query::builder()
            .vector(vector)
            .context(context)
            .limit(1000)
            .build();

        Ok(vector_store.search(query).await?)
    }

    pub async fn multi_search_with_context(
        context: MemoryContext,
        query: String,
        options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, MemoryControllerError> {
        Ok(Self::multi_search_with_context_impl(context, query, options).await?)
    }

    async fn multi_search_with_context_impl(
        context: MemoryContext,
        query: String,
        options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, SearchMemoryError> {
        use futures::stream::{FuturesUnordered, StreamExt};
        use tokio::sync::Semaphore;
        use tokio::task::JoinHandle;

        let options = options.unwrap_or_default();
        let vector_store = VectorStore::get_store().await?;
        let embedder = options.embedder.unwrap_or(Embedder::get_embedder().await?);
        let reranker = Reranker::get_reranker().await?;

        let semaphore = Arc::new(Semaphore::new(8)); // limit concurrency (tune this!)
        let mut tasks: FuturesUnordered<JoinHandle<Result<Vec<Memory>, SearchMemoryError>>> =
            FuturesUnordered::new();
        let sub_queries = Segmenter::process(&query)?;
        dbg!(&sub_queries);

        let sub_query_slices: Vec<&str> = sub_queries.iter().map(|s| s.as_str()).collect();
        let vectors = embedder.generate_embeddings(&sub_query_slices).await?;

        for vector in vectors {
            let permit = Arc::clone(&semaphore).acquire_owned().await?;
            let vector_store = Arc::clone(&vector_store);
            let context = context.clone();

            tasks.push(tokio::spawn(async move {
                let _permit = permit;
                let q = Query::builder()
                    .vector(vector)
                    .context(context)
                    .limit(5)
                    .build();

                Ok(vector_store.search(q).await?)
            }));
        }

        let mut all_memories = Vec::new();

        while let Some(task) = tasks.next().await {
            let mut memories = task??;
            all_memories.append(&mut memories);
        }

        let data = reranker.rank(query, &all_memories, None).await?.data;
        // TODO: hybrid search with "row.score" + other metrics
        let memories: Vec<Memory> = data
            .iter()
            .map(|row| std::mem::take(&mut all_memories[row.index]))
            .collect();

        Ok(memories)
    }
}
