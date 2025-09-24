use anyhow::Result;
use lazy_static::lazy_static;
use serde_json::json;
use tokio::sync::OnceCell;
use umem_embeddings::Embedder;
use umem_proto_generated::generated;
use umem_vector::QdrantVectorStore;
use uuid::Uuid;

static MEMORY_STORE: OnceCell<QdrantVectorStore> = OnceCell::const_new();

async fn get_memory_store() -> &'static QdrantVectorStore {
    MEMORY_STORE
        .get_or_init(|| async {
            QdrantVectorStore::new(
                &std::env::var("QDRANT_URL").expect("QDRANT_URL not set"),
                &std::env::var("QDRANT_KEY").expect("QDRANT_KEY not set"),
                &std::env::var("QDRANT_COLLECTION_NAME").expect("QDRANT_COLLECTION_NAME not set"),
            )
            .await
            .expect("qdrant client failed to intialize")
        })
        .await
}

lazy_static! {
    static ref CFEmbeder: umem_embeddings::CfBaaiBgeM3Embeder =
        umem_embeddings::CfBaaiBgeM3Embeder::new(
            std::env::var("CLOUDFLARE_ACCOUNT_ID").expect("CLOUDFLARE_ACCOUNT_ID not set"),
            std::env::var("CLOUDFLARE_API_TOKEN").expect("CLOUDFLARE_API_TOKEN not set"),
        );
}

#[derive(Debug, Default)]
pub struct MemoryController;

impl MemoryController {
    pub async fn add_memory(memory: generated::Memory) -> Result<generated::Memory> {
        let memory_store = get_memory_store().await;

        let memory = generated::Memory {
            memory_id: Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now().timestamp(),
            ..memory
        };

        let vectors = CFEmbeder
            .generate_embedding(memory.content.as_str())
            .await?;
        memory_store
            .insert_embedding(memory.clone(), vectors)
            .await?;
        Ok(memory)
    }

    pub async fn add_memory_bulk(memory_bulk: generated::MemoryBulk) -> Result<()> {
        let memory_store = get_memory_store().await;

        let texts = memory_bulk
            .memories
            .iter()
            .map(|memory| memory.content.as_str())
            .collect();

        let vectors: Vec<Vec<f32>> = CFEmbeder.generate_embeddings_bulk(texts).await?;

        memory_store
            .insert_embeddings_bulk(
                std::iter::zip(memory_bulk.memories, vectors).collect::<Vec<_>>(),
            )
            .await?;

        Ok(())
    }

    pub async fn update_memory(
        update_memory_parameters: generated::UpdateMemoryParameters,
    ) -> Result<()> {
        let memory_store = get_memory_store().await;

        memory_store
            .update_point(
                &update_memory_parameters.memory_id.clone(),
                None,
                Some(update_memory_parameters),
            )
            .await?;

        Ok(())
    }

    pub async fn delete_memory(
        delete_memory_parameters: generated::DeleteMemoryParameters,
    ) -> Result<()> {
        let memory_store = get_memory_store().await;

        memory_store
            .delete_point(delete_memory_parameters.memory_id.as_str())
            .await?;

        Ok(())
    }

    /// Qdrant Queries
    pub async fn get_memories_by_query(
        get_memories_by_query_parameters: generated::GetMemoriesByQueryParameters,
    ) -> Result<generated::MemoryBulk> {
        let memory_store = get_memory_store().await;

        let vector = CFEmbeder
            .generate_embedding(&get_memories_by_query_parameters.query)
            .await?;

        let search_response = memory_store
            .search_with_vector(
                vector,
                Some(1),
                &get_memories_by_query_parameters.user_id,
                Some("created_at".into()),
            )
            .await?;

        Ok(generated::MemoryBulk {
            memories: search_response
                .result
                .into_iter()
                .map(|scored_point| {
                    serde_json::from_value(json!(scored_point.payload))
                        .expect("Payload to Memory parse failed.")
                })
                .collect::<Vec<_>>(),
        })
    }

    pub async fn get_memories_by_user_id(
        get_memories_by_user_id_parameters: generated::GetMemoriesByUserIdParameters,
    ) -> Result<generated::MemoryBulk> {
        let memory_store = get_memory_store().await;

        let search_response = memory_store
            .search_with_payload(
                vec![(
                    "user_id".to_string(),
                    get_memories_by_user_id_parameters.user_id,
                )],
                Some(1),
            )
            .await?;

        Ok(generated::MemoryBulk {
            memories: search_response
                .result
                .into_iter()
                .map(|scored_point| {
                    serde_json::from_value(json!(scored_point.payload))
                        .expect("Payload to Memory parse failed.")
                })
                .collect::<Vec<_>>(),
        })
    }

    pub async fn get_memories_by_memory_id(memory_id: String) -> Result<generated::Memory> {
        let memory_store = get_memory_store().await;

        let search_response = memory_store
            .search_with_payload(vec![("memory_id".to_string(), memory_id)], Some(1))
            .await?;

        Ok(serde_json::from_value(json!(
            search_response
                .result
                .first()
                .expect("Get by memory_id somehow failed")
                .payload
        ))
        .expect("Payload to Memory parse failed."))
    }
}
