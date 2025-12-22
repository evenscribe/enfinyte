use crate::response_generators::GenerateTextError;

pub fn is_retryable_error(e: &GenerateTextError) -> bool {
    match e {
        GenerateTextError::Http(e) => {
            e.is_timeout()
                || e.is_connect()
                || e.status().is_some_and(|s| {
                    s.is_server_error() || s == reqwest::StatusCode::TOO_MANY_REQUESTS
                })
        }
        GenerateTextError::Other(_) => false,
    }
}
