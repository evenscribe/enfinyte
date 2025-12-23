use anyhow::Result;
use tonic::transport::Server;
use tracing::info;
use umem_proto::memory_service_server::MemoryServiceServer;

mod service;
use service::ServiceImpl;

pub struct MemoryServiceGrpc;

impl MemoryServiceGrpc {
    pub async fn run_server(config: umem_config::Grpc) -> Result<()> {
        let addr = config.server_addr;
        info!("Memory gRPC Server listening on {}", addr);
        Server::builder()
            .add_service(MemoryServiceServer::new(ServiceImpl))
            .serve(addr)
            .await?;
        Ok(())
    }
}
