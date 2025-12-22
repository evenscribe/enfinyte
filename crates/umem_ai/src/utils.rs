pub fn is_retryable_error(e: &reqwest::Error) -> bool {
    e.is_timeout()
        || e.is_connect()
        || e.status()
            .is_some_and(|s| s.is_server_error() || s == reqwest::StatusCode::TOO_MANY_REQUESTS)
}
