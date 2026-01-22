use crate::{AIProvider, AIProviderError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmbeddingModelError {
    #[error("ai provider failed with : {0}")]
    AIProviderError(#[from] AIProviderError),
}

#[derive(Debug, Clone)]
pub struct EmbeddingModel {
    pub provider: Arc<AIProvider>,
    pub model_name: String,
}

impl EmbeddingModel {
    fn new(provider: Arc<AIProvider>, model_name: String) -> Self {
        Self {
            provider,
            model_name,
        }
    }
}
