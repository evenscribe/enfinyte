use crate::response_generators::messages::Message;
use crate::utils;
use crate::utils::is_retryable_error;
use crate::LanguageModel;
use crate::ResponseGeneratorError;
use backon::ExponentialBuilder;
use backon::Retryable;
use reqwest::header::HeaderMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

// TODO: Wrap me with observers for logging, metrics, tracing, etc.
pub async fn generate_text(
    request: GenerateTextRequest,
) -> Result<GenerateTextResponse, ResponseGeneratorError> {
    let per_request_timeout = request.timeout;
    let max_retries = request.max_retries;
    let total_delay = per_request_timeout.mul_f32(max_retries as f32 / 2.0);

    let generation = || {
        let model = Arc::clone(&request.model);
        let provider = Arc::clone(&model.provider);
        let request = request.clone();

        async move {
            tokio::time::timeout(per_request_timeout, provider.do_generate_text(request))
                .await
                .map_err(ResponseGeneratorError::TimeoutError)
                .flatten()
        }
    };

    generation
        .retry(
            ExponentialBuilder::default()
                .with_max_times(max_retries)
                .with_total_delay(Some(total_delay)),
        )
        .sleep(tokio::time::sleep)
        .when(is_retryable_error)
        .notify(|err, dur| {
            tracing::debug!("retrying {:?} after {:?}", err, dur);
        })
        .await
}

#[derive(Debug)]
pub struct GenerateTextResponse {
    pub text: String,
}

#[derive(Clone)]
pub struct GenerateTextRequest {
    pub model: Arc<LanguageModel>,
    pub messages: Vec<Message>,
    pub max_output_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<usize>,
    pub presence_penalty: Option<f32>,
    pub seed: Option<u64>,
    pub max_retries: usize,
    pub headers: HeaderMap,
    pub timeout: Duration,
}

impl GenerateTextRequest {
    pub fn builder() -> GenerateTextRequestBuilder {
        GenerateTextRequestBuilder::new()
    }
}

#[derive(Debug, Error)]
pub enum GenerateTextRequestBuilderError {
    #[error("either set the `system` field or provide a system message in `messages` array")]
    RedundantSystemMessage,

    #[error("missing system message")]
    MissingSytemMessage,

    #[error("missing model")]
    MissingModel,

    #[error("missing provider")]
    MissingProvider,
}

pub struct GenerateTextRequestBuilder {
    pub model: Option<Arc<LanguageModel>>,
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
    pub duration: Option<Duration>,
}

impl GenerateTextRequestBuilder {
    pub fn new() -> Self {
        GenerateTextRequestBuilder {
            model: None,
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
            duration: Some(Duration::from_secs(60)),
        }
    }

    pub fn model(mut self, model: Arc<LanguageModel>) -> Self {
        self.model = Some(model);
        self
    }

    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
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

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn build(mut self) -> Result<GenerateTextRequest, GenerateTextRequestBuilderError> {
        let has_system_message_in_messages = self
            .messages
            .iter()
            .any(|message| matches!(message, Message::System(_)));

        if !has_system_message_in_messages && self.system.is_none() {
            return Err(GenerateTextRequestBuilderError::MissingSytemMessage);
        }

        if has_system_message_in_messages && self.system.is_some() {
            return Err(GenerateTextRequestBuilderError::RedundantSystemMessage);
        }

        if !has_system_message_in_messages {
            self.messages
                .insert(0, Message::System(self.system.unwrap()));
        }

        if let Some(user_prompt) = self.prompt {
            self.messages.push(Message::User(user_prompt.into()));
        }

        Ok(GenerateTextRequest {
            model: self
                .model
                .ok_or(GenerateTextRequestBuilderError::MissingModel)?,
            messages: self.messages,
            max_output_tokens: self.max_output_tokens,
            top_p: self.top_p,
            top_k: self.top_k,
            presence_penalty: self.presence_penalty,
            seed: self.seed,
            max_retries: self.max_retries.unwrap_or(3),
            headers: utils::build_header_map(self.headers.as_slice()).unwrap_or_default(),
            temperature: self.temperature,
            timeout: self.duration.unwrap_or(Duration::from_secs(60)),
        })
    }
}

impl Default for GenerateTextRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}
