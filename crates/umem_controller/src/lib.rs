mod create_memory_request;

use rustc_hash::FxHashMap;
use thiserror::Error;
use umem_core::{ContextFilter, Memory, Query};
use umem_embeddings::{Embedder, EmbedderError};
use umem_vector_store::{VectorStore, VectorStoreError};

pub use create_memory_request::{CreateMemoryRequest, CreateMemoryRequestError};

#[derive(Debug, Error)]
pub enum MemoryControllerError {
    #[error("creater memory failed with: {0}")]
    CreateMemoryError(#[from] CreateMemoryRequestError),

    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("embedder action failed with: {0}")]
    EmbedderError(#[from] EmbedderError),
}

#[derive(Debug, Default)]
pub struct MemoryController;

type Result<T> = std::result::Result<T, MemoryControllerError>;

impl MemoryController {
    pub async fn create(request: CreateMemoryRequest) -> Result<Memory> {
        let vector_store = VectorStore::get_store().await?;
        let embedder = Embedder::get_embedder().await?;
        let memory = request.build()?;
        let vector = embedder.generate_embedding(memory.get_summary()).await?;
        vector_store.insert(vec![vector], vec![&memory]).await?;
        Ok(memory)
    }

    pub async fn get(id: String) -> Result<Memory> {
        let vector_store = VectorStore::get_store().await?;
        Ok(vector_store.get(id.as_str()).await?)
    }

    // pub async fn update(request: UpdateMemoryRequestBuilder) -> Result<()> {
    //     let vector_store = VectorStore::get_store().await?;
    //     let embedder = Embedder::get_embedder().await?;
    //     let vector = if let Some(content) = &request.content {
    //         Some(embedder.generate_embedding(content).await?)
    //     } else {
    //         None
    //     };
    //     let payload: FxHashMap<String, serde_json::Value> = [
    //         request.content.map(|v| ("content".to_string(), v.into())),
    //         request.tags.map(|v| ("tags".to_string(), v.into())),
    //         request.priority.map(|v| ("priority".to_string(), v.into())),
    //         request
    //             .parent_id
    //             .map(|v| ("parent_id".to_string(), v.into())),
    //     ]
    //     .into_iter()
    //     .flatten()
    //     .collect();

    //     vector_store
    //         .update(request.id.as_str(), vector, Some(payload))
    //         .await
    // }

    pub async fn delete(id: String) -> Result<()> {
        let vector_store = VectorStore::get_store().await?;
        Ok(vector_store.delete(id.as_str()).await?)
    }

    pub async fn list(user_id: String) -> Result<Vec<Memory>> {
        let vector_store = VectorStore::get_store().await?;
        let query = Query::builder()
            .context(ContextFilter::for_user(user_id))
            .limit(1000)
            .build();

        Ok(vector_store.list(query).await?)
    }

    pub async fn search(user_id: String, query: String) -> Result<Vec<Memory>> {
        let vector_store = VectorStore::get_store().await?;
        let embedder = Embedder::get_embedder().await?;

        let vector = embedder.generate_embedding(query.as_str()).await?;

        let query = Query::builder()
            .vector(vector)
            .context(ContextFilter::for_user(user_id))
            .limit(1000)
            .build();

        Ok(vector_store.search(query).await?)
    }
}
