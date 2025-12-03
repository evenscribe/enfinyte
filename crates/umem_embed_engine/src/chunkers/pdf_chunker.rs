use super::Chunker;
use crate::{
    config::{AddConfig, ChunkerConfig},
    data_type::DataType,
};
use anyhow::Result;
use text_splitter::{ChunkConfig, TextSplitter};
// Can also use anything else that implements the ChunkSizer
// trait from the text_splitter crate.
use tiktoken_rs::cl100k_base;

pub struct PdfChunker {
    config: ChunkerConfig,
}

impl PdfChunker {
    pub fn try_new(config: &AddConfig) -> Result<Self> {
        Ok(PdfChunker {
            config: config.chunker.clone().unwrap_or_default(),
        })
    }
}

impl Chunker for PdfChunker {
    fn get_data_type(&self) -> DataType {
        DataType::PdfFile
    }

    fn get_chunks(&self, content: String) -> Vec<String> {
        let tokenizer = tiktoken_rs::cl100k_base_singleton();
        let splitter =
            TextSplitter::new(ChunkConfig::new(self.config.chunk_size).with_sizer(tokenizer));
        splitter.chunks(&content).map(|c| c.to_string()).collect()
    }
}
