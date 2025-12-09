use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use reqwest::Client;
mod cloudflare;
use cloudflare::Cloudflare;
use umem_config::CONFIG;

use std::sync::Arc;
use tokio::sync::OnceCell;

lazy_static! {
    static ref client: Client = Client::new();
}

static EMBEDDER: OnceCell<Arc<dyn EmbedderBase + Send + Sync>> = OnceCell::const_new();

pub struct Embedder;

impl Embedder {
    pub async fn get_embedder() -> Result<Arc<dyn EmbedderBase + Send + Sync>> {
        EMBEDDER
            .get_or_try_init(|| async {
                match CONFIG.embedder.clone() {
                    umem_config::Embedder::Cloudflare(config) => {
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
    async fn generate_embeddings(&self, text: Vec<&str>) -> Result<Vec<Vec<f32>>>;
}
