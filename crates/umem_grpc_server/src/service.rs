use tonic::{Code, Request, Response, Status};
use umem_controller::MemoryController;
use umem_proto::{
    memory_service_server::MemoryService, ArchiveMemoryRequest, CreateMemoryRequest,
    DeleteMemoryRequest, GetMemoryRequest, ListMemoriesRequest, MemoryListResponse, MemoryResponse,
    SearchMemoriesRequest, UpdateMemoryRequest,
};

#[derive(Debug, Default)]
pub struct ServiceImpl;

#[tonic::async_trait]
impl MemoryService for ServiceImpl {
    async fn create_memory(
        &self,
        request: Request<CreateMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        let CreateMemoryRequest {
            user_id,
            raw_content,
            agent_id,
            run_id,
        } = request.into_inner();

        MemoryController::create(
            umem_controller::CreateMemoryRequest::builder()
                .raw_content(raw_content)
                .user_id(user_id)
                .agent_id(agent_id)
                .run_id(run_id)
                .build(),
        )
        .await
        .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn update_memory(
        &self,
        _request: Request<UpdateMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn archive_memory(
        &self,
        _request: Request<ArchiveMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn delete_memory(
        &self,
        _request: Request<DeleteMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_memory(
        &self,
        _request: Request<GetMemoryRequest>,
    ) -> Result<Response<MemoryResponse>, Status> {
        todo!()
    }

    async fn list_memories(
        &self,
        _request: Request<ListMemoriesRequest>,
    ) -> Result<Response<MemoryListResponse>, Status> {
        todo!()
    }

    async fn search_memories(
        &self,
        _request: Request<SearchMemoriesRequest>,
    ) -> Result<Response<MemoryListResponse>, Status> {
        todo!()
    }
}
