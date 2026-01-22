mod amazon_bedrock;
mod anthropic;
mod azure_openai;
mod cohere;
mod google_vertex;
mod openai;
mod xai;
use crate::{
    Embeds, GenerateObjectRequest, GenerateObjectResponse, GenerateTextRequest,
    GenerateTextResponse, GeneratesObject, GeneratesText, RerankRequest, RerankResponse, Reranks,
    ReranksStructuredData, ResponseGeneratorError, StructuredRerankRequest,
    StructuredRerankResponse,
    embed::{EmbeddingRequest, EmbeddingResponse},
};
pub use amazon_bedrock::*;
pub use anthropic::AnthropicProvider;
pub use azure_openai::AzureOpenAIProvider;
pub use cohere::CohereProvider;
pub use google_vertex::GoogleVertexAIProvider;
pub use openai::OpenAIProvider;
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
pub use xai::XAIProvider;

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

#[derive(Debug)]
pub enum AIProvider {
    OpenAI(OpenAIProvider),
    AzureOpenAI(AzureOpenAIProvider),
    GoogleVertexAI(GoogleVertexAIProvider),
    Anthropic(AnthropicProvider),
    XAI(XAIProvider),
    AmazonBedrock(AmazonBedrockProvider),
    Cohere(CohereProvider),
}

impl AIProvider {
    pub(crate) async fn do_generate_text(
        &self,
        request: GenerateTextRequest,
    ) -> Result<GenerateTextResponse, ResponseGeneratorError> {
        match self {
            AIProvider::OpenAI(provider) => provider.generate_text(request),
            AIProvider::AmazonBedrock(provider) => provider.generate_text(request),
            _ => unimplemented!(),
        }
        .await
    }

    pub(crate) async fn do_generate_object<
        T: Clone + JsonSchema + Serialize + Send + Sync + DeserializeOwned,
    >(
        &self,
        request: GenerateObjectRequest<T>,
    ) -> Result<GenerateObjectResponse<T>, ResponseGeneratorError> {
        match self {
            AIProvider::OpenAI(provider) => provider.generate_object(request),
            AIProvider::AmazonBedrock(provider) => provider.generate_object(request),
            _ => unimplemented!(),
        }
        .await
    }

    pub(crate) async fn do_reranking(
        &self,
        request: RerankRequest,
    ) -> Result<RerankResponse, ResponseGeneratorError> {
        match self {
            AIProvider::Cohere(provider) => provider.rerank(request),
            AIProvider::AmazonBedrock(provider) => provider.rerank(request),
            _ => unimplemented!(),
        }
        .await
    }

    pub(crate) async fn do_structured_reranking<T>(
        &self,
        request: StructuredRerankRequest<T>,
    ) -> Result<StructuredRerankResponse<T>, ResponseGeneratorError>
    where
        T: Serialize + Clone + Send + Sync,
    {
        match self {
            AIProvider::Cohere(provider) => provider.rerank_structured(request).await,
            AIProvider::AmazonBedrock(provider) => provider.rerank_structured(request).await,
            _ => unimplemented!(),
        }
    }

    pub(crate) async fn do_embed(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, ResponseGeneratorError> {
        match self {
            AIProvider::AmazonBedrock(provider) => provider.embed(request),
            _ => unimplemented!(),
        }
        .await
    }
}

impl From<OpenAIProvider> for AIProvider {
    fn from(config: OpenAIProvider) -> Self {
        AIProvider::OpenAI(config)
    }
}

impl From<AzureOpenAIProvider> for AIProvider {
    fn from(config: AzureOpenAIProvider) -> Self {
        AIProvider::AzureOpenAI(config)
    }
}

impl From<AnthropicProvider> for AIProvider {
    fn from(config: AnthropicProvider) -> Self {
        AIProvider::Anthropic(config)
    }
}

impl From<XAIProvider> for AIProvider {
    fn from(config: XAIProvider) -> Self {
        AIProvider::XAI(config)
    }
}

impl From<AmazonBedrockProvider> for AIProvider {
    fn from(config: AmazonBedrockProvider) -> Self {
        AIProvider::AmazonBedrock(config)
    }
}

impl From<GoogleVertexAIProvider> for AIProvider {
    fn from(config: GoogleVertexAIProvider) -> Self {
        AIProvider::GoogleVertexAI(config)
    }
}
