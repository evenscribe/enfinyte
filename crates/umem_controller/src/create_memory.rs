use super::{MemoryController, MemoryControllerError};
use chrono::Utc;
use std::sync::Arc;
use thiserror::Error;
use typed_builder::TypedBuilder;
use umem_ai::{
    embed::{embed, EmbeddingRequest},
    AIProviderError, EmbeddingModel, LanguageModel, ResponseGeneratorError,
};
use umem_annotations::{Annotation, AnnotationError, LLMAnnotated};
use umem_core::{
    LifecycleState, Memory, MemoryContentError, MemoryContext, MemoryContextError, MemoryError,
    TemporalMetadata,
};
use umem_vector_store::VectorStoreError;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum CreateMemoryError {
    #[error("memory request --build-- failed with: {0}")]
    CreateMemoryRequestError(#[from] CreateMemoryRequestError),

    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("ai provider action failed with: {0}")]
    AIProviderError(#[from] AIProviderError),

    #[error("response generator action failed with: {0}")]
    ResponseGeneratorError(#[from] ResponseGeneratorError),
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
    pub embedding_model: Option<Arc<EmbeddingModel>>,
    #[builder(default = None)]
    pub language_model: Option<Arc<LanguageModel>>,
}

impl MemoryController {
    pub async fn create(
        &self,
        request: CreateMemoryRequest,
        options: Option<CreateMemoryOptions>,
    ) -> Result<Memory, MemoryControllerError> {
        Ok(self.create_impl(request, options).await?)
    }

    async fn create_impl(
        &self,
        request: CreateMemoryRequest,
        _options: Option<CreateMemoryOptions>,
    ) -> Result<Memory, CreateMemoryError> {
        let vector_store = Arc::clone(&self.vector_store);
        let embedding_model = Arc::clone(&self.embedding_model);
        let language_model = Arc::clone(&self.language_model);

        let memory = request.build(language_model).await?;

        let request = EmbeddingRequest::builder()
            .model(embedding_model)
            .input(vec![memory.get_summary().to_owned()])
            .build();

        let embedding_response = embed(request).await?;

        //NOTE: change this later, just didin't want to fight with the drilled types
        let slices: Vec<&[f32]> = embedding_response
            .embeddings
            .iter()
            .map(|inner| inner.as_slice())
            .collect();
        let slice_of_slices: &[&[f32]] = &slices;

        vector_store.insert(slice_of_slices, &[&memory]).await?;
        Ok(memory)
    }
}
