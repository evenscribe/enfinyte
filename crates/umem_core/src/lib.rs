use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typed_builder::TypedBuilder;
use uuid::Uuid;

pub mod credence;
pub mod lifecycle_state;
pub mod memory_content;
pub mod memory_context;
pub mod memory_kind;
pub mod memory_signals;
pub mod provenance;
pub mod query;
pub mod temporal_metadata;

use crate::credence::{Credence, CredenceError};
pub use crate::{
    lifecycle_state::*, memory_content::*, memory_context::*, memory_kind::*, memory_signals::*,
    provenance::*, query::*, temporal_metadata::*,
};

#[derive(Debug, Error, Clone)]
pub enum MemoryError {
    #[error("invalid memory kind: {0}")]
    MemoryKindError(#[from] ParseMemoryKindError),

    #[error("invalid memory content: {0}")]
    ContentError(#[from] MemoryContentError),

    #[error("invalid memory context: {0}")]
    ContextError(#[from] MemoryContextError),

    #[error("invalid memory signals: {0}")]
    SignalsError(#[from] MemorySignalsError),

    #[error("invalid credence: {0}")]
    CredenceError(#[from] CredenceError),

    #[error("invalid provenance: {0}")]
    ProvenanceError(#[from] ProvenanceMethodError),

    #[error("invalid temporal metadata: {0}")]
    TemporalMetadataError(#[from] TemporalMetadataError),

    #[error("invalid lifecycle state: {0}")]
    LifecycleStateError(#[from] ParseLifecycleStateError),

    #[error("lifecycle state is Archived but archived_at timestamp is not set")]
    ArchivedWithoutTimestamp,

    #[error("lifecycle state is Active but archived_at timestamp is set")]
    ActiveWithArchivedTimestamp,
}

#[derive(TypedBuilder, Serialize, Default, Deserialize)]
pub struct Memory {
    id: Uuid,
    context: MemoryContext,
    lifecycle: LifecycleState,
    kind: MemoryKind,
    content: MemoryContent,
    signals: MemorySignals,
    temporal: TemporalMetadata,
    provenance: Provenance,
}

type Result<T> = std::result::Result<T, MemoryError>;

impl Memory {
    pub fn validate(&self) -> Result<()> {
        self.context.validate()?;
        self.temporal.validate()?;
        self.provenance.validate()?;

        match (&self.lifecycle, self.temporal.archived_at()) {
            (LifecycleState::Archived, None) => {
                return Err(MemoryError::ArchivedWithoutTimestamp);
            }
            (LifecycleState::Active, Some(_)) => {
                return Err(MemoryError::ActiveWithArchivedTimestamp);
            }
            _ => {}
        }

        Ok(())
    }

    pub fn mark_updated(&mut self, time: chrono::DateTime<chrono::Utc>) -> Result<()> {
        self.temporal.mark_updated(time.timestamp())?;
        Ok(())
    }

    pub fn archive(&mut self, time: chrono::DateTime<chrono::Utc>) -> Result<()> {
        self.temporal.mark_archived(time.timestamp())?;
        self.lifecycle = LifecycleState::Archived;
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.lifecycle.is_active()
    }

    pub fn is_archived(&self) -> bool {
        self.lifecycle.is_archived()
    }

    pub fn score(&self) -> f32 {
        self.signals.get_certainty() * self.signals.get_salience()
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    pub fn get_summary(&self) -> &String {
        self.content.summary()
    }

    pub fn context(&self) -> &MemoryContext {
        &self.context
    }

    pub fn lifecycle(&self) -> &LifecycleState {
        &self.lifecycle
    }

    pub fn kind(&self) -> &MemoryKind {
        &self.kind
    }

    pub fn content(&self) -> &MemoryContent {
        &self.content
    }

    pub fn signals(&self) -> &MemorySignals {
        &self.signals
    }

    pub fn temporal(&self) -> &TemporalMetadata {
        &self.temporal
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    pub fn gen_dummy() -> Result<Memory> {
        Ok(Memory::builder()
            .id(Uuid::new_v4())
            .content(MemoryContent::new("content", vec![])?)
            .context(MemoryContext::for_user("test")?)
            .kind(MemoryKind::Working)
            .signals(MemorySignals::new(
                Credence::new(0.2)?,
                Credence::new(0.3)?,
            )?)
            .provenance(Provenance {
                origin: ProvenanceOrigin::User,
                method: ProvenanceMethod::Direct,
            })
            .lifecycle(LifecycleState::Active)
            .temporal(TemporalMetadata::new(Utc::now()))
            .build())
    }
}
