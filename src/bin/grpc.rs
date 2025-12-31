use anyhow::Result;
use dotenv::dotenv;
use umem::tracing_conf;
use umem_config::CONFIG;
use umem_grpc_server::MemoryServiceGrpc;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let _guard = tracing_conf::init_tracing()?;

    let grpc_handle = tokio::spawn(async move {
        MemoryServiceGrpc::run_server(CONFIG.grpc.clone().expect("[grpc] not set, check docs"))
            .await
    });

    grpc_handle.await??;

    Ok(())
}
