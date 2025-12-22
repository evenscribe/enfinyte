use tonic::{Code, Request, Response, Status};
use umem_controller::MemoryController;
use umem_proto::{
    memory_service_server::MemoryService, ContextFilter, CreateMemoryRequest, DeleteMemoryRequest,
    GetMemoryRequest, ListMemoriesRequest, Memory, MemoryListResponse, MemoryResponse,
    SearchMemoriesRequest,
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

    async fn delete_memory(
        &self,
        request: Request<DeleteMemoryRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        MemoryController::delete(request.id)
            .await
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn get_memory(
        &self,
        request: Request<GetMemoryRequest>,
    ) -> Result<Response<MemoryResponse>, Status> {
        let request = request.into_inner();
        let memory = MemoryController::get(request.id)
            .await
            .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        Ok(Response::new(MemoryResponse {
            memory: Some(Self::map_memory(memory)),
        }))
    }

    async fn list_memories(
        &self,
        request: Request<ListMemoriesRequest>,
    ) -> Result<Response<MemoryListResponse>, Status> {
        let request = request.into_inner();

        if request.context.is_none() {
            return Err(Status::new(Code::InvalidArgument, "context must be passed"));
        }

        let memories = MemoryController::list_with_context(
            Self::map_context(request.context.unwrap())
                .map_err(|e| Status::new(Code::InvalidArgument, e.to_string()))?,
        )
        .await
        .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        Ok(Response::new(MemoryListResponse {
            memories: memories.into_iter().map(Self::map_memory).collect(),
        }))
    }

    async fn search_memories(
        &self,
        request: Request<SearchMemoriesRequest>,
    ) -> Result<Response<MemoryListResponse>, Status> {
        let request = request.into_inner();

        if request.context.is_none() {
            return Err(Status::new(Code::InvalidArgument, "context must be passed"));
        }

        let memories = MemoryController::search_with_context(
            Self::map_context(request.context.unwrap())
                .map_err(|e| Status::new(Code::InvalidArgument, e.to_string()))?,
            request.query,
        )
        .await
        .map_err(|e| Status::new(Code::Internal, e.to_string()))?;

        Ok(Response::new(MemoryListResponse {
            memories: memories.into_iter().map(Self::map_memory).collect(),
        }))
    }
}

impl ServiceImpl {
    fn map_context(
        context: ContextFilter,
    ) -> Result<umem_core::MemoryContext, umem_core::MemoryContextError> {
        umem_core::MemoryContext::new(context.user_id, context.agent_id, context.run_id)
    }

    fn map_memory(memory: umem_core::Memory) -> Memory {
        let context = memory.context();
        let content = memory.content();
        let signals = memory.signals();
        let temporal = memory.temporal();
        let provenance = memory.provenance();

        Memory {
            id: memory.get_id().to_string(),
            context: Some(umem_proto::MemoryContext {
                user_id: context.user_id().map(|s| s.to_string()),
                agent_id: context.agent_id().map(|s| s.to_string()),
                run_id: context.run_id().map(|s| s.to_string()),
            }),
            lifecycle: match memory.lifecycle() {
                umem_core::LifecycleState::Active => umem_proto::LifecycleState::Active as i32,
                umem_core::LifecycleState::Archived => umem_proto::LifecycleState::Archived as i32,
            },
            kind: match memory.kind() {
                umem_core::MemoryKind::Semantic => umem_proto::MemoryKind::Semantic as i32,
                umem_core::MemoryKind::Episodic => umem_proto::MemoryKind::Episodic as i32,
                umem_core::MemoryKind::Procedural => umem_proto::MemoryKind::Procedural as i32,
                umem_core::MemoryKind::Instruction => umem_proto::MemoryKind::Instruction as i32,
                umem_core::MemoryKind::Relational => umem_proto::MemoryKind::Relational as i32,
                umem_core::MemoryKind::Working => umem_proto::MemoryKind::Working as i32,
                umem_core::MemoryKind::Prospective => umem_proto::MemoryKind::Prospective as i32,
            },
            content: Some(umem_proto::MemoryContent {
                summary: content.summary().clone(),
                tags: content.tags().clone(),
            }),
            signals: Some(umem_proto::MemorySignals {
                certainty: signals.get_certainty(),
                salience: signals.get_salience(),
            }),
            temporal: Some(umem_proto::TemporalMetadata {
                created_at: temporal.created_at(),
                updated_at: temporal.updated_at(),
                archived_at: temporal.archived_at(),
            }),
            provenance: Some(umem_proto::Provenance {
                origin: match &provenance.origin {
                    umem_core::ProvenanceOrigin::User => umem_proto::ProvenanceOrigin::User as i32,
                    umem_core::ProvenanceOrigin::Agent => {
                        umem_proto::ProvenanceOrigin::Agent as i32
                    }
                },
                method: Some(umem_proto::ProvenanceMethod {
                    method: Some(match &provenance.method {
                        umem_core::ProvenanceMethod::Direct => {
                            umem_proto::provenance_method::Method::Direct(true)
                        }
                        umem_core::ProvenanceMethod::Extracted { model, prompt } => {
                            umem_proto::provenance_method::Method::Extracted(
                                umem_proto::ExtractedMethod {
                                    model: model.clone(),
                                    prompt: prompt.clone(),
                                },
                            )
                        }
                        umem_core::ProvenanceMethod::Summarized { model } => {
                            umem_proto::provenance_method::Method::Summarized(
                                umem_proto::SummarizedMethod {
                                    model: model.clone(),
                                },
                            )
                        }
                    }),
                }),
            }),
        }
    }
}
