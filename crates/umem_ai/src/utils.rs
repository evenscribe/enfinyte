use crate::response_generators::ResponseGeneratorError;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use thiserror::Error;

pub fn is_retryable_error(e: &ResponseGeneratorError) -> bool {
    match e {
        ResponseGeneratorError::Http(e) => {
            tracing::warn!(
                "Retryable HTTP error occurred when communicating with AI provider: {}",
                e
            );
            e.is_timeout()
                || e.is_connect()
                || e.status().is_some_and(|s| {
                    s.is_server_error() || s == reqwest::StatusCode::TOO_MANY_REQUESTS
                })
        }
        ResponseGeneratorError::Deserialization(error) => {
            tracing::warn!(
                "Serialization error, AI Might have built a bad JSON output: {}",
                error
            );
            true
        }
        ResponseGeneratorError::TimeoutError(error) => {
            tracing::warn!(
                "Attempt Timeout Error, Your AI provider might be congested or your LLM is taking more time than you are expecting: {}",
                error
            );
            true
        }
        ResponseGeneratorError::Transient(e) => {
            tracing::error!(
                "Transient error occurred when communicating with AI provider: {}",
                e
            );
            false
        }
        ResponseGeneratorError::InvalidProviderResponse(e) => {
            tracing::error!("Invalid response from AI provider: {}", e);
            true
        }
        ResponseGeneratorError::EmptyProviderResponse => {
            tracing::error!("Empty response from AI provider");
            true
        }
        ResponseGeneratorError::BedrockConverseError(sdk_error) => {
            tracing::error!(
                "AWS Bedrock Converse SDK error occurred when communicating with AI provider: {}",
                sdk_error
            );
            true
        }
    }
}

#[derive(Error, Debug)]
pub enum BuildHeaderMapError {
    #[error(transparent)]
    InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),
    #[error(transparent)]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
}

pub fn build_header_map(headers: &[(String, String)]) -> Result<HeaderMap, BuildHeaderMapError> {
    Ok(headers
        .iter()
        .filter_map(|(key, value)| {
            match (
                HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                (Ok(name), Ok(val)) => Some((name, val)),
                (Err(e), _) => {
                    tracing::debug!("Skipping invalid header name '{}': {}", key, e);
                    None
                }
                (_, Err(e)) => {
                    tracing::debug!("Skipping invalid header value for '{}': {}", key, e);
                    None
                }
            }
        })
        .collect())
}
