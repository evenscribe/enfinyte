use crate::{VectorStoreBase, VectorStoreError};
use async_trait::async_trait;
use qdrant_client::{
    qdrant::{
        Condition, CreateCollectionBuilder, CreateFieldIndexCollectionBuilder, DeletePointsBuilder,
        Distance, FieldType, Filter, GetPointsBuilder, PointStruct, PointVectors, PointsIdsList,
        Query, QueryPointsBuilder, RetrievedPoint, ScalarQuantizationBuilder, ScoredPoint,
        ScrollPointsBuilder, SetPayloadPointsBuilder, UpdatePointVectorsBuilder,
        UpsertPointsBuilder, UuidIndexParamsBuilder, VectorParamsBuilder,
    },
    Payload,
};
use rustc_hash::FxHashMap;
use serde_json::json;
use std::iter::zip;
use thiserror::Error;
use umem_core::Memory;

pub struct Qdrant {
    client: qdrant_client::Qdrant,
    collection_name: String,
    embedding_model_dims: u16,
    chunk_size: u16,
}

#[derive(Error, Debug)]
pub enum QdrantError {
    #[error("Point with ID '{0}' not found in collection")]
    PointNotFound(String),

    #[error("Qdrant client error: {0}")]
    ClientError(#[from] qdrant_client::QdrantError),
}

impl From<qdrant_client::QdrantError> for VectorStoreError {
    fn from(value: qdrant_client::QdrantError) -> Self {
        Into::<QdrantError>::into(value).into()
    }
}

type Result<T> = std::result::Result<T, QdrantError>;

impl Qdrant {
    pub async fn new(qdrant: umem_config::Qdrant) -> Result<Self> {
        let client = qdrant_client::Qdrant::from_url(&qdrant.url)
            .api_key(qdrant.key)
            .build()?;

        Ok(Self {
            client,
            collection_name: qdrant.collection_name,
            embedding_model_dims: qdrant.embedding_model_dimensions,
            chunk_size: qdrant.chunk_size,
        })
    }

    async fn create_indexes(&self) -> Result<()> {
        self.client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(
                    &self.collection_name,
                    "user_id",
                    FieldType::Uuid,
                )
                .field_index_params(UuidIndexParamsBuilder::default().is_tenant(true)),
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl VectorStoreBase for Qdrant {
    async fn create_collection(&self) -> crate::Result<()> {
        if self.client.collection_exists(&self.collection_name).await? {
            return Ok(());
        }

        self.client
            .create_collection(
                CreateCollectionBuilder::new(&self.collection_name)
                    .vectors_config(VectorParamsBuilder::new(
                        self.embedding_model_dims.into(),
                        Distance::Cosine,
                    ))
                    .quantization_config(ScalarQuantizationBuilder::default()),
            )
            .await?;
        self.create_indexes().await?;
        Ok(())
    }

    async fn delete_collection(&self) -> crate::Result<()> {
        self.client.delete_collection(&self.collection_name).await?;
        Ok(())
    }

    async fn reset(&self) -> crate::Result<()> {
        self.delete_collection().await?;
        self.create_collection().await?;
        Ok(())
    }

    async fn insert<'a>(
        &self,
        vectors: Vec<Vec<f32>>,
        payloads: Vec<&'a Memory>,
    ) -> crate::Result<()> {
        let mut points: Vec<PointStruct> = Vec::with_capacity(vectors.len());
        for (vector, payload) in zip(vectors, payloads) {
            let point_id = payload.get_id();
            let payload = Payload::try_from(json!(payload))?;
            points.push(PointStruct::new(point_id.to_string(), vector, payload));
        }

        self.client
            .upsert_points_chunked(
                UpsertPointsBuilder::new(&self.collection_name, points).wait(true),
                self.chunk_size.into(),
            )
            .await?;
        Ok(())
    }

    async fn get(&self, vector_id: &str) -> crate::Result<Memory> {
        let result = self
            .client
            .get_points(
                GetPointsBuilder::new(&self.collection_name, vec![vector_id.into()])
                    .with_payload(true),
            )
            .await?
            .result;

        if result.is_empty() {
            return Err(QdrantError::PointNotFound(vector_id.to_string()))?;
        }

        let string = serde_json::to_string(&result[0].payload)?;
        let memory: Memory = serde_json::from_str(string.as_str())?;

        Ok(memory)
    }

    async fn update(
        &self,
        vector_id: &str,
        vector: Option<Vec<f32>>,
        payload: Option<FxHashMap<String, serde_json::Value>>,
    ) -> crate::Result<()> {
        if let Some(vector) = vector {
            self.client
                .update_vectors(UpdatePointVectorsBuilder::new(
                    &self.collection_name,
                    vec![PointVectors {
                        id: Some(vector_id.into()),
                        vectors: Some(vector.into()),
                    }],
                ))
                .await?;
        }

        if let Some(payload) = payload {
            self.client
                .set_payload(
                    SetPayloadPointsBuilder::new(
                        &self.collection_name,
                        Payload::try_from(json!(payload))?,
                    )
                    .points_selector(PointsIdsList {
                        ids: vec![vector_id.into()],
                    }),
                )
                .await?;
        }
        Ok(())
    }

    async fn delete(&self, vector_id: &str) -> crate::Result<()> {
        self.client
            .delete_points(
                DeletePointsBuilder::new(&self.collection_name)
                    .points(PointsIdsList {
                        ids: vec![vector_id.into()],
                    })
                    .wait(true),
            )
            .await?;

        Ok(())
    }

    async fn list(
        &self,
        filters: Option<FxHashMap<&str, String>>,
        limit: u32,
    ) -> crate::Result<Vec<Memory>> {
        let mut scroll = ScrollPointsBuilder::new(&self.collection_name)
            .limit(limit)
            .with_payload(true);

        if let Some(filters) = filters {
            scroll = scroll.filter(Filter::must(
                filters
                    .into_iter()
                    .map(|(field, value)| Condition::matches(field, value))
                    .collect::<Vec<Condition>>(),
            ));
        }

        self.client
            .scroll(scroll)
            .await?
            .result
            .into_iter()
            .map(|RetrievedPoint { payload, .. }| {
                let string = serde_json::to_string(&payload)?;
                let memory: Memory = serde_json::from_str(&string)?;
                Ok(memory)
            })
            .collect()
    }

    async fn search(
        &self,
        query_vector: Vec<f32>,
        filters: Option<FxHashMap<&str, String>>,
        limit: u64,
    ) -> crate::Result<Vec<Memory>> {
        let mut query = QueryPointsBuilder::new(&self.collection_name)
            .query(Query::new_nearest(query_vector))
            .limit(limit)
            .with_payload(true);

        if let Some(filters) = filters {
            query = query.filter(Filter::must(
                filters
                    .into_iter()
                    .map(|(field, value)| Condition::matches(field, value))
                    .collect::<Vec<Condition>>(),
            ));
        }

        self.client
            .query(query)
            .await?
            .result
            .into_iter()
            .map(|ScoredPoint { payload, .. }| {
                let string = serde_json::to_string(&payload)?;
                let memory: Memory = serde_json::from_str(&string)?;
                Ok(memory)
            })
            .collect()
    }
}
