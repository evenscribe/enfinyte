use chrono::Utc;
use std::sync::Arc;
use thiserror::Error;
use typed_builder::TypedBuilder;
use umem_ai::{LLMProvider, OpenAIProviderBuilder};
use umem_annotations::{Annotation, AnnotationError, LLMAnnotated};
use umem_core::{
    LifecycleState, Memory, MemoryContentError, MemoryContext, MemoryContextError, MemoryError,
    TemporalMetadata,
};
use uuid::Uuid;

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

    pub async fn build(self) -> Result<Memory, CreateMemoryRequestError> {
        self.validate()?;
        let annotations = self.annotations().await?;

        Ok(Memory::builder()
            .id(Uuid::new_v4())
            .content(annotations.content)
            .context(self.context()?)
            .kind(annotations.kind)
            .signals(annotations.signals)
            .provenance(annotations.provenance)
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

    async fn annotations(&self) -> Result<LLMAnnotated, CreateMemoryRequestError> {
        let provider = Arc::new(LLMProvider::from(
            OpenAIProviderBuilder::new()
                .api_key("")
                .base_url("")
                .build()
                .unwrap(),
        ));

        Ok(Annotation::generate(
            &self.raw_content,
            provider,
            "arcee-ai/trinity-mini:free",
            None,
        )
        .await?)
    }
}
