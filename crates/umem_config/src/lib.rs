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
pub struct PgVector {
    pub url: String,
    pub collection_name: String,
    pub embedding_model_dimensions: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub enum VectorStore {
    #[serde(rename = "qdrant")]
    Qdrant(Qdrant),
    #[serde(rename = "pgvector")]
    PgVector(PgVector),
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub work_os: WorkOs,
    pub cloudflare: Cloudflare,
    pub vector_store: VectorStore,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AppConfig {
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Could not determine home directory");
        let config_path = home.join(".config/enfinyte/enfinyte.toml");
        let config_path = config_path
            .to_str()
            .expect("Config path contains invalid UTF-8");

        Config::builder()
            .add_source(File::with_name(config_path))
            .build()
            .unwrap_or_else(|e| panic!("{e}"))
            .try_deserialize::<Self>()
            .unwrap_or_else(|e| panic!("{e}"))
    }
}

lazy_static! {
    pub static ref CONFIG: AppConfig = AppConfig::new();
}
