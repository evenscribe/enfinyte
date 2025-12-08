mod pgvector;
mod qdrant;

use anyhow::Result;
use async_trait::async_trait;
use pgvector::PgVector;
use qdrant::Qdrant;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;
use umem_config::CONFIG;
use umem_proto::generated;

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

    async fn insert(&self, vectors: Vec<Vec<f32>>, payloads: Vec<generated::Memory>) -> Result<()>;

    async fn get(&self, vector_id: &str) -> Result<generated::Memory>;

    async fn update(
        &self,
        vector_id: &str,
        vector: Option<Vec<f32>>,
        payload: Option<FxHashMap<String, serde_json::Value>>,
    ) -> Result<()>;

    async fn delete(&self, vector_id: &str) -> Result<()>;

    async fn list(
        &self,
        filters: Option<FxHashMap<&str, String>>,
        limit: u32,
    ) -> Result<Vec<generated::Memory>>;

    async fn search(
        &self,
        query_vector: Vec<f32>,
        filters: Option<FxHashMap<&str, String>>,
        limit: u64,
    ) -> Result<Vec<generated::Memory>>;
}
