use anyhow::Result;
use config::{Config, File};
use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Cloudflare {
    pub account_id: String,
    pub api_token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkOs {
    pub client_id: String,
    pub client_secret: String,
    pub authkit_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Qdrant {
    pub url: String,
    pub key: String,
    pub collection_name: String,
    pub chunk_size: u16,
    pub embedding_model_dimensions: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VectorStore {
    pub qdrant: Option<Qdrant>,
    #[serde(skip)]
    pub kind: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub work_os: WorkOs,
    pub cloudflare: Cloudflare,
    pub vector_store: VectorStore,
}

impl AppConfig {
    fn validate(mut self) -> Result<Self> {
        if self.vector_store.qdrant.is_none() {
            anyhow::bail!("At least one of the [vector_store.<backend>] must be set.");
        }
        if self.vector_store.qdrant.is_some() {
            self.vector_store.kind = "qdrant".into();
        }
        Ok(self)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AppConfig {
    pub fn new() -> Self {
        let home = env!("HOME");
        let config_path = home.to_owned() + "/.config/enfinyte/enfinyte.toml";

        Config::builder()
            .add_source(File::with_name(&config_path))
            .build()
            .unwrap_or_else(|e| panic!("{e}"))
            .try_deserialize::<Self>()
            .unwrap_or_else(|e| panic!("{e}"))
            .validate()
            .unwrap_or_else(|e| panic!("{e}"))
    }
}

lazy_static! {
    pub static ref CONFIG: AppConfig = AppConfig::new();
}
