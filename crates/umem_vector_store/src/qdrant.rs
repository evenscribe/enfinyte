use crate::{VectorStoreBase, VectorStoreError};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use qdrant_client::{
    qdrant::{
        Condition, CreateCollectionBuilder, CreateFieldIndexCollectionBuilder, DatetimeRange,
        DeletePointsBuilder, Distance, FieldType, Filter, GetPointsBuilder, PointStruct,
        PointVectors, PointsIdsList, Query, QueryPointsBuilder, Range, RetrievedPoint,
        ScalarQuantizationBuilder, ScoredPoint, ScrollPointsBuilder, SetPayloadPointsBuilder,
        UpdatePointVectorsBuilder, UpsertPointsBuilder, UuidIndexParamsBuilder,
        VectorParamsBuilder,
    },
    Payload,
};
use rustc_hash::FxHashMap;
use serde_json::json;
use std::{iter::zip, time};
use thiserror::Error;
use umem_core::{LifecycleState, Memory};

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

    #[error("Vector must be supplied for search.")]
    VectorNotSupplied,

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

    fn filter_include_archived(conds: &mut Vec<Condition>, query: &umem_core::Query) {
        if !query.include_archived() {
            conds.push(Condition::matches(
                "lifecycle",
                LifecycleState::Active.as_str().to_owned(),
            ));
        }
    }

    fn filter_context(conds: &mut Vec<Condition>, query: &umem_core::Query) {
        if let Some(user_id) = query.context().user_id() {
            conds.push(Condition::matches("context.user_id", user_id.to_string()));
        }
        if let Some(agent_id) = query.context().agent_id() {
            conds.push(Condition::matches("context.agent_id", agent_id.to_string()));
        }
        if let Some(run_id) = query.context().run_id() {
            conds.push(Condition::matches("context.run_id", run_id.to_string()));
        }
    }

    fn filter_kinds(conds: &mut Vec<Condition>, query: &umem_core::Query) {
        if let Some(kinds) = query.kinds() {
            conds.push(Condition::matches(
                "kind",
                kinds
                    .iter()
                    .map(|kind| kind.as_str().to_string())
                    .collect::<Vec<String>>(),
            ));
        }
    }

    fn filter_tags(conds: &mut Vec<Condition>, query: &umem_core::Query) {
        if let Some(tags) = query.tags() {
            conds.push(Condition::matches(
                "content.tags[]",
                tags.iter()
                    .map(|tag| tag.to_owned())
                    .collect::<Vec<String>>(),
            ));
        }
    }

    fn filter_temporal(conds: &mut Vec<Condition>, query: &umem_core::Query) {
        if let Some(temporal) = query.temporal() {
            if temporal.has_created_range() {
                conds.push(Condition::datetime_range(
                    "temporal.created_at",
                    DatetimeRange {
                        lt: temporal.created_range().1.map(|t| {
                            let dt = Utc.timestamp_opt(t, 0).unwrap();
                            let st: time::SystemTime = dt.into();
                            st.into()
                        }),
                        gt: temporal.created_range().0.map(|t| {
                            let dt = Utc.timestamp_opt(t, 0).unwrap();
                            let st: time::SystemTime = dt.into();
                            st.into()
                        }),
                        gte: None,
                        lte: None,
                    },
                ));
            }
            if temporal.has_updated_range() {
                conds.push(Condition::datetime_range(
                    "temporal.updated_at",
                    DatetimeRange {
                        lt: temporal.updated_range().1.map(|t| {
                            let dt = Utc.timestamp_opt(t, 0).unwrap();
                            let st: time::SystemTime = dt.into();
                            st.into()
                        }),
                        gt: temporal.updated_range().0.map(|t| {
                            let dt = Utc.timestamp_opt(t, 0).unwrap();
                            let st: time::SystemTime = dt.into();
                            st.into()
                        }),
                        gte: None,
                        lte: None,
                    },
                ));
            }
        }
    }

    fn filter_signals(conds: &mut Vec<Condition>, query: &umem_core::Query) {
        if let Some(signal) = query.signals() {
            if let Some(salience) = signal.min_salience() {
                conds.push(Condition::range(
                    "signals.salience",
                    Range {
                        lt: None,
                        gt: Some(salience.into()),
                        gte: None,
                        lte: None,
                    },
                ));
            }

            if let Some(certainty) = signal.min_certainty() {
                conds.push(Condition::range(
                    "signals.certainty",
                    Range {
                        lt: None,
                        gt: Some(certainty.into()),
                        gte: None,
                        lte: None,
                    },
                ));
            }
        }
    }

    fn create_filter(query: &umem_core::Query) -> Filter {
        let mut conds = vec![];

        Self::filter_include_archived(&mut conds, query);
        Self::filter_context(&mut conds, query);
        Self::filter_kinds(&mut conds, query);
        Self::filter_tags(&mut conds, query);
        Self::filter_temporal(&mut conds, query);
        Self::filter_signals(&mut conds, query);

        Filter::must(conds)
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

    async fn list(&self, query: umem_core::Query) -> crate::Result<Vec<Memory>> {
        let mut scroll = ScrollPointsBuilder::new(&self.collection_name)
            .limit(query.limit())
            .with_payload(true);

        scroll = scroll.filter(Qdrant::create_filter(&query));

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

    async fn search(&self, query: umem_core::Query) -> crate::Result<Vec<Memory>> {
        if query.vector().is_none() {
            return Err(QdrantError::VectorNotSupplied)?;
        }

        let mut query_builder = QueryPointsBuilder::new(&self.collection_name)
            .query(Query::new_nearest(query.vector().unwrap().to_vec()))
            .limit(query.limit().into())
            .with_payload(true);

        query_builder = query_builder.filter(Qdrant::create_filter(&query));

        self.client
            .query(query_builder)
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
