use anyhow::Result;
use dotenv::dotenv;
use umem::tracing_conf;
use umem_memory_machine::MemoryMachine;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let _guard = tracing_conf::init_tracing()?;

    let machine = MemoryMachine::new().await?;
    let grpc_handle = tokio::spawn(async move { machine.run_grpc().await });
    grpc_handle.await??;

    Ok(())
}
