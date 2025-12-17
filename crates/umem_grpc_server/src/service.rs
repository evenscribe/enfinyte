use tonic::{Code, Request, Response, Status};
use umem_proto::generated::{
    memory_service_server::MemoryService, CreateMemoryRequest, DeleteMemoryRequest,
    GetMemoryRequest, ListMemoriesRequest, ListMemoriesResponse, Memory, SearchMemoriesRequest,
    UpdateMemoryRequest,
};

#[derive(Debug, Default)]
pub struct ServiceImpl;

#[tonic::async_trait]
impl MemoryService for ServiceImpl {
    async fn create_memory(
        &self,
        request: Request<CreateMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn update_memory(
        &self,
        request: Request<UpdateMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn delete_memory(
        &self,
        request: Request<DeleteMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        todo!()
    }

    async fn get_memory(
        &self,
        request: Request<GetMemoryRequest>,
    ) -> Result<Response<Memory>, Status> {
        todo!()
    }

    async fn list_memories(
        &self,
        request: Request<ListMemoriesRequest>,
    ) -> Result<Response<ListMemoriesResponse>, Status> {
        todo!()
    }

    async fn search_memories(
        &self,
        request: Request<SearchMemoriesRequest>,
    ) -> Result<Response<ListMemoriesResponse>, Status> {
        todo!()
    }
}
