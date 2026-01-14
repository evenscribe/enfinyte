use crate::{
    GeneratesText,
    response_generators::{GenerateTextRequest, GenerateTextResponse, ResponseGeneratorError},
};
use anyhow::{Result, bail};
use async_trait::async_trait;
pub struct AzureOpenAIProvider {
    pub resource_name: Option<String>,
    pub api_key: String,
    pub api_version: String,
    pub base_url: String,
    pub headers: Vec<(String, String)>,
}

#[async_trait]
impl GeneratesText for AzureOpenAIProvider {
    async fn generate_text(
        &self,
        _request: GenerateTextRequest,
    ) -> Result<GenerateTextResponse, ResponseGeneratorError> {
        unimplemented!()
    }
}

pub struct AzureOpenAIProviderBuilder {
    pub resource_name: Option<String>,
    pub api_key: Option<String>,
    pub api_version: Option<String>,
    pub base_url: Option<String>,
    pub headers: Option<Vec<(String, String)>>,
}

impl AzureOpenAIProviderBuilder {
    pub fn new() -> Self {
        AzureOpenAIProviderBuilder {
            resource_name: None,
            api_key: None,
            api_version: None,
            base_url: None,
            headers: None,
        }
    }

    pub fn resource_name(mut self, resource_name: String) -> Self {
        self.resource_name = Some(resource_name);
        self
    }

    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn api_version(mut self, api_version: String) -> Self {
        self.api_version = Some(api_version);
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

    pub fn build(mut self) -> Result<AzureOpenAIProvider> {
        if self.api_key.is_none() {
            bail!("api_key is required");
        }

        if self.base_url.is_none() && self.resource_name.is_none() {
            bail!("Either base_url or resource_name must be provided");
        }

        if self.base_url.is_none() && self.resource_name.is_some() {
            let resource_name = self.resource_name.clone().unwrap();
            self.base_url = Some(format!(
                "https://{}.openai.azure.com/openai/v1",
                resource_name
            ));
        }

        Ok(AzureOpenAIProvider {
            resource_name: self.resource_name,
            api_key: self.api_key.unwrap(),
            api_version: self.api_version.unwrap_or("v1".to_string()),
            base_url: self.base_url.unwrap(),
            headers: self.headers.unwrap_or_default(),
        })
    }
}
