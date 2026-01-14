use crate::{
    Ranking, RerankRequest, RerankResponse, Reranks, ReranksStructuredData, ResponseGeneratorError,
    SerializationFormat, SerializationMode, StructuredRanking, StructuredRerankRequest,
    StructuredRerankResponse, reqwest_client, utils,
};
use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CohereRerankAPIV2Response {
    pub results: Vec<CohereRerankResult>,
    #[serde(flatten)]
    pub raw_fields: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CohereRerankResult {
    pub index: usize,
    pub relevance_score: f32,
}

#[derive(TypedBuilder, Debug, Clone)]
pub struct CohereProvider {
    #[builder(default = "https://api.cohere.com/v2".into(), setter(transform = |value: impl Into<String>| value.into()))]
    base_url: String,

    #[builder(setter(transform = |value: impl Into<String>| value.into()))]
    api_key: String,

    #[builder(default = HeaderMap::default(), setter(transform = |value: Vec<(String, String)>|
    utils::build_header_map(value.as_slice()).unwrap_or_default()
    ))]
    headers: HeaderMap,
}

#[async_trait]
impl Reranks for CohereProvider {
    async fn rerank(
        &self,
        request: RerankRequest,
    ) -> Result<RerankResponse, ResponseGeneratorError> {
        let response = reqwest_client
            .post(format!("{}/rerank", self.base_url))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", &format!("bearer {}", &self.api_key))
            .json(&json!({
                "model": &request.model.model_name,
                "query": &request.query,
                "documents": &request.documents,
                "top_n": request.top_n,
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<CohereRerankAPIV2Response>()
            .await?;

        let (rankings, ranked_documents) = response.results.iter().try_fold(
            (
                Vec::with_capacity(response.results.len()),
                Vec::with_capacity(response.results.len()),
            ),
            |(mut rankings, mut docs), result| {
                let document = request.documents.get(result.index).ok_or(
                    ResponseGeneratorError::InvalidProviderResponse(
                        "Cohere returned an invalid index".to_string(),
                    ),
                )?;
                docs.push(document.clone());
                rankings.push(Ranking {
                    original_index: result.index,
                    score: result.relevance_score,
                    document: document.clone(),
                });
                Ok::<_, ResponseGeneratorError>((rankings, docs))
            },
        )?;

        Ok(RerankResponse {
            raw_fields: response.raw_fields,
            rankings,
            ranked_documents,
        })
    }
}

#[async_trait]
impl ReranksStructuredData for CohereProvider {
    async fn rerank_structured<T>(
        &self,
        request: StructuredRerankRequest<T>,
    ) -> Result<StructuredRerankResponse<T>, ResponseGeneratorError>
    where
        T: Serialize + Clone + Send + Sync,
    {
        let serialized_documents: Vec<String> = request
            .documents
            .iter()
            .map(|doc| {
                match (request.serialization_mode, request.serialization_format) {
                    (SerializationMode::Json, SerializationFormat::Compact) => {
                        serde_json::to_string(doc).map_err(|e| e.to_string())
                    }
                    (SerializationMode::Json, SerializationFormat::Pretty) => {
                        serde_json::to_string_pretty(doc).map_err(|e| e.to_string())
                    }
                    (SerializationMode::Yaml, _) => {
                        serde_saphyr::to_string(doc).map_err(|e| e.to_string())
                    }
                }
                .map_err(ResponseGeneratorError::StructuredRerankDocumentsSerializationError)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let response = reqwest_client
            .post(format!("{}/rerank", self.base_url))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", &format!("bearer {}", &self.api_key))
            .headers(self.headers.clone())
            .json(&json!({
                "model": &request.model.model_name,
                "query": &request.query,
                "documents": &serialized_documents,
                "top_n": request.top_n,
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<CohereRerankAPIV2Response>()
            .await?;

        let (rankings, ranked_documents) = response.results.iter().try_fold(
            (
                Vec::with_capacity(response.results.len()),
                Vec::with_capacity(response.results.len()),
            ),
            |(mut rankings, mut docs), result| {
                let document = request.documents.get(result.index).ok_or(
                    ResponseGeneratorError::InvalidProviderResponse(
                        "Cohere returned an invalid index".to_string(),
                    ),
                )?;
                docs.push(document.clone());
                rankings.push(StructuredRanking {
                    original_index: result.index,
                    score: result.relevance_score,
                    document: document.clone(),
                });
                Ok::<_, ResponseGeneratorError>((rankings, docs))
            },
        )?;

        Ok(StructuredRerankResponse {
            rankings,
            ranked_documents,
            raw_fields: response.raw_fields,
        })
    }
}
