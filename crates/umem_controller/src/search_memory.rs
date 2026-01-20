use super::{MemoryController, MemoryControllerError};
use std::{sync::Arc, time::Instant};
use thiserror::Error;
use tokio::{sync::AcquireError, task::JoinError};
use tracing::info;
use typed_builder::TypedBuilder;
use umem_core::{Memory, MemoryContext, MemoryContextError, Query};
use umem_embeddings::{EmbedderBase, EmbedderError};
use umem_refine::{RefineError, Segmenter};
use umem_rerank::RerankError;
use umem_vector_store::VectorStoreError;

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
        &self,
        user_id: String,
        query: String,
        options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, MemoryControllerError> {
        Ok(self.search_for_user_impl(user_id, query, options).await?)
    }

    async fn search_for_user_impl(
        &self,
        user_id: String,
        query: String,
        _options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, SearchMemoryError> {
        let vector_store = Arc::clone(&self.vector_store);
        let embedder = Arc::clone(&self.embedder);

        let vector = embedder.generate_embedding(query.as_str()).await?;
        let query = Query::builder()
            .vector(vector)
            .context(MemoryContext::for_user(user_id)?)
            .limit(1000)
            .build();

        Ok(vector_store.search(query).await?)
    }

    pub async fn search_with_context(
        &self,
        context: MemoryContext,
        query: String,
        options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, MemoryControllerError> {
        Ok(self
            .search_with_context_impl(context, query, options)
            .await?)
    }

    async fn search_with_context_impl(
        &self,
        context: MemoryContext,
        query: String,
        _options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, SearchMemoryError> {
        let vector_store = Arc::clone(&self.vector_store);
        let embedder = Arc::clone(&self.embedder);
        let reranker = Arc::clone(&self.reranker);

        let vector = embedder.generate_embedding(query.as_str()).await?;
        let vector_query = Query::builder()
            .vector(vector)
            .context(context)
            .limit(20)
            .build();

        let mut memories = vector_store.search(vector_query).await?;

        let data = reranker.rank(query, &memories, None).await?.data;

        let memories: Vec<Memory> = data
            .iter()
            .map(|row| std::mem::take(&mut memories[row.index]))
            .collect();

        Ok(memories)
    }

    pub async fn multi_search_with_context(
        &self,
        context: MemoryContext,
        query: String,
        options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, MemoryControllerError> {
        Ok(self
            .multi_search_with_context_impl(context, query, options)
            .await?)
    }

    async fn multi_search_with_context_impl(
        &self,
        context: MemoryContext,
        query: String,
        _options: Option<SearchMemoryOptions>,
    ) -> Result<Vec<Memory>, SearchMemoryError> {
        use futures::stream::{FuturesUnordered, StreamExt};
        use tokio::sync::Semaphore;
        use tokio::task::JoinHandle;

        let vector_store = Arc::clone(&self.vector_store);
        let embedder = Arc::clone(&self.embedder);
        let reranker = Arc::clone(&self.reranker);

        let semaphore = Arc::new(Semaphore::new(8)); // limit concurrency (tune this!)
        let mut tasks: FuturesUnordered<JoinHandle<Result<Vec<Memory>, SearchMemoryError>>> =
            FuturesUnordered::new();
        let mut sub_queries = Segmenter::process(&query)?;
        sub_queries.push(query.clone());

        let sub_query_slices: Vec<&str> = sub_queries.iter().map(|s| s.as_str()).collect();

        let start = Instant::now();
        let vectors = embedder.generate_embeddings(&sub_query_slices).await?;
        let duration = start.elapsed();
        info!("Embedder time : {:?}", duration);

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

        let start = Instant::now();
        while let Some(task) = tasks.next().await {
            let mut memories = task??;
            all_memories.append(&mut memories);
        }
        let duration = start.elapsed();
        info!("Searching time : {:?}", duration);

        let start = Instant::now();
        let data = reranker.rank(query, &all_memories, None).await?.data;
        let duration = start.elapsed();
        info!("Ranking time : {:?}", duration);

        // TODO: hybrid search with "row.score" + other metrics
        let memories: Vec<Memory> = data
            .iter()
            .map(|row| std::mem::take(&mut all_memories[row.index]))
            .collect();

        Ok(memories)
    }
}
