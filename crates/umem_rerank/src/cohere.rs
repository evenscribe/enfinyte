use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use umem_core::{Memory, MemoryContent, MemoryKind};

use super::{
    DataRow, RerankError, RerankOptions, RerankOptionsError, RerankResponse, RerankerBase,
};

#[derive(Debug, Deserialize)]
pub struct CohereResponse {
    pub results: Vec<CohereDataRow>,
}

#[derive(Debug, Deserialize)]
pub struct CohereDataRow {
    pub index: usize,
    pub relevance_score: f32,
}

impl From<CohereDataRow> for DataRow {
    fn from(value: CohereDataRow) -> Self {
        Self {
            index: value.index,
            score: value.relevance_score,
        }
    }
}

#[derive(Error, Debug)]
pub enum CohereError {
    #[error("pinecone api call failed with: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("rerank option action failed with: {0}")]
    RerankOptionsError(#[from] RerankOptionsError),
}

pub struct Cohere {
    api_key: String,
    model: String,
}

impl Cohere {
    pub fn new(config: umem_config::Cohere) -> Self {
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
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct CohereMemory {
    kind: MemoryKind,
    content: MemoryContent,
}

impl From<&Memory> for CohereMemory {
    fn from(value: &Memory) -> Self {
        Self {
            kind: *value.kind(),
            content: value.content().clone(),
        }
    }
}

impl Cohere {
    async fn rank_impl(
        &self,
        query: String,
        documents: &[Memory],
        options: Option<RerankOptions>,
    ) -> Result<CohereResponse, CohereError> {
        let client = Client::new();

        self.validate(options, documents.len())?;

        let documents: Vec<CohereMemory> = documents.iter().map(|m| m.into()).collect();
        let documents: Vec<String> = documents
            .iter()
            .map(|m| serde_saphyr::to_string(m).unwrap())
            .collect();

        let response = client
            .post("https://api.cohere.com/v2/rerank")
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", &format!("bearer {}", &self.api_key))
            .json(&json!({
                "model": &self.model,
                "query": query,
                "documents": documents,
                "top_n": 2,
            }));

        let response = response.send().await?;

        match response.error_for_status() {
            Ok(res) => {
                let response: CohereResponse = res.json().await?;
                Ok(response)
            }
            Err(err) => Err(err.into()),
        }
    }
}
impl From<CohereResponse> for RerankResponse {
    fn from(value: CohereResponse) -> Self {
        let data = value.results.into_iter().map(|d| d.into()).collect();
        Self { data }
    }
}

#[async_trait]
impl RerankerBase for Cohere {
    async fn rank(
        &self,
        query: String,
        documents: &[Memory],
        options: Option<RerankOptions>,
    ) -> Result<RerankResponse, RerankError> {
        Ok(self.rank_impl(query, documents, options).await?.into())
    }
}
