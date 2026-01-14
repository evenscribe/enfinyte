use super::{
    DataRow, RerankError, RerankOptions, RerankOptionsError, RerankResponse, RerankerBase,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use umem_core::Memory;

#[derive(Debug, Deserialize)]
pub struct PineconeResponse {
    pub data: Vec<PineconeDataRow>,
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

    #[error("rerank option action failed with: {0}")]
    RerankOptionsError(#[from] RerankOptionsError),
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

    fn validate(
        &self,
        options: Option<RerankOptions>,
        doc_len: usize,
    ) -> Result<(), RerankOptionsError> {
        if let Some(options) = options {
            if let Some(top_n) = options.top_n {
                if doc_len < top_n {
                    return Err(RerankOptionsError::InvalidTopN(top_n, doc_len));
                }
            }
            if options.structured_input.is_some() {
                return Err(RerankOptionsError::InvalidField(
                    "structured_input".to_string(),
                    "Pinecone".to_string(),
                ));
            }
        }

        Ok(())
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

        self.validate(options, documents.len())?;

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
