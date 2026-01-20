use crate::{MemoryContext, MemoryKind};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typed_builder::TypedBuilder;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum QueryError {
    #[error("limit must be greater than zero")]
    InvalidLimit,

    #[error("min_certainty must be in range [0.0, 1.0], got {0}")]
    InvalidMinCertainty(f32),

    #[error("min_salience must be in range [0.0, 1.0], got {0}")]
    InvalidMinSalience(f32),

    #[error("date range invalid: start ({start}) is after end ({end})")]
    InvalidDateRange { start: i64, end: i64 },

    #[error("query vector cannot be empty or whitespace")]
    EmptyQueryVector,

    #[error("context filter must specify at least one identifier")]
    EmptyContextFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporalFilter {
    created_range: (Option<i64>, Option<i64>),
    updated_range: (Option<i64>, Option<i64>),
}

impl TemporalFilter {
    pub fn new(
        created_after: Option<DateTime<Utc>>,
        created_before: Option<DateTime<Utc>>,
        updated_after: Option<DateTime<Utc>>,
        updated_before: Option<DateTime<Utc>>,
    ) -> Result<Self, QueryError> {
        if let (Some(start), Some(end)) = (created_after, created_before) {
            if start > end {
                return Err(QueryError::InvalidDateRange {
                    start: start.timestamp(),
                    end: end.timestamp(),
                });
            }
        }

        if let (Some(start), Some(end)) = (updated_after, updated_before) {
            if start > end {
                return Err(QueryError::InvalidDateRange {
                    start: start.timestamp(),
                    end: end.timestamp(),
                });
            }
        }

        Ok(Self {
            created_range: (
                created_after.map(|t| t.timestamp()),
                created_before.map(|t| t.timestamp()),
            ),
            updated_range: (
                updated_after.map(|t| t.timestamp()),
                updated_before.map(|t| t.timestamp()),
            ),
        })
    }

    pub fn created_range(&self) -> (Option<i64>, Option<i64>) {
        self.created_range
    }

    pub fn updated_range(&self) -> (Option<i64>, Option<i64>) {
        self.updated_range
    }

    pub fn has_created_range(&self) -> bool {
        self.created_range.0.is_some() || self.created_range.1.is_some()
    }

    pub fn has_updated_range(&self) -> bool {
        self.updated_range.0.is_some() || self.updated_range.1.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignalFilter {
    min_certainty: Option<f32>,
    min_salience: Option<f32>,
}

impl SignalFilter {
    pub fn new(min_certainty: Option<f32>, min_salience: Option<f32>) -> Result<Self, QueryError> {
        if let Some(c) = min_certainty {
            if !(0.0..=1.0).contains(&c) {
                return Err(QueryError::InvalidMinCertainty(c));
            }
        }

        if let Some(s) = min_salience {
            if !(0.0..=1.0).contains(&s) {
                return Err(QueryError::InvalidMinSalience(s));
            }
        }

        Ok(Self {
            min_certainty,
            min_salience,
        })
    }

    pub fn with_min_certainty(certainty: f32) -> Result<Self, QueryError> {
        if !(0.0..=1.0).contains(&certainty) {
            return Err(QueryError::InvalidMinCertainty(certainty));
        }

        Ok(Self {
            min_certainty: Some(certainty),
            ..Default::default()
        })
    }

    pub fn with_min_salience(salience: f32) -> Result<Self, QueryError> {
        if !(0.0..=1.0).contains(&salience) {
            return Err(QueryError::InvalidMinSalience(salience));
        }

        Ok(Self {
            min_salience: Some(salience),
            ..Default::default()
        })
    }

    pub fn min_certainty(&self) -> Option<f32> {
        self.min_certainty
    }

    pub fn min_salience(&self) -> Option<f32> {
        self.min_salience
    }

    pub fn is_empty(&self) -> bool {
        self.min_certainty.is_none() && self.min_salience.is_none()
    }
}

#[derive(TypedBuilder, Debug, Clone, Serialize, Deserialize, Default)]
pub struct Query {
    limit: u32,
    context: MemoryContext,
    #[builder(default = false)]
    include_archived: bool,
    #[builder(default, setter(strip_option))]
    vector: Option<Vec<f32>>,
    #[builder(default, setter(strip_option))]
    kinds: Option<Vec<MemoryKind>>,
    #[builder(default, setter(strip_option))]
    tags: Option<Vec<String>>,
    #[builder(default, setter(strip_option))]
    temporal: Option<TemporalFilter>,
    #[builder(default, setter(strip_option))]
    signals: Option<SignalFilter>,
}

impl Query {
    pub fn validate(&self) -> Result<(), QueryError> {
        if let Some(ref vector) = self.vector {
            if vector.is_empty() {
                return Err(QueryError::EmptyQueryVector);
            }
        }

        if let Some(ref signals) = self.signals {
            if let Some(c) = signals.min_certainty {
                if !(0.0..=1.0).contains(&c) {
                    return Err(QueryError::InvalidMinCertainty(c));
                }
            }
            if let Some(s) = signals.min_salience {
                if !(0.0..=1.0).contains(&s) {
                    return Err(QueryError::InvalidMinSalience(s));
                }
            }
        }

        if let Some(ref temporal) = self.temporal {
            if let (Some(start), Some(end)) = temporal.created_range {
                if start > end {
                    return Err(QueryError::InvalidDateRange { start, end });
                }
            }
            if let (Some(start), Some(end)) = temporal.updated_range {
                if start > end {
                    return Err(QueryError::InvalidDateRange { start, end });
                }
            }
        }

        Ok(())
    }

    pub fn limit(&self) -> u32 {
        self.limit
    }

    pub fn vector(&self) -> Option<&[f32]> {
        self.vector.as_deref()
    }

    pub fn context(&self) -> &MemoryContext {
        &self.context
    }

    pub fn kinds(&self) -> Option<&[MemoryKind]> {
        self.kinds.as_deref()
    }

    pub fn tags(&self) -> Option<&[String]> {
        self.tags.as_deref()
    }

    pub fn temporal(&self) -> Option<&TemporalFilter> {
        self.temporal.as_ref()
    }

    pub fn signals(&self) -> Option<&SignalFilter> {
        self.signals.as_ref()
    }

    pub fn include_archived(&self) -> bool {
        self.include_archived
    }

    pub fn active_only() -> Self {
        Self {
            include_archived: false,
            ..Default::default()
        }
    }
}
