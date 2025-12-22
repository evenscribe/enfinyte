use std::iter::zip;

use crate::{VectorStoreBase, VectorStoreError};
use async_trait::async_trait;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, query, Pool, Postgres, QueryBuilder, Row};
use thiserror::Error;
use umem_core::LifecycleState;
use umem_core::Memory;
use umem_core::Query;
use uuid::Uuid;

pub struct PgVector {
    client: Pool<Postgres>,
    collection_name: String,
    embedding_model_dimensions: u16,
}

#[derive(Error, Debug)]
pub enum PgError {
    #[error("Pg client error: {0}")]
    ClientError(#[from] sqlx::Error),

    #[error("Vector must be supplied for search.")]
    VectorNotSupplied,
}

impl From<sqlx::Error> for VectorStoreError {
    fn from(value: sqlx::Error) -> Self {
        Into::<PgError>::into(value).into()
    }
}

type Result<T> = std::result::Result<T, PgError>;

impl PgVector {
    pub async fn new(pgvector: umem_config::PgVector) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&pgvector.url)
            .await?;

        Ok(Self {
            client: pool,
            embedding_model_dimensions: pgvector.embedding_model_dimensions,
            collection_name: pgvector.collection_name,
        })
    }

    fn filter_include_archived(builder: &mut QueryBuilder<'_, Postgres>, query: &umem_core::Query) {
        if !query.include_archived() {
            builder.push(format!(
                "AND payload->>'lifecycle'='{}' ",
                LifecycleState::Active.as_str()
            ));
        }
    }

    fn filter_context(builder: &mut QueryBuilder<'_, Postgres>, query: &umem_core::Query) {
        if let Some(user_id) = query.context().user_id() {
            builder.push(format!("AND payload->'context'->>'user_id'='{}' ", user_id));
        }
        if let Some(agent_id) = query.context().agent_id() {
            builder.push(format!(
                "AND payload->'context'->>'agent_id'='{}' ",
                agent_id
            ));
        }
        if let Some(run_id) = query.context().run_id() {
            builder.push(format!("AND payload->'context'->>'run_id'='{}' ", run_id));
        }
    }

    fn filter_kinds(builder: &mut QueryBuilder<'_, Postgres>, query: &umem_core::Query) {
        if let Some(kinds) = query.kinds() {
            builder.push(" AND payload->>'kind'=ANY('");
            builder.push_bind(
                kinds
                    .iter()
                    .map(|kind| kind.as_str())
                    .collect::<Vec<&str>>(),
            );
            builder.push(") ");
        }
    }

    fn filter_tags(builder: &mut QueryBuilder<'_, Postgres>, query: &umem_core::Query) {
        if let Some(tags) = query.tags() {
            builder.push(" AND payload->'content'->'tags' ?| ");
            builder.push_bind(
                tags.iter()
                    .map(|tag| tag.to_owned())
                    .collect::<Vec<String>>(),
            );
        }
    }

    fn filter_temporal(builder: &mut QueryBuilder<'_, Postgres>, query: &umem_core::Query) {
        if let Some(temporal) = query.temporal() {
            if temporal.has_created_range() {
                if let Some(created) = temporal.created_range().0 {
                    builder.push(" AND payload->'temporal'->>'created_at' > ");
                    builder.push_bind(created);
                }
                if let Some(created) = temporal.created_range().1 {
                    builder.push(" AND payload->'temporal'->>'created_at' < ");
                    builder.push_bind(created);
                }
            }
            if temporal.has_updated_range() {
                if let Some(updated) = temporal.updated_range().0 {
                    builder.push(" AND payload->'temporal'->>'updated_at' > ");
                    builder.push_bind(updated);
                }
                if let Some(updated) = temporal.updated_range().1 {
                    builder.push(" AND payload->'temporal'->>'updated_at' < ");
                    builder.push_bind(updated);
                }
            }
        }
    }

    fn filter_signals(builder: &mut QueryBuilder<'_, Postgres>, query: &umem_core::Query) {
        if let Some(signal) = query.signals() {
            if let Some(salience) = signal.min_salience() {
                builder.push(" AND payload->'signals'->>'salience' > ");
                builder.push_bind(salience);
            }
            if let Some(certainty) = signal.min_certainty() {
                builder.push(" AND payload->'signals'->>'certainty' > ");
                builder.push_bind(certainty);
            }
        }
    }

    fn create_filter(builder: &mut QueryBuilder<'_, Postgres>, query: &Query) {
        Self::filter_include_archived(builder, query);
        Self::filter_context(builder, query);
        Self::filter_kinds(builder, query);
        Self::filter_tags(builder, query);
        Self::filter_temporal(builder, query);
        Self::filter_signals(builder, query);

        if query.vector().is_some() {
            builder.push(" ORDER by distance ");
        }
        builder.push(format!(" LIMIT {} ", query.limit()));
    }
}

