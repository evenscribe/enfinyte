use provider_configs::{
    AmazonBedrockConfig, AnthropicConfig, AzureOpenAIConfig, GoogleVertexAIConfig, OpenAIConfig,
    XAIConfig,
};

mod provider_configs;
mod response_generators;

pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

pub enum Provider {
    OpenAI(OpenAIConfig),
    AzureOpenAI(AzureOpenAIConfig),
    GoogleVertexAI(GoogleVertexAIConfig),
    Anthropic(AnthropicConfig),
    XAI(XAIConfig),
    AmazonBedrock(AmazonBedrockConfig),
}

impl From<OpenAIConfig> for Provider {
    fn from(config: OpenAIConfig) -> Self {
        Provider::OpenAI(config)
    }
}

impl From<AzureOpenAIConfig> for Provider {
    fn from(config: AzureOpenAIConfig) -> Self {
        Provider::AzureOpenAI(config)
    }
}

impl From<AnthropicConfig> for Provider {
    fn from(config: AnthropicConfig) -> Self {
        Provider::Anthropic(config)
    }
}

impl From<XAIConfig> for Provider {
    fn from(config: XAIConfig) -> Self {
        Provider::XAI(config)
    }
}

impl From<AmazonBedrockConfig> for Provider {
    fn from(config: AmazonBedrockConfig) -> Self {
        Provider::AmazonBedrock(config)
    }
}

impl From<GoogleVertexAIConfig> for Provider {
    fn from(config: GoogleVertexAIConfig) -> Self {
        Provider::GoogleVertexAI(config)
    }
}
