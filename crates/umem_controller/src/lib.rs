use thiserror::Error;

mod create_memory;
mod delete_memory;
mod get_memory;
mod list_memory;
mod search_memory;
mod update_memory;

pub use create_memory::*;
pub use delete_memory::*;
pub use get_memory::*;
pub use list_memory::*;
pub use search_memory::*;
pub use update_memory::*;

#[derive(Debug, Error)]
pub enum MemoryControllerError {
    #[error("create memory failed with: {0}")]
    CreateMemoryError(#[from] CreateMemoryError),

    #[error("update memory failed with: {0}")]
    UpdateMemoryError(#[from] UpdateMemoryError),

    #[error("get memory failed with: {0}")]
    GetMemoryError(#[from] GetMemoryError),

    #[error("delete memory failed with: {0}")]
    DeleteMemoryError(#[from] DeleteMemoryError),

    #[error("list memory failed with: {0}")]
    ListMemoryError(#[from] ListMemoryError),

    #[error("search memory failed with: {0}")]
    SearchMemoryError(#[from] SearchMemoryError),
}

#[derive(Debug, Default)]
pub struct MemoryController;
