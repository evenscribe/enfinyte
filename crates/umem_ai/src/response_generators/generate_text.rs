use crate::response_generators::messages::Message;
use crate::utils::is_retryable_error;
use crate::LLMProvider;
use anyhow::anyhow;
use anyhow::Result;
use backon::ExponentialBuilder;
use backon::Retryable;
use std::sync::Arc;

// TODO: Wrap me with observers for logging, metrics, tracing, etc.
pub async fn generate_text(request: GenerateTextRequest) -> Result<GenerateTextResponse> {
    let generation = || {
        let provider = Arc::clone(&request.provider);
        let request = request.clone();
        async move { provider.do_generate_text(request).await }
    };

    generation
        .retry(ExponentialBuilder::default().with_max_times(request.max_retries))
        .sleep(tokio::time::sleep)
        .when(is_retryable_error)
        .await
        .map_err(|e| anyhow!(e))
}

#[derive(Debug)]
pub struct GenerateTextResponse {
    pub text: String,
}

#[derive(Clone)]
pub struct GenerateTextRequest {
    pub model: String,
    pub provider: Arc<LLMProvider>,
    pub messages: Vec<Message>,
    pub max_output_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<usize>,
    pub presence_penalty: Option<f32>,
    pub seed: Option<u64>,
    pub max_retries: usize,
    pub headers: Vec<(String, String)>,
}

pub struct GenerateTextRequestBuilder {
    pub model: Option<String>,
    pub provider: Option<Arc<LLMProvider>>,
    pub system: Option<String>,
    pub prompt: Option<String>,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub top_k: Option<usize>,
    pub presence_penalty: Option<f32>,
    pub seed: Option<u64>,
    pub max_retries: Option<usize>,
    pub headers: Vec<(String, String)>,
}

impl GenerateTextRequestBuilder {
    pub fn new() -> Self {
        GenerateTextRequestBuilder {
            model: None,
            provider: None,
            system: None,
            prompt: None,
            max_output_tokens: None,
            top_p: None,
            top_k: None,
            temperature: None,
            presence_penalty: None,
            seed: None,
            max_retries: None,
            headers: vec![],
            messages: vec![],
        }
    }

    pub fn model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn provider(mut self, provider: Arc<LLMProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn system(mut self, system: String) -> Self {
        self.system = Some(system);
        self
    }

    pub fn prompt(mut self, prompt: String) -> Self {
        self.prompt = Some(prompt);
        self
    }

    pub fn max_output_tokens(mut self, max_output_tokens: usize) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn top_k(mut self, top_k: usize) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    pub fn headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers = headers;
        self
    }

    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn build(mut self) -> Result<GenerateTextRequest> {
        let has_system_message_in_messages = self
            .messages
            .iter()
            .any(|message| matches!(message, Message::System(_)));

        if !has_system_message_in_messages && self.system.is_none() {
            anyhow::bail!(
                "either set the `system` field or provide a system message in `messages` array"
            )
        }

        if has_system_message_in_messages && self.system.is_some() {
            anyhow::bail!(
                "cannot set `system` field and also have a system message in `messages` array"
            );
        }

        if !has_system_message_in_messages {
            self.messages
                .insert(0, Message::System(self.system.unwrap()));
        }

        if let Some(user_prompt) = self.prompt {
            self.messages.push(Message::User(user_prompt.into()));
        }

        Ok(GenerateTextRequest {
            model: self.model.ok_or(anyhow!("model is required"))?,
            messages: self.messages,
            provider: self.provider.ok_or(anyhow!("provider is required"))?,
            max_output_tokens: self.max_output_tokens,
            top_p: self.top_p,
            top_k: self.top_k,
            presence_penalty: self.presence_penalty,
            seed: self.seed,
            max_retries: self.max_retries.unwrap_or(3),
            headers: self.headers,
            temperature: self.temperature,
        })
    }
}
