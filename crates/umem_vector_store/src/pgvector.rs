use std::iter::zip;

use crate::VectorStoreBase;
use anyhow::Result;
use async_trait::async_trait;
use rustc_hash::FxHashMap;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, query, Pool, Postgres, QueryBuilder, Row};
use umem_proto::generated;
use uuid::Uuid;

pub struct PgVector {
    client: Pool<Postgres>,
    collection_name: String,
    embedding_model_dimensions: u16,
}

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
}

#[async_trait]
impl VectorStoreBase for PgVector {
    async fn create_collection(&self) -> Result<()> {
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

    async fn delete_collection(&self) -> Result<()> {
        query(&format!(r#"DROP TABLE IF EXISTS {}"#, self.collection_name))
            .execute(&self.client)
            .await?;

        Ok(())
    }

    async fn reset(&self) -> Result<()> {
        self.delete_collection().await?;
        self.create_collection().await
    }

    async fn insert(&self, vectors: Vec<Vec<f32>>, payloads: Vec<generated::Memory>) -> Result<()> {
        for (vector, payload) in zip(vectors, payloads) {
            query(&format!(
                r#"INSERT INTO {}
                    (id, vector, payload)
                    VALUES
                    ($1, $2, $3)"#,
                self.collection_name
            ))
            .bind(Uuid::parse_str(&payload.id)?)
            .bind(vector)
            .bind(json!(payload))
            .execute(&self.client)
            .await?;
        }

        Ok(())
    }

    async fn get(&self, vector_id: &str) -> Result<generated::Memory> {
        let result = query(&format!(
            r#"SELECT payload FROM {} WHERE id = $1"#,
            self.collection_name,
        ))
        .bind(Uuid::parse_str(vector_id)?)
        .fetch_one(&self.client)
        .await?;

        let payload: serde_json::Value = result.try_get(0)?;
        let memory: generated::Memory = serde_json::from_value(payload)?;

        Ok(memory)
    }

    async fn update(
        &self,
        vector_id: &str,
        vector: Option<Vec<f32>>,
        payload: Option<FxHashMap<String, serde_json::Value>>,
    ) -> Result<()> {
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

    async fn delete(&self, vector_id: &str) -> Result<()> {
        query(&format!(
            r#"DELETE FROM {} WHERE id = $1"#,
            self.collection_name,
        ))
        .bind(Uuid::parse_str(vector_id)?)
        .execute(&self.client)
        .await?;

        Ok(())
    }

    async fn list(
        &self,
        filters: Option<FxHashMap<&str, String>>,
        limit: u32,
    ) -> Result<Vec<generated::Memory>> {
        let mut query = QueryBuilder::<Postgres>::new(format!(
            " SELECT payload FROM {} ",
            self.collection_name
        ));

        if let Some(filters) = filters {
            query.push(" WHERE ");

            let mut filters = filters.into_iter().peekable();
            while let Some(current) = filters.next() {
                query.push(format!(" payload->>'{}'='{}'", current.0, current.1));
                if filters.peek().is_some() {
                    query.push(" AND ");
                }
            }
        }
        query.push(format!(" LIMIT {} ", limit));

        let q = query.build();

        q.fetch_all(&self.client)
            .await?
            .into_iter()
            .map(|row| {
                let payload: serde_json::Value = row.try_get(0)?;
                let memory: generated::Memory = serde_json::from_value(payload)?;
                Ok(memory)
            })
            .collect()
    }

    async fn search(
        &self,
        query_vector: Vec<f32>,
        filters: Option<FxHashMap<&str, String>>,
        limit: u64,
    ) -> Result<Vec<generated::Memory>> {
        let mut query = QueryBuilder::<Postgres>::new(format!(
            " SELECT payload, vector<=>'{:?}'::vector AS distance FROM {} ",
            query_vector, self.collection_name
        ));

        if let Some(filters) = filters {
            query.push("WHERE");

            let mut filters = filters.into_iter().peekable();
            while let Some(current) = filters.next() {
                query.push(format!("  payload->>'{}'='{}' ", current.0, current.1));
                if filters.peek().is_some() {
                    query.push(" AND ");
                }
            }
        }
        query.push("ORDER by distance");
        query.push(format!(" LIMIT {} ", limit));

        query
            .build()
            .fetch_all(&self.client)
            .await?
            .into_iter()
            .map(|row| {
                let payload: serde_json::Value = row.try_get(0)?;
                let memory: generated::Memory = serde_json::from_value(payload)?;
                Ok(memory)
            })
            .collect()
    }
}
