// TODO: remove this allow once the module is fully implemented
#![allow(dead_code)]
mod providers;
mod response_generators;
pub(crate) mod utils;

use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

pub use providers::*;
pub use response_generators::*;

pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

lazy_static! {
    static ref reqwest_client: reqwest::Client = reqwest::Client::new();
}

pub enum LLMProvider {
    OpenAI(OpenAIProvider),
    AzureOpenAI(AzureOpenAIProvider),
    GoogleVertexAI(GoogleVertexAIProvider),
    Anthropic(AnthropicProvider),
    XAI(XAIProvider),
    AmazonBedrock(AmazonBedrockProvider),
}

impl LLMProvider {
    pub(crate) async fn do_generate_text(
        &self,
        request: GenerateTextRequest,
    ) -> Result<GenerateTextResponse, ResponseGeneratorError> {
        match self {
            LLMProvider::OpenAI(provider) => provider.generate_text(request),
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
            LLMProvider::OpenAI(provider) => provider.generate_object(request),
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

impl From<OpenAIProvider> for LLMProvider {
    fn from(config: OpenAIProvider) -> Self {
        LLMProvider::OpenAI(config)
    }
}

impl From<AzureOpenAIProvider> for LLMProvider {
    fn from(config: AzureOpenAIProvider) -> Self {
        LLMProvider::AzureOpenAI(config)
    }
}

impl From<AnthropicProvider> for LLMProvider {
    fn from(config: AnthropicProvider) -> Self {
        LLMProvider::Anthropic(config)
    }
}

impl From<XAIProvider> for LLMProvider {
    fn from(config: XAIProvider) -> Self {
        LLMProvider::XAI(config)
    }
}

impl From<AmazonBedrockProvider> for LLMProvider {
    fn from(config: AmazonBedrockProvider) -> Self {
        LLMProvider::AmazonBedrock(config)
    }
}

impl From<GoogleVertexAIProvider> for LLMProvider {
    fn from(config: GoogleVertexAIProvider) -> Self {
        LLMProvider::GoogleVertexAI(config)
    }
}
