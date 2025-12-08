use crate::{
    HashMap,
    chunkers::Chunker,
    config::{AddConfig, ChunkerConfig, EngineConfig},
    data_formatter::{CreateChunksResponse, DataFormatter},
    data_type::{DIRECT_DATA_TYPES, DataType, INDIRECT_DATA_TYPES},
    loaders::Loader,
    vectordb::{VectorDB, VectorDBGetResponse},
};
use anyhow::Result;

pub struct Engine<LLM, VectorDB, Embedder> {
    config: EngineConfig,
    chunker: Option<ChunkerConfig>,
    llm: LLM,
    db: VectorDB,
    embedder: Embedder,
    system_prompt: Option<String>,
    user_asks: Vec<UserAsk>,
}

pub struct AddRequest {
    source: AddSource,
    data_type: Option<DataType>,
    metadata: HashMap<String, String>,
    add_config: Option<AddConfig>,
    chunker: Option<Box<dyn Chunker>>,
    loader: Option<Box<dyn Loader>>,
    dry_run: bool,
}

pub struct UserAsk {
    source: AddSource,
    data_type: DataType,
    metadata: HashMap<String, String>,
}

pub enum AddSource {
    LocalFile(String),
    Url(String),
}

impl AddSource {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            AddSource::LocalFile(path) => path.as_bytes(),
            AddSource::Url(url) => url.as_bytes(),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            AddSource::LocalFile(path) => path.clone(),
            AddSource::Url(url) => url.clone(),
        }
    }
}

pub struct LoadAndEmbedResponse {
    pub documents: Vec<String>,
    pub metadatas: Vec<HashMap<String, String>>,
    pub ids: Vec<String>,
    pub added_chunks_count: usize,
}

