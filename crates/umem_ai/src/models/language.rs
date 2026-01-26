use crate::{AIProvider, AIProviderError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LanguageModelError {
    #[error("ai provider failed with : {0}")]
    AIProviderError(#[from] AIProviderError),
}

pub struct LanguageModel {
    pub provider: Arc<AIProvider>,
    pub model_name: String,
}

impl LanguageModel {
    fn new(provider: Arc<AIProvider>, model_name: String) -> Self {
        Self {
            provider,
            model_name,
        }
    }
}
