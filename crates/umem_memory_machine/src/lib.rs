use std::sync::Arc;

use typed_builder::TypedBuilder;
use umem_ai::LanguageModel;
use umem_controller::MemoryController;
use umem_embeddings::EmbedderBase;
use umem_rerank::RerankerBase;
use umem_vector_store::VectorStoreBase;

#[derive(TypedBuilder)]
pub struct MemoryMachine {
    pub memory_controller: MemoryController,
}

impl MemoryMachine {
    pub fn new(
        embedder: Arc<dyn EmbedderBase + Send + Sync>,
        vector_store: Arc<dyn VectorStoreBase + Send + Sync>,
        reranker: Arc<dyn RerankerBase + Send + Sync>,
        language_model: Arc<LanguageModel>,
    ) -> Self {
        Self {
            memory_controller: MemoryController {
                embedder,
                vector_store,
                reranker,
                language_model,
            },
        }
    }
}
