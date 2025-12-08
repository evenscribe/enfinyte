use anyhow::Result;

use crate::HashMap;

#[async_trait::async_trait]
pub trait VectorDB {
    async fn get(
        &self,
        ids: Option<&[String]>,
        filter: Option<&[(String, String)]>,
        limit: Option<usize>,
    ) -> Result<VectorDBGetResponse>;
    async fn delete(&self, document_ids: &[String]) -> Result<()>;
    async fn count(&self) -> Result<usize>;
    async fn add(
        &self,
        documents: &[String],
        metadatas: &[HashMap<String, String>],
        ids: &[String],
    ) -> Result<()>;
}

pub struct VectorDBGetResponse {
    pub ids: Vec<String>,
    pub metadatas: Vec<HashMap<String, String>>,
}
