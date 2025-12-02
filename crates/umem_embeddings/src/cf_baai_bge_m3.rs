use crate::client;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use umem_config::CONFIG;

const CF_BAAI_BGE_M3_EMBEDER_NAME: &str = "@cf/baai/bge-m3";

pub struct CfBaaiBgeM3Embeder;

impl CfBaaiBgeM3Embeder {
    pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/{}",
            CONFIG.cloudflare.account_id, CF_BAAI_BGE_M3_EMBEDER_NAME
        );
        let request_body = EmbeddingRequest { text: vec![text] };
        let response = client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", CONFIG.cloudflare.api_token),
            )
            .json(&request_body)
            .send()
            .await?;
        let mut embedding_response: EmbeddingResponse = response.json().await?;
        if !embedding_response.success {
            bail!("{:?}", embedding_response.errors);
        }
        Ok(std::mem::take(&mut embedding_response.result.data[0]))
    }

    pub async fn generate_embeddings_bulk(texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/{}",
            CONFIG.cloudflare.account_id, CF_BAAI_BGE_M3_EMBEDER_NAME
        );
        let request_body = EmbeddingRequest { text: texts };
        let response = client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", CONFIG.cloudflare.api_token),
            )
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
