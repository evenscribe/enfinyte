pub mod generate_object;
pub mod generate_text;
pub mod messages;
pub mod rerank;
pub mod structured_rerank;

pub use generate_object::*;
pub use generate_text::*;
pub use messages::*;
pub use rerank::*;

use thiserror::Error;
#[derive(Error, Debug)]
pub enum ResponseGeneratorError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error("deserialization error, Details: {1}, Response: {0}")]
    Deserialization(serde_json::Error, String),
    #[error(transparent)]
    TimeoutError(#[from] tokio::time::error::Elapsed),
    #[error("Bedrock Converse API error, Details: {0}")]
    BedrockConverseError(String),
    #[error("empty response from AI provider")]
    EmptyProviderResponse,
    #[error("invalid response from AI provider, Details: {0}")]
    InvalidProviderResponse(String),
    #[error(transparent)]
    Transient(#[from] anyhow::Error),
}
