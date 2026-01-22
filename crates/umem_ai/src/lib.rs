// TODO: remove this allow once the module is fully implemented
#![allow(dead_code)]
mod model_impl;
mod providers;
mod response_generators;
mod utils;

use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
pub use model_impl::*;
pub use providers::*;
pub use response_generators::*;
use schemars::JsonSchema;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use thiserror::Error;

pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

lazy_static! {
    static ref reqwest_client: reqwest::Client = reqwest::Client::new();
}

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

#[async_trait]
pub trait GeneratesText {
    async fn generate_text(
        &self,
        request: GenerateTextRequest,
    ) -> Result<GenerateTextResponse, ResponseGeneratorError>;
}

#[async_trait]
pub trait GeneratesObject {
    async fn generate_object<T: Clone + JsonSchema + Serialize + Send + Sync + DeserializeOwned>(
        &self,
        request: GenerateObjectRequest<T>,
    ) -> Result<GenerateObjectResponse<T>, ResponseGeneratorError>;
}

#[async_trait]
pub trait Reranks {
    async fn rerank(
        &self,
        request: RerankRequest,
    ) -> Result<RerankResponse, ResponseGeneratorError>;
}

#[async_trait]
pub trait ReranksStructuredData {
    async fn rerank_structured<T>(
        &self,
        request: StructuredRerankRequest<T>,
    ) -> Result<StructuredRerankResponse<T>, ResponseGeneratorError>
    where
        T: Serialize + Clone + Send + Sync;
}

#[async_trait]
pub trait Embeds {
    async fn embed(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, ResponseGeneratorError>;
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
