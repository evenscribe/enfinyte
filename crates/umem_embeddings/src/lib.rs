use async_trait::async_trait;
use lazy_static::lazy_static;
use reqwest::Client;
mod cloudflare;
use cloudflare::{Cloudflare, CloudflareError};
use umem_config::CONFIG;

use std::sync::Arc;
use thiserror::Error;
use tokio::sync::OnceCell;

#[derive(Debug, Error)]
pub enum EmbedderError {
    #[error("cloudflare embedder failed : {0}")]
    CloudflareError(#[from] CloudflareError),

    #[error("reqwest failed : {0}")]
    ReqwestError(#[from] reqwest::Error),
}

lazy_static! {
    static ref client: Client = Client::new();
}

static EMBEDDER: OnceCell<Arc<dyn EmbedderBase + Send + Sync>> = OnceCell::const_new();

#[derive(Copy, Clone)]
pub struct Embedder;

type Result<T> = std::result::Result<T, EmbedderError>;

impl Embedder {
    pub async fn get_embedder() -> Result<Arc<dyn EmbedderBase + Send + Sync>> {
        EMBEDDER
            .get_or_try_init(|| async {
                match CONFIG.embedding_model.clone() {
                    umem_config::EmbeddingModel::Cloudflare(config) => {
                        let cloudflare = Cloudflare::new(config);
                        Ok(Arc::new(cloudflare) as Arc<dyn EmbedderBase + Send + Sync>)
                    }
                }
            })
            .await
            .cloned()
    }
}

#[async_trait]
pub trait EmbedderBase {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
    async fn generate_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>>;
}
