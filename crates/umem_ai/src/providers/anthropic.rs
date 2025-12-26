use crate::response_generators::GenerateTextRequest;
use crate::response_generators::GenerateTextResponse;
use crate::GeneratesText;
use crate::ResponseGeneratorError;
use anyhow::bail;
use anyhow::Result;
use async_trait::async_trait;

pub struct AnthropicProvider {
    pub api_key: String,
    pub base_url: String,
    pub headers: Vec<(String, String)>,
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

pub struct AnthropicProviderBuilder {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub headers: Option<Vec<(String, String)>>,
}

impl AnthropicProviderBuilder {
    pub fn new() -> Self {
        AnthropicProviderBuilder {
            api_key: None,
            base_url: None,
            headers: None,
        }
    }
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }
    pub fn headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers = Some(headers);
        self
    }
    pub fn build(self) -> Result<AnthropicProvider> {
        if self.api_key.is_none() {
            bail!("api_key is required");
        }

        Ok(AnthropicProvider {
            api_key: self.api_key.unwrap_or_default(),
            base_url: self
                .base_url
                .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
            headers: self.headers.unwrap_or_default(),
        })
    }
}
