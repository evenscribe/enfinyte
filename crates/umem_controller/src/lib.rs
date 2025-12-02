mod builders;

use anyhow::Result;
pub use builders::{CreateMemoryRequestBuilder, UpdateMemoryRequestBuilder};
use rustc_hash::FxHashMap;
use umem_embeddings::CfBaaiBgeM3Embeder;
use umem_proto::generated;
use umem_vector_store::VectorStore;

#[derive(Debug, Default)]
pub struct MemoryController;

impl MemoryController {
    pub async fn create(request: CreateMemoryRequestBuilder) -> Result<generated::Memory> {
        let memory_store = VectorStore::get_store().await?;
        let memory: generated::Memory = request.build();
        let vector = CfBaaiBgeM3Embeder::generate_embedding(&memory.content).await?;
        memory_store
            .insert(vec![vector], vec![memory.clone()])
            .await?;
        Ok(memory)
    }

    pub async fn get(id: String) -> Result<generated::Memory> {
        let memory_store = VectorStore::get_store().await?;
        memory_store.get(id.as_str()).await
    }

    pub async fn update(request: UpdateMemoryRequestBuilder) -> Result<()> {
        let memory_store = VectorStore::get_store().await?;
        let vector = if let Some(content) = &request.content {
            Some(CfBaaiBgeM3Embeder::generate_embedding(content).await?)
        } else {
            None
        };
        let payload: FxHashMap<String, serde_json::Value> = [
            request.content.map(|v| ("content".to_string(), v.into())),
            request.tags.map(|v| ("tags".to_string(), v.into())),
            request.priority.map(|v| ("priority".to_string(), v.into())),
            request
                .parent_id
                .map(|v| ("parent_id".to_string(), v.into())),
        ]
        .into_iter()
        .flatten()
        .collect();

        memory_store
            .update(request.id.as_str(), vector, Some(payload))
            .await
    }

    pub async fn delete(id: String) -> Result<()> {
        let memory_store = VectorStore::get_store().await?;
        memory_store.delete(id.as_str()).await
    }

    pub async fn list(user_id: String) -> Result<Vec<generated::Memory>> {
        let memory_store = VectorStore::get_store().await?;
        let filters = FxHashMap::from_iter([("user_id", user_id)]);
        memory_store.list(Some(filters), 1000).await
    }

    pub async fn search(user_id: String, query: String) -> Result<Vec<generated::Memory>> {
        let memory_store = VectorStore::get_store().await?;
        let filters = FxHashMap::from_iter([("user_id", user_id)]);
        let vector = CfBaaiBgeM3Embeder::generate_embedding(query.as_str()).await?;
        memory_store.search(vector, Some(filters), 1000).await
    }
}
