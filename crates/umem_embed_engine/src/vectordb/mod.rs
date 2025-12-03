use anyhow::Result;

pub trait VectorDB {
    fn get(
        &self,
        ids: Option<&[String]>,
        filter: Option<&[(String, String)]>,
        limit: Option<usize>,
    ) -> Result<VectorDBGetResponse>;
}

pub struct VectorDBGetResponse {
    pub ids: Vec<String>,
    pub metadatas: Vec<Metadata>,
}

pub struct Metadata {
    pub doc_id: String,
}
