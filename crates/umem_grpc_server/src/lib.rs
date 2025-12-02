use anyhow::Result;
use tonic::transport::Server;
use tracing::info;
use umem_proto::generated;

mod service;
use service::ServiceImpl;

pub struct MemoryServiceGrpc;

impl MemoryServiceGrpc {
    pub async fn run_server(addr: &str) -> Result<()> {
        let addr = addr.parse()?;
        info!("Memory gRPC Server listening on {}", addr);
        Server::builder()
            .add_service(generated::memory_service_server::MemoryServiceServer::new(
                ServiceImpl,
            ))
            .serve(addr)
            .await?;
        Ok(())
    }
}
