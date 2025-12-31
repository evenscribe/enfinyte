use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use typed_builder::TypedBuilder;
use umem_core::Memory;

use crate::{DataRow, RerankOptions, RerankResponse};

use super::{RerankError, RerankerBase};

#[derive(TypedBuilder, Debug, Default, Deserialize)]
pub struct PineconeOptions {
    #[builder(default = None)]
    pub top_n: Option<usize>,
}
impl PineconeOptions {
    pub fn validate(&self, doc_len: usize) -> Result<(), PineconeOptionsError> {
        if let Some(top_n) = self.top_n {
            if doc_len < top_n {
                return Err(PineconeOptionsError::InvalidTopN(top_n, doc_len));
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum PineconeOptionsError {
    #[error("top_n : {0} cannot be more than doc_len: {1}")]
    InvalidTopN(usize, usize),
}

#[derive(Debug, Deserialize)]
pub struct PineconeResponse {
    pub model: String,
    pub data: Vec<PineconeDataRow>,
    pub usage: Usage,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub rerank_units: usize,
}

#[derive(Debug, Deserialize)]
pub struct PineconeDataRow {
    pub index: usize,
    pub score: f32,
}

impl From<PineconeDataRow> for DataRow {
    fn from(value: PineconeDataRow) -> Self {
        Self {
            index: value.index,
            score: value.score,
        }
    }
}

#[derive(Error, Debug)]
pub enum PineconeError {
    #[error("pinecone api call failed with: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("invalid options were passed: {0}")]
    PineconeOptionsError(#[from] PineconeOptionsError),
}

pub struct Pinecone {
    api_key: String,
    model: String,
}

impl Pinecone {
    pub fn new(config: umem_config::Pinecone) -> Self {
        Self {
            api_key: config.api_key,
            model: config.model,
        }
    }
}

impl Pinecone {
    async fn rank_impl(
        &self,
        query: String,
        documents: &[Memory],
        options: Option<RerankOptions>,
    ) -> Result<PineconeResponse, PineconeError> {
        let client = Client::new();

        let options = options.unwrap_or_default();
        let pinecone_options = options.pinecone.unwrap_or_default();
        pinecone_options.validate(documents.len())?;

        let documents: Vec<serde_json::Value> = documents
            .iter()
            .map(|m| json!({"text": m.content().summary()}))
            .collect();

        let response = client
            .post("https://api.pinecone.io/rerank")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("X-Pinecone-Api-Version", "2025-10")
            .header("Api-Key", &self.api_key)
            .json(&json!({
                "model": &self.model,
                "query": query,
                "documents": documents,
                "top_n": 2,
                "return_documents": false,
                "parameters": {
                    "truncate": "END"
                }
            }));

        let response = response.send().await?;

        match response.error_for_status() {
            Ok(res) => {
                let response: PineconeResponse = res.json().await?;
                Ok(response)
            }
            Err(err) => Err(err.into()),
        }
    }
}
impl From<PineconeResponse> for RerankResponse {
    fn from(value: PineconeResponse) -> Self {
        let data = value.data.into_iter().map(|d| d.into()).collect();
        Self { data }
    }
}

#[async_trait]
impl RerankerBase for Pinecone {
    async fn rank(
        &self,
        query: String,
        documents: &[Memory],
        options: Option<RerankOptions>,
    ) -> Result<RerankResponse, RerankError> {
        Ok(self.rank_impl(query, documents, options).await?.into())
    }
}
