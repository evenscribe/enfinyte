use std::{sync::Arc, time::Duration};

use backon::{ExponentialBuilder, Retryable};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use crate::{RerankingModel, ResponseGeneratorError, utils::is_retryable_error};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SerializationFormat {
    #[default]
    Compact,
    Pretty,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SerializationMode {
    #[default]
    Json,
    Yaml,
}

pub async fn structured_rerank<T>(
    request: StructuredRerankRequest<T>,
) -> Result<StructuredRerankResponse<T>, ResponseGeneratorError>
where
    T: Serialize + Clone + Send + Sync,
{
    let per_request_timeout = request.timeout;
    let max_retries = request.max_retries;
    let total_delay = per_request_timeout.mul_f32(max_retries as f32 / 2.0);

    let reranking_request = || {
        let model = Arc::clone(&request.model);
        let provider = Arc::clone(&model.provider);
        let request = request.clone();

        async move {
            tokio::time::timeout(
                per_request_timeout,
                provider.do_structured_reranking(request),
            )
            .await
            .map_err(ResponseGeneratorError::TimeoutError)
            .flatten()
        }
    };

    reranking_request
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

#[derive(Clone)]
pub struct StructuredRerankRequest<T>
where
    T: Serialize + Clone,
{
    pub query: String,
    pub documents: Vec<T>,
    pub top_n: usize,
    pub timeout: Duration,
    pub max_retries: usize,
    pub model: Arc<RerankingModel>,
    pub serialization_format: SerializationFormat,
    pub serialization_mode: SerializationMode,
}

impl<T> StructuredRerankRequest<T>
where
    T: Serialize + Clone,
{
    pub fn builder() -> StructuredRerankRequestBuilder<T> {
        StructuredRerankRequestBuilder::default()
    }
}

pub struct StructuredRerankRequestBuilder<T>
where
    T: Serialize + Clone,
{
    query: Option<String>,
    documents: Vec<T>,
    top_n: usize,
    timeout: Duration,
    max_retries: usize,
    model: Option<Arc<RerankingModel>>,
    serialization_format: SerializationFormat,
    serialization_mode: SerializationMode,
}

impl<T> Default for StructuredRerankRequestBuilder<T>
where
    T: Serialize + Clone,
{
    fn default() -> Self {
        Self {
            query: None,
            documents: vec![],
            top_n: 5,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            model: None,
            serialization_format: SerializationFormat::default(),
            serialization_mode: SerializationMode::default(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StructuredRerankRequestBuilderError {
    #[error("missing query from structured rerank request")]
    MissingQuery,

    #[error("at least one document is required in structured rerank request")]
    EmptyDocuments,

    #[error("missing model while sending structured reranking request")]
    MissingModel,
}

impl<T> StructuredRerankRequestBuilder<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn documents<I>(mut self, documents: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.documents.extend(documents);
        self
    }

    pub fn document(mut self, document: T) -> Self {
        self.documents.push(document);
        self
    }

    pub fn top_k(mut self, top_n: usize) -> Self {
        self.top_n = top_n;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn model(mut self, model: Arc<RerankingModel>) -> Self {
        self.model = Some(model);
        self
    }

    pub fn serialization_format(mut self, format: SerializationFormat) -> Self {
        self.serialization_format = format;
        self
    }

    pub fn serialization_mode(mut self, mode: SerializationMode) -> Self {
        self.serialization_mode = mode;
        self
    }

    pub fn build(self) -> Result<StructuredRerankRequest<T>, StructuredRerankRequestBuilderError> {
        if self.documents.is_empty() {
            return Err(StructuredRerankRequestBuilderError::EmptyDocuments);
        }

        Ok(StructuredRerankRequest {
            query: self
                .query
                .ok_or(StructuredRerankRequestBuilderError::MissingQuery)?,
            documents: self.documents,
            top_n: self.top_n,
            timeout: self.timeout,
            max_retries: self.max_retries,
            model: self
                .model
                .ok_or(StructuredRerankRequestBuilderError::MissingModel)?,
            serialization_format: self.serialization_format,
            serialization_mode: self.serialization_mode,
        })
    }
}

pub struct StructuredRerankResponse<T>
where
    T: Serialize + Clone,
{
    pub rankings: Vec<StructuredRanking<T>>,
    pub ranked_documents: Vec<T>,
    pub raw_fields: Map<String, Value>,
}

pub struct StructuredRanking<T>
where
    T: Serialize + Clone,
{
    pub original_index: usize,
    pub score: f32,
    pub document: T,
}
