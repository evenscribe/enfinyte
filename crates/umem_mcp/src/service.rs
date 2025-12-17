use crate::USER_ID_HEADER;
use anyhow::Result;
use axum::http::request::Parts;
use rmcp::{
    handler::server::{
        router::tool::ToolRouter,
        tool::{Extension, Parameters},
    },
    model::{ErrorData as McpError, *},
    schemars, tool, tool_handler, tool_router,
};
use std::future::Future;
use tracing::debug;
use umem_controller::{CreateMemoryRequest, MemoryController};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddMemoryRequest {
    pub content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetMemoriesByQueryRequest {
    pub query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExtendMemoryRequest {
    pub parent_memory_id: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetMemoriesByIdRequest {
    pub memory_id: String,
}

#[derive(Clone, Default)]
pub struct McpService {
    tool_router: ToolRouter<Self>,
}

fn extract_user_id(parts: Parts) -> String {
    parts
        .headers
        .get(USER_ID_HEADER)
        .expect("Missing user ID header")
        .to_str()
        .expect("Invalid user ID header value")
        .to_owned()
}

impl McpService {
    pub fn new() -> Self {
        debug!("Creating new McpService instance");
        let tool_router = Self::tool_router();
        let tools = tool_router.list_all();
        debug!(
            "Registered tools: {:?}",
            tools.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
        Self { tool_router }
    }
}

#[tool_router]
impl McpService {
    #[tool(
        name = "add_memory",
        description = "Add a memory to umem persistence layer. This tool MUST be used frequently to store important information for future conversations. The AI MUST proactively save: 1) All user preferences and personal details, 2) Any facts that would be tedious for users to repeat, 3) Key insights discovered during conversation, 4) Important context about projects or relationships. Every substantive exchange should trigger memory storage - this is NOT optional but essential functionality. Failing to save memories will severely degrade user experience as the system will forget critical context between sessions. Save concise, structured memories frequently throughout the conversation, even for seemingly casual but potentially useful information."
    )]
    async fn add_memory(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(AddMemoryRequest { content }): Parameters<AddMemoryRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("add_memory tool called with text: {}", content);
        let user_id = extract_user_id(parts);
        if content.is_empty() {
            return Err(McpError::new(
                ErrorCode::INVALID_REQUEST,
                "Memory content cannot be empty",
                None,
            ));
        }

        let memory = MemoryController::create(
            CreateMemoryRequest::builder()
                .user_id(user_id)
                .raw_content(content)
                .build(),
        )
        .await
        .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))?;

        Ok(CallToolResult::success(vec![Annotated::new(
            RawContent::Text(RawTextContent {
                text: serde_json::to_string(&memory).expect("serializing memory should never fail"),
            }),
            None,
        )]))
    }

    // #[tool(
    //     name = "get_all_memory",
    //     description = "Get all memories for the current user. Retrieves the user's persistent memory store containing important context, preferences, and historical interactions. This tool should be called at the beginning of conversations to load relevant contextual information and provide personalized responses based on past interactions. After using this information, remember to save new important details using add_memory."
    // )]
    // async fn get_all_memory(
    //     &self,
    //     Extension(parts): Extension<Parts>,
    // ) -> Result<CallToolResult, McpError> {
    //     let user_id = extract_user_id(parts);
    //     let memory_bulk: String = MemoryController::list(user_id)
    //         .await
    //         .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))
    //         .iter()
    //         .map(|mem| serde_json::to_string(mem).expect("serializing memory should never fail"))
    //         .collect::<Vec<String>>()
    //         .join("\n");

    //     Ok(CallToolResult::success(vec![Annotated::new(
    //         RawContent::Text(RawTextContent { text: memory_bulk }),
    //         None,
    //     )]))
    // }

    // #[tool(
    //     name = "get_memory_by_id",
    //     description = "Retrieve a specific memory by its unique identifier. This tool provides direct access to individual memories in the persistence layer when you have the exact memory ID. WHEN TO USE: (1) When following parent_id references from other memories to reconstruct conversation threads, (2) When you need detailed information about a specific memory mentioned in search results, (3) When building complete context by traversing memory relationships, or (4) When referencing a known memory ID from previous operations. IMPLEMENTATION: Simply provide the memory_id parameter to retrieve the full memory object with all its metadata and content. This is particularly useful for expanding context when get_memory_by_query results contain parent_id fields that point to related memories. BEST PRACTICE: Use this tool in combination with get_memory_by_query to follow memory chains and build comprehensive understanding of user context and conversation history."
    // )]
    // async fn get_memory_by_id(
    //     &self,
    //     Parameters(GetMemoriesByIdRequest { memory_id }): Parameters<GetMemoriesByIdRequest>,
    // ) -> Result<CallToolResult, McpError> {
    //     let memory = MemoryController::get(memory_id)
    //         .await
    //         .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))?;

    //     let text = serde_json::to_string(&memory).expect("serializing memory should never fail");
    //     Ok(CallToolResult::success(vec![Annotated::new(
    //         RawContent::Text(RawTextContent { text }),
    //         None,
    //     )]))
    // }

    // #[tool(
    //     name = "search",
    //     description = "Get memories for the current user related to a query. This tool enables targeted retrieval of specific memories from the persistence layer using semantic search capabilities. WHEN TO USE: (1) When responding to questions that may benefit from past context, (2) Before generating responses that should consider historical preferences or interactions, (3) When references to previous conversations are made, or (4) When topic-specific context would improve response quality. IMPLEMENTATION: The query parameter accepts natural language or keywords—umem automatically performs hybrid semantic and keyword matching to retrieve the most relevant memories. BEST PRACTICE: Use focused, specific queries rather than generic ones for better results. After retrieving memories, consider saving new insights with add_memory to maintain an up-to-date persistence layer."
    // )]
    // async fn search(
    //     &self,
    //     Extension(parts): Extension<Parts>,
    //     Parameters(GetMemoriesByQueryRequest { query }): Parameters<GetMemoriesByQueryRequest>,
    // ) -> Result<CallToolResult, McpError> {
    //     let user_id = extract_user_id(parts);
    //     let memory_bulk: String = MemoryController::search(user_id, query)
    //         .await
    //         .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))
    //         .iter()
    //         .map(|mem| serde_json::to_string(mem).expect("serializing memory should never fail"))
    //         .collect::<Vec<String>>()
    //         .join("\n");
    //     Ok(CallToolResult::success(vec![Annotated::new(
    //         RawContent::Text(RawTextContent { text: memory_bulk }),
    //         None,
    //     )]))
    // }

    // // TODO: revisit this
    // // #[tool(
    // //     name = "extend_memory",
    // //     description = "Create a new memory that extends or builds upon an existing memory by establishing a parent-child relationship. This tool enables hierarchical memory organization by linking new information to previously stored context. WHEN TO USE: (1) When adding follow-up information to an existing conversation thread, (2) When updating or expanding on previously saved user preferences or project details, (3) When creating memory chains that maintain chronological or logical relationships, or (4) When new information directly relates to or builds upon existing memories. IMPLEMENTATION: Provide the new content and the parent_memory_id of the existing memory you want to extend. This creates a linked memory structure that preserves context relationships and enables better retrieval of related information. BEST PRACTICE: Use this tool to maintain organized memory hierarchies rather than creating isolated memories for related information. This helps preserve conversation threads and project continuity across sessions."
    // // )]
    // // async fn extend_memory(
    // //     &self,
    // //     Extension(parts): Extension<Parts>,
    // //     Parameters(ExtendMemoryRequest {
    // //         content,
    // //         parent_memory_id,
    // //     }): Parameters<ExtendMemoryRequest>,
    // // ) -> Result<CallToolResult, McpError> {
    // //     let parameters = generated::Memory {
    // //         content,
    // //         user_id: extract_user_id(parts),
    // //         parent_memory_id,
    // //         ..Default::default()
    // //     };
    // //     MemoryController::add_memory(parameters).await.unwrap();
    // //     Ok(CallToolResult::success(vec![]))
    // // }
}

#[tool_handler]
impl rmcp::ServerHandler for McpService {
    fn get_info(&self) -> ServerInfo {
        debug!("McpService::get_info called");
        let tools = self.tool_router.list_all();
        debug!(
            "Available tools in get_info: {:?}",
            tools.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
        ServerInfo {
            instructions: Some("An external Memory Persistence Layer for LLM and AI Agents".into()),
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
