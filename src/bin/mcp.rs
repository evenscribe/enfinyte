use anyhow::Result;
use dotenv::dotenv;
use umem::tracing_conf;
use umem_memory_machine::MemoryMachine;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let _guard = tracing_conf::init_tracing()?;

    let machine = MemoryMachine::new().await?;
    let mcp_handle = tokio::spawn(async move { machine.run_grpc().await });
    mcp_handle.await??;

    Ok(())
}
