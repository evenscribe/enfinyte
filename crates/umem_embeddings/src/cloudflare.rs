use crate::{client, EmbedderBase};
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
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
            bail!("{:?}", embedding_response.errors);
        }
        Ok(std::mem::take(&mut embedding_response.result.data[0]))
    }

    async fn generate_embeddings(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
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
            bail!("{:?}", embedding_response.errors);
        }
        Ok(embedding_response.result.data)
    }
}
