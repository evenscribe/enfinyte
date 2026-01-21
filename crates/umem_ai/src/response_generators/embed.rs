use crate::{EmbeddingModel, ResponseGeneratorError, utils::is_retryable_error};
use backon::{ExponentialBuilder, Retryable};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

pub async fn embed(
    mut request: EmbeddingRequest,
) -> Result<EmbeddingResponse, ResponseGeneratorError> {
    let per_request_timeout = request.timeout;
    let max_retries = request.max_retries;
    let total_delay = per_request_timeout.mul_f32(max_retries as f32 / 2.0);

    let generation = || {
        let model = Arc::clone(&request.model);
        let provider = Arc::clone(&model.provider);
        let request = EmbeddingRequest {
            model: Arc::clone(&model),
            input: std::mem::take(&mut request.input),
            custom_headers: std::mem::take(&mut request.custom_headers),
            timeout: request.timeout.clone(),
            max_retries,
            max_parallels: request.max_parallels,
        };

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

#[derive(Clone)]
pub struct EmbeddingRequest {
    pub model: Arc<EmbeddingModel>,
    pub input: Vec<String>,
    pub max_retries: usize,
    pub max_parallels: usize,
    pub custom_headers: Vec<(String, String)>,
    pub timeout: Duration,
}

pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
}
