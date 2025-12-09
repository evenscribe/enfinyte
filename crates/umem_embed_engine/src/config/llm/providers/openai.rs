use crate::HashMap;
use anyhow::{Result, bail};

pub struct OpenAIConfig {
    pub api_key: String,
    pub base_url: String,
    pub default_headers: HashMap<String, String>,
    pub organization: Option<String>,
    pub project: Option<String>,
}

pub struct OpenAIConfigBuilder {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_headers: Option<HashMap<String, String>>,
    pub organization: Option<String>,
    pub project: Option<String>,
}

impl OpenAIConfigBuilder {
    fn new() -> Self {
        OpenAIConfigBuilder {
            api_key: None,
            base_url: None,
            default_headers: None,
            organization: None,
            project: None,
        }
    }

    fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    fn base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }

    fn default_headers(mut self, default_headers: HashMap<String, String>) -> Self {
        self.default_headers = Some(default_headers);
        self
    }

    fn organization(mut self, organization: String) -> Self {
        self.organization = Some(organization);
        self
    }

    fn project(mut self, project: String) -> Self {
        self.project = Some(project);
        self
    }

    fn build(self) -> Result<OpenAIConfig> {
        if self.api_key.is_none() {
            bail!("api_key is required");
        }

        Ok(OpenAIConfig {
            api_key: self.api_key.unwrap(),
            base_url: self
                .base_url
                .unwrap_or("https://api.openai.com/v1".to_string()),
            default_headers: self.default_headers.unwrap_or_default(),
            organization: self.organization,
            project: self.project,
        })
    }
}
