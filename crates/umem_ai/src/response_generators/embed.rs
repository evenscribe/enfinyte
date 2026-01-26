use crate::{
    ResponseGeneratorError,
    models::EmbeddingModel,
    utils::{self, is_retryable_error},
};
use async_trait::async_trait;
use backon::{ExponentialBuilder, Retryable};
use reqwest::header::HeaderMap;
use std::{sync::Arc, time::Duration};

#[async_trait]
pub trait Embeds {
    async fn embed(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, ResponseGeneratorError>;
}

pub async fn embed(request: EmbeddingRequest) -> Result<EmbeddingResponse, ResponseGeneratorError> {
    let per_request_timeout = request.timeout;
    let max_retries = request.max_retries;
    let total_delay = per_request_timeout.mul_f32(max_retries as f32 / 2.0);

    let generation = || {
        let model = Arc::clone(&request.model);
        let provider = Arc::clone(&model.provider);
        let request = request.clone();

        async move {
            tokio::time::timeout(per_request_timeout, provider.do_embed(request))
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

#[derive(Clone, Debug, typed_builder::TypedBuilder)]
pub struct EmbeddingRequest {
    pub model: Arc<EmbeddingModel>,

    #[builder(setter(transform = |value: impl IntoIterator<Item = String>|
        value.into_iter().collect())
    )]
    pub input: Vec<String>,

    #[builder(default = 3_usize)]
    pub max_retries: usize,

    #[builder(default = 1000_usize)]
    pub max_parallels: usize,

    #[builder(default, setter(transform = |value: Vec<(String, String)>|
           utils::build_header_map(value.as_slice()).unwrap_or_default()
    ))]
    pub custom_headers: HeaderMap,

    #[builder(default = Duration::from_secs(60))]
    pub timeout: Duration,

    #[builder(default = 1024)]
    pub dimensions: usize,

    #[builder(default = true)]
    pub normalize: bool,
}

#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
}
