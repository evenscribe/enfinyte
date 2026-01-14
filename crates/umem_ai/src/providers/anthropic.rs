use crate::GeneratesText;
use crate::ResponseGeneratorError;
use crate::response_generators::GenerateTextRequest;
use crate::response_generators::GenerateTextResponse;
use crate::utils;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Debug, Clone)]
pub struct AnthropicProvider {
    #[builder(setter(transform = |value: impl Into<String>| value.into()))]
    pub api_key: String,

    #[builder(default = "2023-06-01".into())]
    pub api_version: String,

    #[builder(default = "https://api.anthropic.com/v1".into())]
    pub base_url: String,

    #[builder(default = HeaderMap::default(), setter(transform = |value: Vec<(String, String)>|
           utils::build_header_map(value.as_slice()).unwrap_or_default()
    ))]
    pub headers: HeaderMap,
}

#[async_trait]
impl GeneratesText for AnthropicProvider {
    async fn generate_text(
        &self,
        _request: GenerateTextRequest,
    ) -> Result<GenerateTextResponse, ResponseGeneratorError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_anthropic_provider() {
        let provider = AnthropicProvider::builder()
            .api_key("sk-some-api-key")
            .build();
        dbg!("Anthropic Provider: {:?}", provider);
    }
}