#[async_trait]
impl VectorStoreBase for PgVector {
    async fn create_collection(&self) -> crate::Result<()> {
        query(r#"CREATE EXTENSION IF NOT EXISTS vector"#)
            .execute(&self.client)
            .await?;

        query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id UUID PRIMARY KEY,
                vector vector({}),
                payload JSONB
            )
            "#,
            self.collection_name, self.embedding_model_dimensions
        ))
        .execute(&self.client)
        .await?;

        query(&format!(
            r#"
            CREATE INDEX IF NOT EXISTS {}_hnsw_idx
                ON {}
                USING hnsw (vector vector_cosine_ops)
            "#,
            &self.collection_name, &self.collection_name
        ))
        .execute(&self.client)
        .await?;

        Ok(())
    }

    async fn delete_collection(&self) -> crate::Result<()> {
        query(&format!(r#"DROP TABLE IF EXISTS {}"#, self.collection_name))
            .execute(&self.client)
            .await?;

        Ok(())
    }

    async fn reset(&self) -> crate::Result<()> {
        self.delete_collection().await?;
        self.create_collection().await
    }

    async fn insert<'a>(
        &self,
        vectors: Vec<Vec<f32>>,
        payloads: Vec<&'a Memory>,
    ) -> crate::Result<()> {
        for (vector, payload) in zip(vectors, payloads) {
            query(&format!(
                r#"INSERT INTO {}
                    (id, vector, payload)
                    VALUES
                    ($1, $2, $3)"#,
                self.collection_name
            ))
            .bind(payload.get_id())
            .bind(vector)
            .bind(json!(payload))
            .execute(&self.client)
            .await?;
        }

        Ok(())
    }

    async fn get(&self, vector_id: &str) -> crate::Result<Memory> {
        let result = query(&format!(
            r#"SELECT payload FROM {} WHERE id = $1"#,
            self.collection_name,
        ))
        .bind(Uuid::parse_str(vector_id)?)
        .fetch_one(&self.client)
        .await?;

        let payload: serde_json::Value = result.try_get(0)?;
        let memory: Memory = serde_json::from_value(payload)?;

        Ok(memory)
    }

    async fn update(
        &self,
        vector_id: &str,
        vector: Option<Vec<f32>>,
        payload: Option<Memory>,
    ) -> crate::Result<()> {
        if let Some(vector) = vector {
            query(&format!(
                r#"UPDATE {} SET vector = $1 WHERE id = $2"#,
                self.collection_name,
            ))
            .bind(vector)
            .bind(Uuid::parse_str(vector_id)?)
            .execute(&self.client)
            .await?;
        }

        if let Some(payload) = payload {
            query(&format!(
                r#"UPDATE {} SET payload = $1 WHERE id = $2"#,
                self.collection_name,
            ))
            .bind(json!(payload))
            .bind(Uuid::parse_str(vector_id)?)
            .execute(&self.client)
            .await?;
        }

        Ok(())
    }

    async fn delete(&self, vector_id: &str) -> crate::Result<()> {
        query(&format!(
            r#"DELETE FROM {} WHERE id = $1"#,
            self.collection_name,
        ))
        .bind(Uuid::parse_str(vector_id)?)
        .execute(&self.client)
        .await?;

        Ok(())
    }

    async fn list(&self, query: umem_core::Query) -> crate::Result<Vec<Memory>> {
        let mut query_builder = QueryBuilder::<Postgres>::new(format!(
            " SELECT payload FROM {} WHERE 1=1 ",
            self.collection_name
        ));

        PgVector::create_filter(&mut query_builder, &query);

        let q = query_builder.build();

        q.fetch_all(&self.client)
            .await?
            .into_iter()
            .map(|row| {
                let payload: serde_json::Value = row.try_get(0)?;
                let memory: Memory = serde_json::from_value(payload)?;
                Ok(memory)
            })
            .collect()
    }

    async fn search(&self, query: umem_core::Query) -> crate::Result<Vec<Memory>> {
        if query.vector().is_none() {
            return Err(PgError::VectorNotSupplied)?;
        }

        let mut query_builder = QueryBuilder::<Postgres>::new(format!(
            " SELECT payload, vector<=>'{:?}'::vector AS distance FROM {} WHERE 1=1 ",
            query.vector().unwrap(),
            self.collection_name
        ));

        PgVector::create_filter(&mut query_builder, &query);

        query_builder
            .build()
            .fetch_all(&self.client)
            .await?
            .into_iter()
            .map(|row| {
                let payload: serde_json::Value = row.try_get(0)?;
                let memory: Memory = serde_json::from_value(payload)?;
                Ok(memory)
            })
            .collect()
    }
}
