mod pgvector;
mod qdrant;

use async_trait::async_trait;
use pgvector::{PgError, PgVector};
use qdrant::{Qdrant, QdrantError};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::OnceCell;
use umem_config::CONFIG;
use umem_core::{Memory, Query};

#[derive(Error, Debug)]
pub enum VectorStoreError {
    #[error("qdrant client failed with: {0}")]
    QdrantError(#[from] QdrantError),

    #[error("pg client failed with: {0}")]
    PgError(#[from] PgError),

    #[error("uuid action failed: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("serde action failed: {0}")]
    SerdeError(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, VectorStoreError>;

static VECTOR_STORE: OnceCell<Arc<dyn VectorStoreBase + Send + Sync>> = OnceCell::const_new();

pub struct VectorStore;

impl VectorStore {
    pub async fn get_store() -> Result<Arc<dyn VectorStoreBase + Send + Sync>> {
        VECTOR_STORE
            .get_or_try_init(|| async {
                match CONFIG.vector_store.clone() {
                    umem_config::VectorStore::Qdrant(qdrant) => {
                        let qdrant = Qdrant::new(qdrant).await?;
                        qdrant.create_collection().await?;
                        Ok(Arc::new(qdrant) as Arc<dyn VectorStoreBase + Send + Sync>)
                    }
                    umem_config::VectorStore::PgVector(pgvector) => {
                        let pgvector = PgVector::new(pgvector).await?;
                        pgvector.create_collection().await?;
                        Ok(Arc::new(pgvector) as Arc<dyn VectorStoreBase + Send + Sync>)
                    }
                }
            })
            .await
            .cloned()
    }
}

#[async_trait]
pub trait VectorStoreBase {
    async fn create_collection(&self) -> Result<()>;

    async fn delete_collection(&self) -> Result<()>;

    async fn reset(&self) -> Result<()>;

    async fn insert<'a>(&self, vectors: Vec<Vec<f32>>, payloads: Vec<&'a Memory>) -> Result<()>;

    async fn get(&self, vector_id: &str) -> Result<Memory>;

    async fn update(
        &self,
        vector_id: &str,
        vector: Option<Vec<f32>>,
        payload: Option<Memory>,
    ) -> Result<()>;

    async fn delete(&self, vector_id: &str) -> Result<()>;

    async fn list(&self, query: Query) -> Result<Vec<Memory>>;

    async fn search(&self, query: Query) -> Result<Vec<Memory>>;
}
