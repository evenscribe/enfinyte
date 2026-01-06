use crate::ResponseGeneratorError;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

pub async fn structured_rerank<T>(
    request: StructuredRerankRequest<T>,
) -> Result<StructuredRerankResponse<T>, ResponseGeneratorError>
where
    T: Serialize + DeserializeOwned,
{
    unimplemented!()
}

#[derive(Clone)]
pub struct StructuredRerankRequest<T>
where
    T: Serialize + DeserializeOwned,
{
    pub query: String,
    pub documents: Vec<T>,
    pub top_n: usize,
    pub output_type: PhantomData<T>,
}

impl<T> StructuredRerankRequest<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn builder() -> StructuredRerankRequestBuilder<T> {
        StructuredRerankRequestBuilder {
            top_n: 5,
            output_type: PhantomData,
            query: None,
            documents: vec![],
        }
    }
}

pub struct StructuredRerankRequestBuilder<T>
where
    T: Serialize + DeserializeOwned,
{
    query: Option<String>,
    documents: Vec<T>,
    top_n: usize,
    pub output_type: PhantomData<T>,
}

#[derive(thiserror::Error, Debug)]
pub enum RerankRequestBuilderError {
    #[error("missing query from rerank request")]
    MissingQuery,
    #[error("at least one document is required in rerank request")]
    EmptyDocuments,
}

impl<T> StructuredRerankRequestBuilder<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn documents<I>(mut self, documents: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        self.documents.extend(documents);
        self
    }

    pub fn document(mut self, document: impl Into<T>) -> Self {
        self.documents.push(document.into());
        self
    }

    pub fn top_k(mut self, top_n: usize) -> Self {
        self.top_n = top_n;
        self
    }

    pub fn build(self) -> Result<StructuredRerankRequest<T>, RerankRequestBuilderError> {
        if self.documents.is_empty() {
            return Err(RerankRequestBuilderError::EmptyDocuments);
        }

        Ok(StructuredRerankRequest {
            query: self.query.ok_or(RerankRequestBuilderError::MissingQuery)?,
            documents: self.documents,
            top_n: self.top_n,
            output_type: PhantomData,
        })
    }
}

pub struct StructuredRerankResponse<T>
where
    T: Serialize + DeserializeOwned,
{
    pub rankings: Vec<StructuredRanking<T>>,
    pub ranked_documents: Vec<T>,
}

pub struct StructuredRanking<T>
where
    T: Serialize + DeserializeOwned,
{
    original_index: usize,
    score: f32,
    document: T,
}
