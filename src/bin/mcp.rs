use anyhow::Result;
use dotenv::dotenv;
use umem::tracing;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let _guard = tracing::init_tracing()?;

    let mcp_handle = tokio::spawn(async move { umem_mcp::run_server().await });

    let _ = mcp_handle.await?;

    Ok(())
}
