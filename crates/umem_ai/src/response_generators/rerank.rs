use backon::{ExponentialBuilder, Retryable};
use serde_json::{Map, Value};

use crate::{RerankingModel, ResponseGeneratorError, utils::is_retryable_error};
use std::{sync::Arc, time::Duration};

pub async fn rerank(request: RerankRequest) -> Result<RerankResponse, ResponseGeneratorError> {
    let per_request_timeout = request.timeout;
    let max_retries = request.max_retries;
    let total_delay = per_request_timeout.mul_f32(max_retries as f32 / 2.0);

    let reranking_request = || {
        let model = Arc::clone(&request.model);
        let provider = Arc::clone(&model.provider);
        let request = request.clone();

        async move {
            tokio::time::timeout(per_request_timeout, provider.do_reranking(request))
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
pub struct RerankRequest {
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: usize,
    pub timeout: Duration,
    pub max_retries: usize,
    pub model: Arc<RerankingModel>,
}

impl RerankRequest {
    pub fn builder() -> RerankRequestBuilder {
        RerankRequestBuilder {
            top_n: 5,
            ..Default::default()
        }
    }
}

pub struct RerankRequestBuilder {
    query: Option<String>,
    documents: Vec<String>,
    top_n: usize,
    timeout: Duration,
    max_retries: usize,
    model: Option<Arc<RerankingModel>>,
}

impl Default for RerankRequestBuilder {
    fn default() -> Self {
        Self {
            query: None,
            documents: vec![],
            top_n: 5,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            model: None,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RerankRequestBuilderError {
    #[error("missing query from rerank request")]
    MissingQuery,

    #[error("at least one document is required in rerank request")]
    EmptyDocuments,

    #[error("missing model while sending reranking request")]
    MissingModel,
}

impl RerankRequestBuilder {
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn documents<I>(mut self, documents: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.documents.extend(documents);
        self
    }

    pub fn document(mut self, document: impl Into<String>) -> Self {
        self.documents.push(document.into());
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

    pub fn build(self) -> Result<RerankRequest, RerankRequestBuilderError> {
        if self.documents.is_empty() {
            return Err(RerankRequestBuilderError::EmptyDocuments);
        }

        Ok(RerankRequest {
            query: self.query.ok_or(RerankRequestBuilderError::MissingQuery)?,
            documents: self.documents,
            top_n: self.top_n,
            timeout: self.timeout,
            max_retries: self.max_retries,
            model: Arc::clone(&self.model.ok_or(RerankRequestBuilderError::MissingModel)?),
        })
    }
}

pub struct RerankResponse {
    pub rankings: Vec<Ranking>,
    pub ranked_documents: Vec<String>,
    pub raw_fields: Map<String, Value>,
}

pub struct Ranking {
    pub original_index: usize,
    pub score: f32,
    pub document: String,
}
