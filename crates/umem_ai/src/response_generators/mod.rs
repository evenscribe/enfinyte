pub mod generate_object;
mod generate_text;
pub mod messages;
pub use generate_text::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResponseGeneratorError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Transient(#[from] anyhow::Error),
}

pub type GenerateTextError = ResponseGeneratorError;
