use std::sync::Arc;

use thiserror::Error;
use typed_builder::TypedBuilder;
use umem_ai::{LanguageModel, LanguageModelError};
use umem_config::CONFIG;
use umem_controller::MemoryController;
use umem_embeddings::{Embedder, EmbedderBase, EmbedderError};
use umem_grpc_server::MemoryServiceGrpc;
use umem_mcp::MemoryServiceMcp;
use umem_rerank::{RerankError, Reranker, RerankerBase};
use umem_vector_store::{VectorStore, VectorStoreBase, VectorStoreError};

#[derive(Debug, Error)]
pub enum MemoryMachineError {
    #[error("memory machine embedder failed : {0}")]
    EmbedderError(#[from] EmbedderError),

    #[error("memory machine vector_store failed : {0}")]
    VectorStoreError(#[from] VectorStoreError),

    #[error("memory machine rerank failed : {0}")]
    RerankError(#[from] RerankError),

    #[error("memory machine llm failed : {0}")]
    LanguageModelError(#[from] LanguageModelError),
}

#[derive(TypedBuilder)]
pub struct MemoryMachine {
    pub memory_controller: MemoryController,
}

#[derive(TypedBuilder)]
pub struct MemoryMachineOptions {
    vector_store: Option<Arc<dyn VectorStoreBase + Send + Sync>>,
    embedder: Option<Arc<dyn EmbedderBase + Send + Sync>>,
    reranker: Option<Arc<dyn RerankerBase + Send + Sync>>,
    language_model: Option<Arc<LanguageModel>>,
}

impl MemoryMachine {
    pub async fn new() -> Result<Self, MemoryMachineError> {
        Ok(Self {
            memory_controller: MemoryController {
                embedder: Embedder::get_embedder().await?,
                vector_store: VectorStore::get_store().await?,
                reranker: Reranker::get_reranker().await?,
                language_model: LanguageModel::get_model().await?,
            },
        })
    }

    pub async fn new_with(options: MemoryMachineOptions) -> Result<Self, MemoryMachineError> {
        let embedder = options.embedder.unwrap_or(Embedder::get_embedder().await?);
        let vector_store = options
            .vector_store
            .unwrap_or(VectorStore::get_store().await?);
        let reranker = options.reranker.unwrap_or(Reranker::get_reranker().await?);
        let language_model = options
            .language_model
            .unwrap_or(LanguageModel::get_model().await?);

        Ok(Self {
            memory_controller: MemoryController {
                embedder,
                vector_store,
                reranker,
                language_model,
            },
        })
    }

    pub async fn run_grpc(&self) -> anyhow::Result<()> {
        MemoryServiceGrpc::run_server(CONFIG.grpc.clone(), self.memory_controller.clone()).await?;
        Ok(())
    }

    pub async fn run_mcp(&self) -> anyhow::Result<()> {
        MemoryServiceMcp::run_server(CONFIG.mcp.clone(), self.memory_controller.clone()).await?;
        Ok(())
    }
}
