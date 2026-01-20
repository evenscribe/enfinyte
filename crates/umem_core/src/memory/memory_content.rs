use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MemoryContentError {
    #[error("summary must not be empty or whitespace")]
    EmptySummary,

    #[error("tag must not be empty or whitespace")]
    EmptyTag,

    #[error("duplicate tag: {0}")]
    DuplicateTag(String),

    #[error("tag not found: {0}")]
    TagNotFound(String),
}

#[derive(Debug, schemars::JsonSchema, Clone, PartialEq, Default, Eq, Serialize, Deserialize)]
pub struct MemoryContent {
    summary: String,
    tags: Vec<String>,
}

impl MemoryContent {
    pub fn new(summary: impl Into<String>, tags: Vec<String>) -> Result<Self, MemoryContentError> {
        let summary = summary.into();
        let summary = summary.trim();

        if summary.is_empty() {
            return Err(MemoryContentError::EmptySummary);
        }

        let mut seen = FxHashSet::default();
        let mut normalized_tags = Vec::with_capacity(tags.len());

        for tag in tags {
            let tag = tag.trim();

            if tag.is_empty() {
                return Err(MemoryContentError::EmptyTag);
            }

            let tag = tag.to_ascii_lowercase();

            if !seen.insert(tag.clone()) {
                return Err(MemoryContentError::DuplicateTag(tag));
            }

            normalized_tags.push(tag);
        }

        Ok(Self {
            summary: summary.to_string(),
            tags: normalized_tags,
        })
    }

    pub fn with_summary(summary: impl Into<String>) -> Result<Self, MemoryContentError> {
        Self::new(summary, Vec::new())
    }

    pub fn is_untagged(&self) -> bool {
        self.tags.is_empty()
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) -> Result<(), MemoryContentError> {
        let tag = tag.into();
        let tag = tag.trim();

        if tag.is_empty() {
            return Err(MemoryContentError::EmptyTag);
        }

        let tag = tag.to_ascii_lowercase();

        if self.tags.iter().any(|t| t == &tag) {
            return Err(MemoryContentError::DuplicateTag(tag));
        }

        self.tags.push(tag);
        Ok(())
    }

    pub fn remove_tag(&mut self, tag: impl Into<String>) -> Result<(), MemoryContentError> {
        let tag = tag.into();
        let tag = tag.trim();

        if tag.is_empty() {
            return Err(MemoryContentError::EmptyTag);
        }

        let tag = tag.to_ascii_lowercase();

        match self.tags.iter().position(|x| x.eq(&tag)) {
            Some(pos) => {
                self.tags.remove(pos);
                Ok(())
            }
            None => Err(MemoryContentError::TagNotFound(tag)),
        }
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn summary(&self) -> &String {
        &self.summary
    }
}
