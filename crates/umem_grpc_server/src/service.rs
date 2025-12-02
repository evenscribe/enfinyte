use tonic::{Code, Request, Response, Status};
use umem_controller::{CreateMemoryRequestBuilder, MemoryController, UpdateMemoryRequestBuilder};
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
        let request = request.into_inner();
        let builder = CreateMemoryRequestBuilder::new(request.user_id, request.content)
            .priority(request.priority)
            .tags(if request.tags.is_empty() {
                None
            } else {
                Some(request.tags)
            })
            .parent_id(request.parent_id);

        MemoryController::create(builder)
            .await
            .map_err(|err| Status::new(Code::Unknown, err.to_string()))?;
        Ok(Response::new(()))
    }

    async fn update_memory(
        &self,
        request: Request<UpdateMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        let builder = UpdateMemoryRequestBuilder::new(request.id)
            .priority(request.priority)
            .content(request.content)
            .tags(if request.tags.is_empty() {
                None
            } else {
                Some(request.tags)
            })
            .parent_id(request.parent_id);

        MemoryController::update(builder)
            .await
            .map_err(|err| Status::new(Code::Unknown, err.to_string()))?;

        Ok(Response::new(()))
    }

    async fn delete_memory(
        &self,
        request: Request<DeleteMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        let memory = MemoryController::delete(request.id)
            .await
            .map_err(|err| Status::new(Code::Unknown, err.to_string()))?;
        Ok(Response::new(memory))
    }

    async fn get_memory(
        &self,
        request: Request<GetMemoryRequest>,
    ) -> Result<Response<Memory>, Status> {
        let request = request.into_inner();
        let memory = MemoryController::get(request.id)
            .await
            .map_err(|err| Status::new(Code::Unknown, err.to_string()))?;
        Ok(Response::new(memory))
    }

    async fn list_memories(
        &self,
        request: Request<ListMemoriesRequest>,
    ) -> Result<Response<ListMemoriesResponse>, Status> {
        let request = request.into_inner();
        let memories = MemoryController::list(request.user_id)
            .await
            .map_err(|err| Status::new(Code::Unknown, err.to_string()))?;

        Ok(Response::new(ListMemoriesResponse { memories }))
    }

    async fn search_memories(
        &self,
        request: Request<SearchMemoriesRequest>,
    ) -> Result<Response<ListMemoriesResponse>, Status> {
        let request = request.into_inner();
        let memories = MemoryController::search(request.user_id, request.query)
            .await
            .map_err(|err| Status::new(Code::Unknown, err.to_string()))?;

        Ok(Response::new(ListMemoriesResponse { memories }))
    }
}
