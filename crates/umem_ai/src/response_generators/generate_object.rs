use crate::{response_generators::messages::Message, utils::is_retryable_error, LLMProvider};
use anyhow::{anyhow, Result};
use backon::{ExponentialBuilder, Retryable};
use schemars::{schema_for, JsonSchema, Schema};
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, sync::Arc};

pub async fn generate_object<T>(
    request: GenerateObjectRequest<T>,
) -> Result<GenerateObjectResponse<T>>
where
    T: Clone + JsonSchema + Send + Sync + Serialize + DeserializeOwned,
{
    let generation = || {
        let provider = Arc::clone(&request.provider);
        let request = request.clone();
        async move { provider.do_generate_object::<T>(request).await }
    };

    generation
        .retry(ExponentialBuilder::default().with_max_times(request.max_retries))
        .sleep(tokio::time::sleep)
        .when(is_retryable_error)
        .await
        .map_err(|e| anyhow!(e))
}

#[derive(Clone)]
pub struct GenerateObjectRequest<T>
where
    T: Clone + JsonSchema + Send + Sync + Serialize + DeserializeOwned,
{
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
    pub output_type: PhantomData<T>,
    pub output_schema: Schema,
}

#[derive(Debug)]
pub struct GenerateObjectResponse<T> {
    pub output: T,
}

pub struct GenerateObjectRequestBuilder<T>
where
    T: Clone + JsonSchema + Send + Sync + Serialize + DeserializeOwned,
{
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
    pub output_type: PhantomData<T>,
    pub output_schema: Schema,
}

impl<T> GenerateObjectRequestBuilder<T>
where
    T: Clone + JsonSchema + Send + Sync + Serialize + DeserializeOwned,
{
    pub fn new() -> Self {
        let schema = schema_for!(T);
        GenerateObjectRequestBuilder {
            model: None,
            provider: None,
            system: None,
            prompt: None,
            messages: Vec::new(),
            temperature: None,
            max_output_tokens: None,
            top_p: None,
            top_k: None,
            presence_penalty: None,
            seed: None,
            max_retries: None,
            headers: Vec::new(),
            output_type: PhantomData,
            output_schema: schema,
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

    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
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

    pub fn build(mut self) -> Result<GenerateObjectRequest<T>> {
        if self.model.is_none() {
            return Err(anyhow!("model is required".to_string()));
        }
        if self.provider.is_none() {
            return Err(anyhow!("provider is required".to_string()));
        }

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

        Ok(GenerateObjectRequest {
            model: self.model.unwrap(),
            provider: self.provider.unwrap(),
            messages: self.messages,
            max_output_tokens: self.max_output_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
            presence_penalty: self.presence_penalty,
            seed: self.seed,
            max_retries: self.max_retries.unwrap_or(3),
            headers: self.headers,
            output_type: PhantomData,
            output_schema: self.output_schema,
        })
    }
}
