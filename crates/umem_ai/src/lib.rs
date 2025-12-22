use crate::{
    providers::{
        AmazonBedrockProvider, AnthropicProvider, AzureOpenAIProvider, GoogleVertexAIProvider,
        OpenAIProvider, XAIProvider,
    },
    response_generators::{GenerateTextError, GenerateTextRequest, GenerateTextResponse},
};
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;

mod providers;
mod response_generators;
pub mod utils;

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
    ) -> Result<GenerateTextResponse, GenerateTextError> {
        match self {
            LLMProvider::OpenAI(provider) => provider.generate_text(request),
            LLMProvider::Anthropic(provider) => provider.generate_text(request),
            LLMProvider::AzureOpenAI(provider) => todo!(),
            LLMProvider::GoogleVertexAI(provider) => todo!(),
            LLMProvider::XAI(provider) => unimplemented!(),
            LLMProvider::AmazonBedrock(provider) => unimplemented!(),
        }
        .await
    }
}

#[async_trait]
pub trait GeneratesText {
    async fn generate_text(
        &self,
        request: GenerateTextRequest,
    ) -> Result<GenerateTextResponse, GenerateTextError>;
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
