mod create_memory_request;
mod update_memory_request;

use std::sync::Arc;
use typed_builder::TypedBuilder;

use thiserror::Error;
use umem_ai::LanguageModel;
use umem_core::{Memory, MemoryContext, MemoryContextError, Query};
use umem_embeddings::{Embedder, EmbedderBase, EmbedderError};
use umem_vector_store::{VectorStore, VectorStoreBase, VectorStoreError};

pub use create_memory_request::{CreateMemoryRequest, CreateMemoryRequestError};
pub use update_memory_request::{UpdateMemoryRequest, UpdateMemoryRequestError};

#[derive(Debug, Error)]
pub enum MemoryControllerError {
    #[error("memory context failed: {0}")]
    MemoryContextError(#[from] MemoryContextError),

    #[error("create memory failed with: {0}")]
    CreateMemoryError(#[from] CreateMemoryRequestError),

    #[error("update memory failed with: {0}")]
    UpdateMemoryError(#[from] UpdateMemoryRequestError),

    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("embedder action failed with: {0}")]
    EmbedderError(#[from] EmbedderError),
}

#[derive(Debug, Default)]
pub struct MemoryController;

type Result<T> = std::result::Result<T, MemoryControllerError>;

#[derive(TypedBuilder)]
pub struct CreateMemoryOptions {
    #[builder(default = None)]
    pub vector_store: Option<Arc<dyn VectorStoreBase + Send + Sync>>,
    #[builder(default = None)]
    pub embedder: Option<Arc<dyn EmbedderBase + Send + Sync>>,
    #[builder(default = None)]
    pub model: Option<Arc<LanguageModel>>,
}

impl MemoryController {
    pub async fn create(
        request: CreateMemoryRequest,
        options: Option<CreateMemoryOptions>,
    ) -> Result<Memory> {
        let mut vector_store = VectorStore::get_store().await?;
        let mut embedder = Embedder::get_embedder().await?;
        let mut model = LanguageModel::get_model();

        if let Some(options) = options {
            if let Some(_vector_store) = options.vector_store {
                vector_store = _vector_store;
            }
            if let Some(_embedder) = options.embedder {
                embedder = _embedder;
            }
            if let Some(_model) = options.model {
                model = _model;
            }
        }

        let memory = request.build(model).await?;
        let vector = embedder.generate_embedding(memory.get_summary()).await?;
        vector_store.insert(&[&vector], &[&memory]).await?;
        Ok(memory)
    }

    pub async fn get(id: String) -> Result<Memory> {
        let vector_store = VectorStore::get_store().await?;
        Ok(vector_store.get(id.as_str()).await?)
    }

    pub async fn update(request: UpdateMemoryRequest) -> Result<()> {
        let vector_store = VectorStore::get_store().await?;
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

    pub async fn delete(id: String) -> Result<()> {
        let vector_store = VectorStore::get_store().await?;
        Ok(vector_store.delete(id.as_str()).await?)
    }

    pub async fn list_for_user(user_id: String) -> Result<Vec<Memory>> {
        let vector_store = VectorStore::get_store().await?;
        let query = Query::builder()
            .context(MemoryContext::for_user(user_id)?)
            .limit(1000)
            .build();

        Ok(vector_store.list(query).await?)
    }

    pub async fn list_with_context(context: MemoryContext) -> Result<Vec<Memory>> {
        let vector_store = VectorStore::get_store().await?;
        let query = Query::builder().context(context).limit(1000).build();

        Ok(vector_store.list(query).await?)
    }

    pub async fn search_for_user(user_id: String, query: String) -> Result<Vec<Memory>> {
        let vector_store = VectorStore::get_store().await?;
        let embedder = Embedder::get_embedder().await?;

        let vector = embedder.generate_embedding(query.as_str()).await?;

        let query = Query::builder()
            .vector(vector)
            .context(MemoryContext::for_user(user_id)?)
            .limit(1000)
            .build();

        Ok(vector_store.search(query).await?)
    }

    pub async fn search_with_context(context: MemoryContext, query: String) -> Result<Vec<Memory>> {
        let vector_store = VectorStore::get_store().await?;
        let embedder = Embedder::get_embedder().await?;

        let vector = embedder.generate_embedding(query.as_str()).await?;

        let query = Query::builder()
            .vector(vector)
            .context(context)
            .limit(1000)
            .build();

        Ok(vector_store.search(query).await?)
    }
}
