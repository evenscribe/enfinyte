use config::{Config, File};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize, Clone)]
pub struct Cloudflare {
    pub account_id: String,
    pub api_token: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAI {
    pub api_key: String,
    pub base_url: String,
    pub default_headers: Option<Vec<(String, String)>>,
    pub organization: Option<String>,
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AmazonBedrock {
    pub region: String,
    pub key_id: String,
    pub access_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Provider {
    #[serde(rename = "openai")]
    OpenAI(OpenAI),

    #[serde(rename = "amazon_bedrock")]
    AmazonBedrock(AmazonBedrock),
}

#[derive(Debug, Deserialize, Clone)]
pub struct LanguageModel {
    pub provider: Provider,
    pub model: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Grpc {
    pub server_addr: SocketAddr,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Mcp {
    pub server_addr: SocketAddr,
    pub remote_url: String,
    pub jwks_url: String,
    pub work_os: WorkOs,
}

#[derive(Debug, Deserialize, Clone)]
pub enum EmbeddingModel {
    #[serde(rename = "cloudflare")]
    Cloudflare(Cloudflare),
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
    pub embedding_model_dimensions: u16,
    pub collection_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub enum VectorStore {
    #[serde(rename = "qdrant")]
    Qdrant(Qdrant),
    #[serde(rename = "pgvector")]
    PgVector(PgVector),
}

#[derive(Debug, Deserialize, Clone)]
pub struct RerankingModel {
    pub provider: Provider,
    pub model: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub vector_store: VectorStore,
    pub embedding_model: EmbeddingModel,
    pub language_model: LanguageModel,
    pub reranking_model: RerankingModel,
    pub mcp: Mcp,
    pub grpc: Grpc,
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
