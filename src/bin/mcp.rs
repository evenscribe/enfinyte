use anyhow::Result;
use dotenv::dotenv;
use umem::tracing_conf;
use umem_config::CONFIG;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let _guard = tracing_conf::init_tracing()?;

    let mcp_handle = tokio::spawn(async move {
        umem_mcp::run_server(
            CONFIG
                .mcp
                .clone()
                .expect("no config [mcp] found in toml file"),
        )
        .await
    });

    mcp_handle.await??;

    Ok(())
}
