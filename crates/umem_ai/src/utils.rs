use std::collections::HashMap;

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
        ResponseGeneratorError::Deserialization(error, response) => {
            tracing::warn!(
                "Serialization error, AI Might have built a bad JSON output: \n Error: {} \n Received Response: {}",
                error,
                response
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
        ResponseGeneratorError::StructuredRerankDocumentsSerializationError(e) => {
            tracing::error!("Structured rerank documents serialization error: {}", e);
            false
        }
        ResponseGeneratorError::InvalidArgumentsProvided(e) => {
            tracing::error!("Invalid arguments provided: {}", e);
            false
        }
        ResponseGeneratorError::BedrockAgentRerankCommandSendError(e) => {
            tracing::error!("Bedrock agent rerank command send error: {}", e);
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

pub fn json_to_aws_smithy_document(value: serde_json::Value) -> aws_smithy_types::Document {
    match value {
        serde_json::Value::Null => aws_smithy_types::Document::Null,
        serde_json::Value::Bool(b) => aws_smithy_types::Document::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                aws_smithy_types::Document::from(i)
            } else if let Some(u) = n.as_u64() {
                aws_smithy_types::Document::from(u)
            } else if let Some(f) = n.as_f64() {
                aws_smithy_types::Document::from(f)
            } else {
                aws_smithy_types::Document::Null
            }
        }
        serde_json::Value::String(s) => aws_smithy_types::Document::String(s),
        serde_json::Value::Array(arr) => aws_smithy_types::Document::Array(
            arr.into_iter().map(json_to_aws_smithy_document).collect(),
        ),
        serde_json::Value::Object(obj) => aws_smithy_types::Document::Object(
            obj.into_iter()
                .map(|(k, v)| (k, json_to_aws_smithy_document(v)))
                .collect::<HashMap<_, _>>(),
        ),
    }
}

pub fn aws_smithy_document_to_json(doc: &aws_smithy_types::Document) -> serde_json::Value {
    match doc {
        aws_smithy_types::Document::Null => serde_json::Value::Null,
        aws_smithy_types::Document::Bool(b) => serde_json::Value::Bool(*b),
        aws_smithy_types::Document::Number(n) => match n {
            aws_smithy_types::Number::PosInt(u) => serde_json::Value::Number((*u).into()),
            aws_smithy_types::Number::NegInt(i) => serde_json::Value::Number((*i).into()),
            aws_smithy_types::Number::Float(f) => serde_json::Number::from_f64((*f).into())
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
        },
        aws_smithy_types::Document::String(s) => serde_json::Value::String(s.clone()),
        aws_smithy_types::Document::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(aws_smithy_document_to_json).collect())
        }
        aws_smithy_types::Document::Object(obj) => serde_json::Value::Object(
            obj.into_iter()
                .map(|(k, v)| (k.clone(), aws_smithy_document_to_json(v)))
                .collect(),
        ),
    }
}
