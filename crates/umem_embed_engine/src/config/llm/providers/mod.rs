mod amazon_bedrock;
mod anthropic;
mod azure_openai;
mod google_vertex;
mod openai;
mod xai;

use amazon_bedrock::AmazonBedrockConfig;
use anthropic::AnthropicConfig;
use azure_openai::AzureOpenAIConfig;
use google_vertex::GoogleVertexAIConfig;
use openai::OpenAIConfig;
use xai::XAIConfig;

pub enum ProviderConfig {
    OpenAI(OpenAIConfig),
    AzureOpenAI(AzureOpenAIConfig),
    GoogleVertexAI(GoogleVertexAIConfig),
    Anthropic(AnthropicConfig),
    XAI(XAIConfig),
    AmazonBedrock(AmazonBedrockConfig),
}

impl From<OpenAIConfig> for ProviderConfig {
    fn from(config: OpenAIConfig) -> Self {
        ProviderConfig::OpenAI(config)
    }
}

impl From<AzureOpenAIConfig> for ProviderConfig {
    fn from(config: AzureOpenAIConfig) -> Self {
        ProviderConfig::AzureOpenAI(config)
    }
}

impl From<AnthropicConfig> for ProviderConfig {
    fn from(config: AnthropicConfig) -> Self {
        ProviderConfig::Anthropic(config)
    }
}

impl From<XAIConfig> for ProviderConfig {
    fn from(config: XAIConfig) -> Self {
        ProviderConfig::XAI(config)
    }
}

impl From<AmazonBedrockConfig> for ProviderConfig {
    fn from(config: AmazonBedrockConfig) -> Self {
        ProviderConfig::AmazonBedrock(config)
    }
}

impl From<GoogleVertexAIConfig> for ProviderConfig {
    fn from(config: GoogleVertexAIConfig) -> Self {
        ProviderConfig::GoogleVertexAI(config)
    }
}
