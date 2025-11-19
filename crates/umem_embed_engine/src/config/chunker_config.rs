use anyhow::{Result, anyhow};

#[derive(Clone)]
pub struct ChunkerConfig {
    chunk_size: usize,
    chunk_overlap: usize,
    length_function: fn(String) -> usize,
    min_chunk_size: usize,
}

// ---------------------------------------------------
// ---------------------------------------------------
// Builder Code
// ---------------------------------------------------
// ---------------------------------------------------

pub struct ChunkerConfigBuilder {
    chunk_size: usize,
    chunk_overlap: usize,
    length_function: Option<fn(String) -> usize>,
    min_chunk_size: usize,
}

impl ChunkerConfigBuilder {
    pub fn new() -> Self {
        ChunkerConfigBuilder {
            chunk_size: 2000,
            chunk_overlap: 0,
            length_function: None,
            min_chunk_size: 0,
        }
    }

    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    pub fn chunk_overlap(mut self, overlap: usize) -> Self {
        self.chunk_overlap = overlap;
        self
    }

    pub fn length_function(mut self, func: fn(String) -> usize) -> Self {
        self.length_function = Some(func);
        self
    }

    pub fn min_chunk_size(mut self, size: usize) -> Self {
        self.min_chunk_size = size;
        self
    }

    pub fn build(self) -> Result<ChunkerConfig> {
        if self.min_chunk_size >= self.chunk_size {
            return Err(anyhow!(
                "min_chunk_size {} should be less than chunk_size {}",
                self.min_chunk_size,
                self.chunk_size
            ));
        }

        if self.chunk_overlap > self.min_chunk_size {
            return Err(anyhow!(
                "min_chunk_size {} should be greater than or equal to chunk_overlap {}, otherwise its redundant",
                self.min_chunk_size,
                self.chunk_overlap
            ));
        }

        let length_function = match self.length_function {
            Some(func) => func,
            None => |text: String| text.len(),
        };

        Ok(ChunkerConfig {
            chunk_size: self.chunk_size,
            chunk_overlap: self.chunk_overlap,
            length_function,
            min_chunk_size: self.min_chunk_size,
        })
    }
}

