use super::{ChunkerConfig, LoaderConfig};

pub struct AddConfig {
    pub(crate) chunker: Option<ChunkerConfig>,
    pub(crate) loader: Option<LoaderConfig>,
}
