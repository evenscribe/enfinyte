use crate::response_generators::ResponseGeneratorError;

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
        ResponseGeneratorError::Serialization(error) => {
            tracing::warn!(
                "Serialization error, AI Might have built a bad JSON output: {}",
                error
            );
            true
        }
        ResponseGeneratorError::Transient(_) => false,
    }
}
