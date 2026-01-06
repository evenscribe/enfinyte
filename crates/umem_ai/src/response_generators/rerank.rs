use crate::ResponseGeneratorError;

pub async fn rerank(request: RerankRequest) -> Result<RerankResponse, ResponseGeneratorError> {
    unimplemented!()
}

pub struct RerankRequest {
    pub query: String,
    pub documents: Vec<String>,
    pub top_n: usize,
}

impl RerankRequest {
    pub fn builder() -> RerankRequestBuilder {
        RerankRequestBuilder {
            top_n: 5,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct RerankRequestBuilder {
    query: Option<String>,
    documents: Vec<String>,
    top_n: usize,
}

#[derive(thiserror::Error, Debug)]
pub enum RerankRequestBuilderError {
    #[error("missing query from rerank request")]
    MissingQuery,
    #[error("at least one document is required in rerank request")]
    EmptyDocuments,
}

impl RerankRequestBuilder {
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn documents<I>(mut self, documents: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.documents.extend(documents);
        self
    }

    pub fn document(mut self, document: impl Into<String>) -> Self {
        self.documents.push(document.into());
        self
    }

    pub fn top_k(mut self, top_n: usize) -> Self {
        self.top_n = top_n;
        self
    }

    pub fn build(self) -> Result<RerankRequest, RerankRequestBuilderError> {
        if self.documents.is_empty() {
            return Err(RerankRequestBuilderError::EmptyDocuments);
        }

        Ok(RerankRequest {
            query: self.query.ok_or(RerankRequestBuilderError::MissingQuery)?,
            documents: self.documents,
            top_n: self.top_n,
        })
    }
}

pub struct RerankResponse {
    pub rankings: Vec<Ranking>,
    pub ranked_documents: Vec<String>,
}

pub struct Ranking {
    pub original_index: usize,
    pub score: f32,
    pub document: String,
}
