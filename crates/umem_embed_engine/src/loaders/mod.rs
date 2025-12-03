use crate::{HashMap, engine::AddSource};
use anyhow::Result;

pub mod pdf_loader;

pub struct LoadDataResult {
    pub(crate) doc_id: String,
    pub(crate) data: Vec<(String, HashMap<String, String>)>,
}

#[async_trait::async_trait]
pub trait Loader {
    async fn load_data(&self, source: &AddSource) -> Result<LoadDataResult>;
}
