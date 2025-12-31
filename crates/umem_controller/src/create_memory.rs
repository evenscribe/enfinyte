use super::{MemoryController, MemoryControllerError};
use chrono::Utc;
use std::{sync::Arc, time::Instant};
use thiserror::Error;
use tracing::info;
use typed_builder::TypedBuilder;
use umem_ai::LanguageModel;
use umem_annotations::{Annotation, AnnotationError, LLMAnnotated};
use umem_core::{
    LifecycleState, Memory, MemoryContentError, MemoryContext, MemoryContextError, MemoryError,
    TemporalMetadata,
};
use umem_embeddings::{Embedder, EmbedderBase, EmbedderError};
use umem_vector_store::{VectorStore, VectorStoreError};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CreateMemoryError {
    #[error("memory request --build-- failed with: {0}")]
    CreateMemoryRequestError(#[from] CreateMemoryRequestError),

    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("embedder action failed with: {0}")]
    EmbedderError(#[from] EmbedderError),
}

#[derive(Debug, Error)]
pub enum CreateMemoryRequestError {
    #[error("At least one of user_id, agent_id, or run_id must be set")]
    MissingContext,

    #[error("Content cannot be empty or whitespace")]
    MissingContent,

    #[error("memory validation failed with: {0}")]
    MemoryError(#[from] MemoryError),

    #[error("summarization failed with: {0}")]
    SummaryError(#[from] AnnotationError),

    #[error("memory content errored with: {0}")]
    MemoryContentError(#[from] MemoryContentError),

    #[error("memory context errored with: {0}")]
    MemoryContextError(#[from] MemoryContextError),
}

#[derive(TypedBuilder)]
pub struct CreateMemoryRequest {
    #[builder(default = None)]
    user_id: Option<String>,
    #[builder(default = None)]
    agent_id: Option<String>,
    #[builder(default = None)]
    run_id: Option<String>,
    raw_content: String,
}

impl CreateMemoryRequest {
    pub fn validate(&self) -> Result<(), CreateMemoryRequestError> {
        if self.user_id.is_none() && self.agent_id.is_none() && self.run_id.is_none() {
            return Err(CreateMemoryRequestError::MissingContext);
        }

        if self.raw_content.trim().is_empty() {
            return Err(CreateMemoryRequestError::MissingContent);
        }

        Ok(())
    }

    pub async fn build(
        self,
        model: Arc<LanguageModel>,
    ) -> Result<Memory, CreateMemoryRequestError> {
        self.validate()?;
        let annotations = self.annotations(model).await?;

        Ok(Memory::builder()
            .id(Uuid::new_v4())
            .content(annotations.content)
            .context(self.context()?)
            .kind(annotations.kind)
            // .signals(annotations.signals)
            // .provenance(annotations.provenance)
            .lifecycle(LifecycleState::Active)
            .temporal(TemporalMetadata::new(Utc::now()))
            .build())
    }

    fn context(&self) -> Result<MemoryContext, MemoryContextError> {
        if let Some(ref user_id) = self.user_id {
            return MemoryContext::for_user(user_id);
        }

        if let Some(ref agent_id) = self.agent_id {
            return MemoryContext::for_agent(agent_id);
        }

        if let Some(ref run_id) = self.run_id {
            return MemoryContext::for_run(run_id);
        }

        // NOTE: unreachable because validated before
        unreachable!()
    }

    async fn annotations(
        &self,
        model: Arc<LanguageModel>,
    ) -> Result<LLMAnnotated, CreateMemoryRequestError> {
        Ok(Annotation::generate(&self.raw_content, model).await?)
    }
}

#[derive(TypedBuilder, Default)]
pub struct CreateMemoryOptions {
    #[builder(default = None)]
    pub embedder: Option<Arc<dyn EmbedderBase + Send + Sync>>,
    #[builder(default = None)]
    pub model: Option<Arc<LanguageModel>>,
}

impl MemoryController {
    pub async fn create(
        request: CreateMemoryRequest,
        options: Option<CreateMemoryOptions>,
    ) -> Result<Memory, MemoryControllerError> {
        Ok(Self::create_impl(request, options).await?)
    }

    async fn create_impl(
        request: CreateMemoryRequest,
        options: Option<CreateMemoryOptions>,
    ) -> Result<Memory, CreateMemoryError> {
        let options = options.unwrap_or_default();
        let vector_store = VectorStore::get_store().await?;
        let embedder = options.embedder.unwrap_or(Embedder::get_embedder().await?);
        let model = options.model.unwrap_or(LanguageModel::get_model());

        let start = Instant::now();
        let memory = request.build(model).await?;
        let duration = start.elapsed();
        info!("llm annotations took {:?}", duration);

        let start = Instant::now();
        let vector = embedder.generate_embedding(memory.get_summary()).await?;
        let duration = start.elapsed();
        info!("embedding took {:?}", duration);

        let start = Instant::now();
        vector_store.insert(&[&vector], &[&memory]).await?;
        let duration = start.elapsed();
        info!("vector insert took {:?}", duration);

        Ok(memory)
    }
}
