use super::MemoryMachine;
use thiserror::Error;
use umem_controller::{
    CreateMemoryOptions, CreateMemoryRequest, MemoryController, MemoryControllerError,
};

#[derive(Debug, Error)]
pub enum MemoryMachineError {
    #[error("memory controller action failed with : {0}")]
    MemoryControllerError(#[from] MemoryControllerError),
}

impl<'alloc> MemoryMachine<'alloc> {
    pub async fn add_memory(
        memory: CreateMemoryRequest,
        options: Option<CreateMemoryOptions>,
    ) -> Result<(), MemoryMachineError> {
        MemoryController::create(memory, options).await?;
        Ok(())
    }
    pub fn update_memory() {}
    pub fn search_memory() {}
}
