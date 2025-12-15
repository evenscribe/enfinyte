use crate::Provider;
use std::sync::Arc;

pub struct GenerateTextRequest {
    pub model: String,
    pub provider: Arc<Provider>,
    pub system: String,
    pub prompt: String,
    pub max_output_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub top_k: Option<usize>,
    pub presence_penalty: Option<f32>,
    pub seed: Option<u64>,
    pub max_retries: usize,
    pub headers: Option<Vec<(String, String)>>,
}

pub struct GenerateTextResponse {
    pub text: String,
}

pub fn generate_text(request: GenerateTextRequest) -> GenerateTextResponse {
    unimplemented!()
}
