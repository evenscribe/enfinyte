use crate::{
    HashMap,
    config::{AddConfig, ChunkerConfig, EngineConfig},
    data_type::DataType,
};
use anyhow::Result;

pub struct Engine<LLM, VectorDB, Embedder> {
    config: EngineConfig,
    chunker: Option<ChunkerConfig>,
    llm: LLM,
    db: VectorDB,
    embedder: Embedder,
    system_prompt: Option<String>,
}

pub enum AddSource {
    LocalFile(String),
    Url(String),
}

impl<LLM, VectorDB, Embedder> Engine<LLM, VectorDB, Embedder> {
    pub fn new(
        config: EngineConfig,
        llm: LLM,
        db: VectorDB,
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
        }
    }

    pub fn add(
        &self,
        source: AddSource,
        data_type: Option<DataType>,
        metadata: HashMap<String, String>,
        add_config: Option<AddConfig>,
        dry_run: bool,
    ) -> Result<String> {
        let add_config = match add_config {
            Some(config) => config,
            None => AddConfig {
                chunker: self.chunker.clone(),
                loader: None,
            },
        };

        let data_type = match data_type {
            Some(dt) => dt,
            None => detect_data_type(&source),
        };

        Ok(String::from("Data added successfully"))
    }
}