impl<LLM, DB, Embedder> Engine<LLM, DB, Embedder>
where
    DB: VectorDB,
{
    pub fn new(
        config: EngineConfig,
        llm: LLM,
        db: DB,
        embedder: Embedder,
        system_prompt: Option<String>,
        chunker: Option<ChunkerConfig>,
    ) -> Self {
        Engine {
            config,
            llm,
            db,
            embedder,
            system_prompt,
            chunker,
            user_asks: vec![],
        }
    }

    pub async fn add(&mut self, request: AddRequest) -> Result<String> {
        let add_config = match request.add_config {
            Some(config) => config,
            None => AddConfig {
                chunker: self.chunker.clone(),
                loader: None,
            },
        };

        let data_type = match request.data_type {
            Some(dt) => dt,
            None => DataType::try_from_source(&request.source).await?,
        };
        let source_hash = blake3::hash(request.source.as_bytes()).to_hex().to_string();
        let data_formatter =
            DataFormatter::try_new(data_type, request.loader, request.chunker, &add_config)?;

        let LoadAndEmbedResponse {
            documents,
            metadatas,
            ids,
            added_chunks_count,
        } = self
            .load_and_embed(
                &data_formatter,
                &request.source,
                source_hash.clone(),
                Some(request.metadata.clone()),
                add_config,
                request.dry_run,
            )
            .await?;
        let _: () = self.user_asks.push(UserAsk {
            source: request.source,
            data_type,
            metadata: request.metadata,
        });

        // TODO: make a data sources table and insert source_hash, app_id, data_type, source, metadata

        if request.dry_run {
            return Ok(format!(
                "Dry run: {} chunks would be added.",
                added_chunks_count
            ));
        }

        if self.config.enable_telemetry {
            // TODO: log it to some telemetry system
        }

        Ok(source_hash)
    }

    async fn load_and_embed(
        &self,
        data_formatter: &DataFormatter,
        source: &AddSource,
        source_hash: String,
        metadata: Option<HashMap<String, String>>,
        add_config: AddConfig,
        dry_run: bool,
    ) -> Result<LoadAndEmbedResponse> {
        let existing_doc_id = self
            .get_existing_document_id(&data_formatter.chunker, source)
            .await?;
        let app_id = self.config.id.clone();

        let CreateChunksResponse {
            mut documents,
            mut metadatas,
            chunk_ids,
            doc_id,
        } = data_formatter
            .create_chunks(source, app_id, add_config.chunker)
            .await?;

        if existing_doc_id
            .as_ref()
            .is_some_and(|id| id.as_str() == doc_id)
        {
            return Ok(LoadAndEmbedResponse {
                documents: vec![],
                metadatas: vec![],
                ids: vec![],
                added_chunks_count: 0,
            });
        } else if existing_doc_id
            .as_ref()
            .is_some_and(|id| id.as_str() != doc_id)
        {
            let _: () = self.db.delete(&[existing_doc_id.unwrap()]).await?;
        }

        let mut filters: Vec<(String, String)> = vec![];
        let data_type = data_formatter.get_data_type();

        if data_type == DataType::Json {
            let url = blake3::hash(source.to_string().as_bytes())
                .to_hex()
                .to_string();
            filters.push(("url".to_string(), url));
        }

        if let Some(app_id) = &self.config.id {
            filters.push(("app_id".to_string(), app_id.clone()));
        }

        let mut chunk_ids_as_vec: Vec<String> = chunk_ids.iter().map(|id| id.clone()).collect();

        let VectorDBGetResponse { ids, .. } = self
            .db
            .get(Some(&chunk_ids_as_vec), Some(filters.as_slice()), None)
            .await?;

        if !ids.is_empty() {
            let filtered_ids: Vec<(String, String, HashMap<String, String>)> = ids
                .into_iter()
                .zip(documents)
                .zip(metadatas)
                .map(|((id, doc), meta)| (id, doc, meta))
                .filter(|(id, _, _)| !chunk_ids.contains(id))
                .collect();

            if filtered_ids.is_empty() {
                return Ok(LoadAndEmbedResponse {
                    documents: vec![],
                    metadatas: vec![],
                    ids: vec![],
                    added_chunks_count: 0,
                });
            }

            let (new_chunk_ids, new_documents, new_metadatas): (Vec<_>, Vec<_>, Vec<_>) =
                filtered_ids.into_iter().collect();
            chunk_ids_as_vec = new_chunk_ids;
            documents = new_documents;
            metadatas = new_metadatas;
        }

        let app_id = self.config.id.clone();

        let metadatas: Vec<HashMap<String, String>> = metadatas
            .into_iter()
            .map(|mut m| {
                m.insert("app_id".to_string(), app_id.clone().unwrap_or_default());
                m.insert("hash".to_string(), source_hash.clone());
                if let Some(metadata) = &metadata {
                    m.extend(metadata.iter().map(|(k, v)| (k.clone(), v.clone())));
                }
                m
            })
            .collect();

        if dry_run {
            return Ok(LoadAndEmbedResponse {
                documents,
                metadatas,
                ids: chunk_ids_as_vec,
                added_chunks_count: 0,
            });
        }

        let chunks_before_addition = self.db.count().await?;

        let _: () = self
            .db
            .add(&documents, &metadatas, &chunk_ids_as_vec)
            .await?;

        let chunks_after_addition = self.db.count().await?;

        Ok(LoadAndEmbedResponse {
            documents,
            metadatas,
            ids: chunk_ids_as_vec,
            added_chunks_count: chunks_after_addition - chunks_before_addition,
        })
    }

    async fn get_existing_document_id(
        &self,
        chunker: &Box<dyn Chunker>,
        source: &AddSource,
    ) -> Result<Option<String>> {
        let chunker_data_type = chunker.get_data_type();
        if DIRECT_DATA_TYPES.contains(&chunker_data_type) {
            // DirectDataTypes can't be updated.
            // Think of a text:
            // Either it's the same, then it won't change, so it's not an update.
            // Or it's different, then it will be added as a new text.
            Ok(None)
        } else if INDIRECT_DATA_TYPES.contains(&chunker_data_type) {
            let mut where_conditions = vec![("url".to_string(), source.to_string())];

            if let Some(app_id) = &self.config.id {
                where_conditions.push(("app_id".to_string(), app_id.clone()));
            }

            let VectorDBGetResponse { metadatas, .. } = self
                .db
                .get(None, Some(where_conditions.as_slice()), Some(1))
                .await?;

            if !metadatas.is_empty() {
                let doc_id = metadatas.first().unwrap().get("doc_id").cloned();
                Ok(doc_id)
            } else {
                Ok(None)
            }
        } else {
            Err(anyhow::anyhow!(
                "DataType {:?} not supported for checking existing document",
                chunker_data_type
            ))
        }
    }

    async fn query(&self, input_query: String, config: Option<BaseLLMConfig>) -> Result<()> {}
}
