mod create_memory_request;

use thiserror::Error;
use umem_core::Memory;
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
        let memory_store = VectorStore::get_store().await?;
        let embedder = Embedder::get_embedder().await?;
        let memory = request.build()?;
        let vector = embedder.generate_embedding(memory.get_summary()).await?;
        memory_store.insert(vec![vector], vec![&memory]).await?;
        Ok(memory)
    }

    // pub async fn get(id: String) -> Result<Memory> {
    //     let memory_store = VectorStore::get_store().await?;
    //     memory_store.get(id.as_str()).await
    // }

    // pub async fn update(request: UpdateMemoryRequestBuilder) -> Result<()> {
    //     let memory_store = VectorStore::get_store().await?;
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

    //     memory_store
    //         .update(request.id.as_str(), vector, Some(payload))
    //         .await
    // }

    // pub async fn delete(id: String) -> Result<()> {
    //     let memory_store = VectorStore::get_store().await?;
    //     memory_store.delete(id.as_str()).await
    // }

    // pub async fn list(user_id: String) -> Result<Vec<Memory>> {
    //     let memory_store = VectorStore::get_store().await?;
    //     let filters = FxHashMap::from_iter([("user_id", user_id)]);
    //     memory_store.list(Some(filters), 1000).await
    // }

    // pub async fn search(user_id: String, query: String) -> Result<Vec<Memory>> {
    //     let memory_store = VectorStore::get_store().await?;
    //     let embedder = Embedder::get_embedder().await?;

    //     let filters = FxHashMap::from_iter([("user_id", user_id)]);
    //     let vector = embedder.generate_embedding(query.as_str()).await?;
    //     memory_store.search(vector, Some(filters), 1000).await
    // }
}
