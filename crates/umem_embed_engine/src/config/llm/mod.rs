pub mod providers;

use crate::HashMap;
use providers::ProviderConfig;
use serde_json::Value;

pub enum QueryType {}

pub struct LLMConfig {
    /// Number of documents to pull from the database for context
    documents_to_pull_from_db: usize,
    /// The system message to use for the LLM, default is None
    system_prompt: Option<String>,
    /// The prompt to use for the LLM, default is None
    prompt: Option<String>,
    /// Controls the randomness of the LLM's output
    /// Higher values like 0.8 will make the output more random,
    /// while lower values like 0.2 will make it more focused and deterministic
    temperature: f32,
    /// Maximum number of tokens to generate in the LLM's response
    /// Defaults to 1024
    max_tokens: usize,
    // The LLM model to use
    model: String,
    /// Controls the diversity of the LLM's output
    /// Values range from 0.0 to 1.0, Defaults to 1.0
    top_p: f32,
    /// Whether to use online resources (google search) to enhance responses
    /// Defaults to false
    online: bool,
    /// If True, the model will be run locally, defaults to False (for huggingface provider)
    local: bool,
    /// Whether to output token usage information for debugging
    debug_token_usage: bool,
    // The type of query to perform
    query_type: Option<QueryType>,
    /// Additional model-specific arguments
    model_args: HashMap<String, Value>,
    /// Additional filters to apply when querying the vector database to narrow down context documents
    where_filters: Vec<(String, Value)>,
    /// ONLY used for OpenAI compliant APIs, if provided will override the model_id while sending
    /// the requests
    deployment_name: Option<String>,
    ///  Provider specific configuration
    provider_config: Option<ProviderConfig>,
}

impl LLMConfig {
    pub fn new() -> Self {
        LLMConfig {
            documents_to_pull_from_db: 1,
            debug_token_usage: false,
            temperature: 0.2,
            top_p: 1.0,
            max_tokens: 1024,
            ..Default::default()
        }
    }

    pub fn provider_config(mut self, config: ProviderConfig) -> Self {
        self.provider_config = Some(config);
        self
    }

    pub fn system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }

    pub fn prompt(mut self, prompt: String) -> Self {
        self.prompt = Some(prompt);
        self
    }

    pub fn query_type(mut self, query_type: QueryType) -> Self {
        self.query_type = Some(query_type);
        self
    }

    pub fn documents_to_pull_from_db(mut self, count: usize) -> Self {
        self.documents_to_pull_from_db = count;
        self
    }

    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = tokens;
        self
    }

    pub fn enable_debug_token_usage(mut self, enable: bool) -> Self {
        self.debug_token_usage = enable;
        self
    }

    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }

    pub fn model_id(mut self, model_id: String) -> Self {
        self.model = model_id;
        self
    }

    pub fn enable_online(mut self, enable: bool) -> Self {
        self.online = enable;
        self
    }

    pub fn enable_local(mut self, enable: bool) -> Self {
        self.local = enable;
        self
    }

    pub fn model_args(mut self, args: HashMap<String, Value>) -> Self {
        self.model_args = args;
        self
    }

    pub fn where_filters(mut self, filters: Vec<(String, Value)>) -> Self {
        self.where_filters = filters;
        self
    }

    pub fn deployment_name(mut self, name: String) -> Self {
        self.deployment_name = Some(name);
        self
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self::new()
    }
}
