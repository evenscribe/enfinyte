use anyhow::{Result, bail};

#[derive(Clone, Debug)]
pub struct XAIProvider {
    pub api_key: String,
    pub base_url: String,
    pub headers: Vec<(String, String)>,
}

pub struct XAIProviderBuilder {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub headers: Option<Vec<(String, String)>>,
}

impl XAIProviderBuilder {
    pub fn new() -> Self {
        XAIProviderBuilder {
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
    pub fn build(self) -> Result<XAIProvider> {
        if self.api_key.is_none() {
            bail!("api_key is required".to_string());
        }

        Ok(XAIProvider {
            api_key: self.api_key.unwrap_or_default(),
            base_url: self
                .base_url
                .unwrap_or_else(|| "https://api.x.ai/v1".to_string()),
            headers: self.headers.unwrap_or_default(),
        })
    }
}
