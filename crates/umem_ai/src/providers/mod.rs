mod amazon_bedrock;
mod anthropic;
mod azure_openai;
mod cohere;
mod google_vertex;
mod openai;
mod xai;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AIProviderError {
    #[error("provider build failed with : {0}")]
    ProviderBuilderError(#[from] ProviderBuilderError),
}

#[derive(Error, Debug)]
pub enum ProviderBuilderError {
    #[error("amazon bedrock provider build failed with : {0}")]
    AmazonBedrockProviderBuilderError(#[from] AmazonBedrockProviderBuilderError),
}

pub use amazon_bedrock::*;
pub use anthropic::AnthropicProvider;
pub use azure_openai::AzureOpenAIProvider;
pub use cohere::CohereProvider;
pub use google_vertex::GoogleVertexAIProvider;
pub use openai::OpenAIProvider;
pub use xai::XAIProvider;
