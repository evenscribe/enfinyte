use crate::{client, EmbedderBase};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CloudflareError {
    #[error("text to embed cannot be empty.")]
    EmptyText,

    #[error("embedding failed: {0:?}")]
    EmbeddingFailed(Vec<String>),
}

#[derive(Serialize)]
struct EmbeddingRequest<'em> {
    text: Vec<&'em str>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    result: EmbeddingResult,
    errors: Vec<String>,
    success: bool,
}

#[derive(Deserialize)]
struct EmbeddingResult {
    data: Vec<Vec<f32>>,
}

pub struct Cloudflare {
    pub account_id: String,
    pub api_token: String,
    pub model: String,
}

impl Cloudflare {
    pub fn new(config: umem_config::Cloudflare) -> Self {
        Self {
            account_id: config.account_id,
            api_token: config.api_token,
            model: config.model,
        }
    }
}

#[async_trait]
impl EmbedderBase for Cloudflare {
    async fn generate_embedding(&self, text: &str) -> crate::Result<Vec<f32>> {
        if text.trim().is_empty() {
            Err(CloudflareError::EmptyText)?;
        }
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/{}",
            self.account_id, self.model
        );
        let request_body = EmbeddingRequest { text: vec![text] };
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&request_body)
            .send()
            .await?;
        let mut embedding_response: EmbeddingResponse = response.json().await?;
        if !embedding_response.success {
            Err(CloudflareError::EmbeddingFailed(embedding_response.errors))?;
        }
        if embedding_response.result.data.is_empty() {
            Err(CloudflareError::EmbeddingFailed(vec!["No embedding data
  returned"
                .to_string()]))?;
        }

        Ok(std::mem::take(&mut embedding_response.result.data[0]))
    }

    async fn generate_embeddings(&self, texts: Vec<&str>) -> crate::Result<Vec<Vec<f32>>> {
        if texts.is_empty() || texts.iter().any(|text| text.trim().is_empty()) {
            Err(CloudflareError::EmptyText)?;
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/{}",
            self.account_id, self.model
        );
        let request_body = EmbeddingRequest { text: texts };
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&request_body)
            .send()
            .await?;
        let embedding_response: EmbeddingResponse = response.json().await?;
        if !embedding_response.success {
            Err(CloudflareError::EmbeddingFailed(embedding_response.errors))?;
        }

        if embedding_response.result.data.is_empty() {
            Err(CloudflareError::EmbeddingFailed(vec!["No embedding data
  returned"
                .to_string()]))?;
        }

        Ok(embedding_response.result.data)
    }
}
