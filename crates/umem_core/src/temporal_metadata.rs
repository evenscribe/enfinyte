use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TemporalMetadataError {
    #[error("updated_at ({updated}) cannot be earlier than created_at ({created})")]
    UpdatedBeforeCreated { created: i64, updated: i64 },

    #[error("archived_at ({archived}) cannot be earlier than created_at ({created})")]
    ArchivedBeforeCreated { created: i64, archived: i64 },

    #[error("archived_at ({archived}) cannot be earlier than updated_at ({updated})")]
    ArchivedBeforeUpdated { archived: i64, updated: i64 },
}

#[derive(Serialize, Default, Deserialize)]
pub struct TemporalMetadata {
    created_at: i64,
    updated_at: Option<i64>,
    archived_at: Option<i64>,
}

impl TemporalMetadata {
    pub fn new(created_at: DateTime<Utc>) -> Self {
        Self {
            created_at: created_at.timestamp(),
            updated_at: None,
            archived_at: None,
        }
    }

    pub fn with_times(
        created_at: i64,
        updated_at: Option<i64>,
        archived_at: Option<i64>,
    ) -> Result<Self, TemporalMetadataError> {
        if let Some(updated) = updated_at {
            if updated < created_at {
                return Err(TemporalMetadataError::UpdatedBeforeCreated {
                    created: created_at,
                    updated,
                });
            }
        }

        if let Some(archived) = archived_at {
            if archived < created_at {
                return Err(TemporalMetadataError::ArchivedBeforeCreated {
                    created: created_at,
                    archived,
                });
            }
        }

        if let (Some(updated), Some(archived)) = (updated_at, archived_at) {
            if archived < updated {
                return Err(TemporalMetadataError::ArchivedBeforeUpdated { updated, archived });
            }
        }

        Ok(Self {
            created_at,
            updated_at,
            archived_at,
        })
    }
    pub fn validate(&self) -> Result<(), TemporalMetadataError> {
        if let Some(updated) = self.updated_at {
            if updated < self.created_at {
                return Err(TemporalMetadataError::UpdatedBeforeCreated {
                    created: self.created_at,
                    updated,
                });
            }
        }

        if let Some(archived) = self.archived_at {
            if archived < self.created_at {
                return Err(TemporalMetadataError::ArchivedBeforeCreated {
                    created: self.created_at,
                    archived,
                });
            }
        }

        if let (Some(updated), Some(archived)) = (self.updated_at, self.archived_at) {
            if archived < updated {
                return Err(TemporalMetadataError::ArchivedBeforeUpdated { updated, archived });
            }
        }

        Ok(())
    }

    pub fn mark_updated(&mut self, time: i64) -> Result<(), TemporalMetadataError> {
        if time < self.created_at {
            return Err(TemporalMetadataError::UpdatedBeforeCreated {
                created: self.created_at,
                updated: time,
            });
        }

        if let Some(current_updated) = self.updated_at {
            if time < current_updated {
                return Err(TemporalMetadataError::UpdatedBeforeCreated {
                    created: self.created_at,
                    updated: time,
                });
            }
        }

        self.updated_at = Some(time);
        Ok(())
    }

    pub fn mark_archived(&mut self, time: i64) -> Result<(), TemporalMetadataError> {
        if time < self.created_at {
            return Err(TemporalMetadataError::ArchivedBeforeCreated {
                created: self.created_at,
                archived: time,
            });
        }

        if let Some(updated) = self.updated_at {
            if time < updated {
                return Err(TemporalMetadataError::ArchivedBeforeUpdated {
                    updated,
                    archived: time,
                });
            }
        }
        self.archived_at = Some(time);
        Ok(())
    }

    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }

    pub fn last_modified(&self) -> i64 {
        self.updated_at.unwrap_or(self.created_at)
    }

    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    pub fn updated_at(&self) -> Option<i64> {
        self.updated_at
    }

    pub fn archived_at(&self) -> Option<i64> {
        self.archived_at
    }
}
