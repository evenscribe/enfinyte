use crate::{
    HashMap,
    chunkers::Chunker,
    config::{AddConfig, ChunkerConfig, EngineConfig},
    data_formatter::DataFormatter,
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

        let source_hash = blake3::hash(request.source.as_bytes());

        let data_formatter =
            DataFormatter::try_new(data_type, request.loader, request.chunker, &add_config)?;

        let load_and_embed_response = self.load_and_embed(
            &data_formatter,
            &request.source,
            source_hash,
            &request.metadata,
            add_config,
            request.dry_run,
        );

        self.user_asks.push(UserAsk {
            source: request.source,
            data_type,
            metadata: request.metadata,
        });

        Ok(String::from("Data added successfully"))
    }

    fn load_and_embed(
        &self,
        data_formatter: &DataFormatter,
        source: &AddSource,
        source_hash: blake3::Hash,
        metadata: &std::collections::HashMap<String, String, rustc_hash::FxBuildHasher>,
        add_config: AddConfig,
        dry_run: bool,
    ) -> Result<()> {
        let existing_doc_id = self.get_existing_document_id(&data_formatter.chunker, source)?;
        let app_id = self.config.id.clone();

        let embeddings_data = data_formatter.create_chunks(source, app_id, add_config.chunker);

        todo!()
    }

    fn get_existing_document_id(
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

            let VectorDBGetResponse { metadatas, .. } =
                self.db
                    .get(None, Some(where_conditions.as_slice()), Some(1))?;

            if !metadatas.is_empty() {
                let doc_id = metadatas.first().unwrap().doc_id.clone();
                Ok(Some(doc_id))
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
}
