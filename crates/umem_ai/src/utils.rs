use crate::response_generators::ResponseGeneratorError;

pub fn is_retryable_error(e: &ResponseGeneratorError) -> bool {
    match e {
        ResponseGeneratorError::Http(e) => {
            e.is_timeout()
                || e.is_connect()
                || e.status().is_some_and(|s| {
                    s.is_server_error() || s == reqwest::StatusCode::TOO_MANY_REQUESTS
                })
        }
        ResponseGeneratorError::Transient(_) => false,
    }
}
