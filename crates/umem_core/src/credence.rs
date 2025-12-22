use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CredenceError {
    #[error("credence must be a finite number")]
    NotFinite,
    #[error("credence must be in the unit interval [0.0, 1.0], got {0}")]
    OutOfRange(f32),
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Credence(f32);

impl Credence {
    pub fn new(value: f32) -> Result<Self, CredenceError> {
        if !value.is_finite() {
            return Err(CredenceError::NotFinite);
        }

        if !(0.0..=1.0).contains(&value) {
            return Err(CredenceError::OutOfRange(value));
        }

        Ok(Self(value))
    }

    pub fn get(self) -> f32 {
        self.0
    }
}
