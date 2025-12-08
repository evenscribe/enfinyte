use std::collections::HashMap;

pub enum ProviderConfig {
    OpenAI(OpenAIConfig),
}

impl ProviderConfig {
    pub fn new_openai(
        api_key: String,
        base_url: Option<String>,
        default_headers: Option<HashMap<String, String>>,
    ) -> Self {
        ProviderConfig::OpenAI(OpenAIConfig {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            default_headers: default_headers.unwrap_or_default(),
        })
    }
}

pub struct OpenAIConfig {
    pub api_key: String,
    pub base_url: String,
    pub default_headers: HashMap<String, String>,
}
