use crate::credence::Credence;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum MemorySignalsError {
    #[error("certainty and salience cannot both be zero")]
    DeadMemory,
}

#[derive(Serialize, Debug, schemars::JsonSchema, Clone, Default, Deserialize)]
pub struct MemorySignals {
    certainty: Credence,
    salience: Credence,
}

impl MemorySignals {
    pub fn new(certainty: Credence, salience: Credence) -> Result<Self, MemorySignalsError> {
        if certainty.get() == 0.0 && salience.get() == 0.0 {
            return Err(MemorySignalsError::DeadMemory);
        }

        Ok(Self {
            certainty,
            salience,
        })
    }

    pub fn get_certainty(&self) -> f32 {
        self.certainty.get()
    }

    pub fn get_salience(&self) -> f32 {
        self.salience.get()
    }

    pub fn is_weak(&self) -> bool {
        self.certainty.get() < 0.3 && self.salience.get() < 0.3
    }
}
