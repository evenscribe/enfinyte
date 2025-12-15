use anyhow::Result;
use dotenv::dotenv;
use umem::tracing;
use umem_grpc_server::MemoryServiceGrpc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let _guard = tracing::init_tracing()?;

    let grpc_handle =
        tokio::spawn(async move { MemoryServiceGrpc::run_server("0.0.0.0:5051").await });

    let _ = grpc_handle.await?;

    Ok(())
}
