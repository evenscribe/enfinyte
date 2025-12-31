use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::OnceCell;
use typed_builder::TypedBuilder;
use umem_config::CONFIG;
use umem_core::Memory;

mod pinecone;

pub use pinecone::*;

static RERANKER: OnceCell<Arc<dyn RerankerBase + Send + Sync>> = OnceCell::const_new();

#[derive(Copy, Clone)]
pub struct Reranker;

impl Reranker {
    pub async fn get_reranker() -> Result<Arc<dyn RerankerBase + Send + Sync>, RerankError> {
        RERANKER
            .get_or_try_init(|| async {
                match CONFIG.reranker.clone() {
                    umem_config::Reranker::Pinecone(config) => {
                        let pinecone = Pinecone::new(config);
                        Ok(Arc::new(pinecone) as Arc<dyn RerankerBase + Send + Sync>)
                    }
                }
            })
            .await
            .cloned()
    }
}

#[derive(TypedBuilder, Debug, Default, Deserialize)]
pub struct RerankOptions {
    #[builder(default = None)]
    pub pinecone: Option<PineconeOptions>,
}

#[derive(Error, Debug)]
pub enum RerankError {
    #[error("pinecone failed with : {0}")]
    PineconeError(#[from] PineconeError),
}

#[derive(Debug, Deserialize)]
pub struct RerankResponse {
    pub data: Vec<DataRow>,
}

#[derive(Debug, Deserialize)]
pub struct DataRow {
    pub index: usize,
    pub score: f32,
}

#[async_trait]
pub trait RerankerBase {
    async fn rank(
        &self,
        query: String,
        documents: &[Memory],
        options: Option<RerankOptions>,
    ) -> Result<RerankResponse, RerankError>;
}
