use std::collections::HashSet;

use crate::{
    HashMap,
    chunkers::{Chunker, pdf_chunker::PdfChunker},
    config::{AddConfig, ChunkerConfig},
    data_type::DataType,
    engine::AddSource,
    loaders::{LoadDataResult, Loader, pdf_loader::PdfLoader},
};
use anyhow::Result;

pub(crate) struct DataFormatter {
    pub(crate) loader: Box<dyn Loader>,
    pub(crate) chunker: Box<dyn Chunker>,
}

pub(crate) struct CreateChunksResponse {
    pub doc_id: String,
    pub chunk_ids: HashSet<String>,
    pub documents: Vec<String>,
    pub metadatas: Vec<HashMap<String, String>>,
}

impl DataFormatter {
    pub fn try_new(
        data_type: DataType,
        loader: Option<Box<dyn Loader>>,
        chunker: Option<Box<dyn Chunker>>,
        add_config: &AddConfig,
    ) -> Result<Self> {
        Ok(DataFormatter {
            loader: get_loader(&data_type, loader)?,
            chunker: get_chunker(&data_type, chunker, add_config)?,
        })
    }

    pub(crate) async fn create_chunks(
        &self,
        source: &AddSource,
        app_id: Option<String>,
        chunker_config: Option<ChunkerConfig>,
    ) -> Result<CreateChunksResponse> {
        let mut chunk_ids: HashSet<String> = HashSet::default();
        let mut documents = vec![];
        let mut metadatas = vec![];

        let min_chunk_size = match chunker_config {
            Some(config) => config.min_chunk_size,
            None => 0,
        };

        let LoadDataResult {
            mut doc_id,
            data: data_records,
        } = self.load_data(source).await?;

        doc_id = match app_id {
            Some(ref app_id) => format!("{}--{}", app_id.clone(), doc_id),
            None => doc_id,
        };

        for (content, mut metadata) in data_records {
            metadata.insert(
                "data_type".to_string(),
                self.chunker.get_data_type().to_string(),
            );
            metadata.insert("doc_id".to_string(), doc_id.clone());

            let url = match metadata.get("url") {
                Some(u) => u.clone(),
                None => source.to_string(),
            };
            let mut chunks = self.chunker.get_chunks(content);

            for chunk in chunks.iter_mut() {
                chunk.push_str(&url);
                let chunk_id = blake3::hash(chunk.as_bytes()).to_hex().to_string();

                let app_id_clone = app_id.clone();
                let chunk_id = match app_id_clone {
                    Some(aid) => format!("{}--{}", aid, chunk_id),
                    None => chunk_id,
                };

                if !chunk_ids.contains(&chunk_id) && chunk.len() >= min_chunk_size {
                    chunk_ids.insert(chunk_id.clone());
                    documents.push(std::mem::take(chunk));
                    metadatas.push(std::mem::take(&mut metadata));
                }
            }
        }

        Ok(CreateChunksResponse {
            doc_id,
            chunk_ids,
            documents,
            metadatas,
        })
    }

    async fn load_data(&self, source: &AddSource) -> Result<LoadDataResult> {
        self.loader.load_data(source).await
    }
}

fn get_loader(data_type: &DataType, loader: Option<Box<dyn Loader>>) -> Result<Box<dyn Loader>> {
    match data_type {
        DataType::Custom => match loader {
            Some(l) => Ok(l),
            None => Err(anyhow::anyhow!("Custom data type requires a loader")),
        },
        DataType::PdfFile => Ok(Box::new(PdfLoader::new())),
        _ => unimplemented!(),
    }
}

fn get_chunker(
    data_type: &DataType,
    chunker: Option<Box<dyn Chunker>>,
    add_config: &AddConfig,
) -> Result<Box<dyn Chunker>> {
    if let Some(c) = chunker {
        return Ok(c);
    }

    match data_type {
        DataType::PdfFile => Ok(Box::new(PdfChunker::try_new(add_config)?)),
        _ => unimplemented!(),
    }
}
