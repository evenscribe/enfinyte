use anyhow::Result;
use tonic::transport::Server;
use tracing::info;
use umem_controller::MemoryController;
use umem_proto::memory_service_server::MemoryServiceServer;

mod service;
use service::ServiceImpl;

pub struct MemoryServiceGrpc;

impl MemoryServiceGrpc {
    pub async fn run_server(config: umem_config::Grpc, controller: MemoryController) -> Result<()> {
        let addr = config.server_addr;
        info!("Memory gRPC Server listening on {}", addr);

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(umem_proto::FILE_DESCRIPTOR_SET)
            .build_v1()?;

        Server::builder()
            .add_service(reflection_service)
            .add_service(MemoryServiceServer::new(ServiceImpl::new(controller)))
            .serve(addr)
            .await?;

        Ok(())
    }
}
