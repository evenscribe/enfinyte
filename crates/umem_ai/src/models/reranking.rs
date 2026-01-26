use crate::{AIProvider, AIProviderError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RerankingModelError {
    #[error("ai provider failed with : {0}")]
    AIProviderError(#[from] AIProviderError),
}

#[derive(Debug, Clone)]
pub struct RerankingModel {
    pub provider: Arc<AIProvider>,
    pub model_name: String,
}

impl RerankingModel {
    fn new(provider: Arc<AIProvider>, model_name: String) -> Self {
        Self {
            provider,
            model_name,
        }
    }
}
