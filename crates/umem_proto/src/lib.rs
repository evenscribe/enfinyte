mod memory_v1;

use chrono::Utc;
use qdrant_client::Payload;
use rmcp::schemars;
use serde_json::json;
use uuid::Uuid;

impl From<generated::CreateMemoryRequest> for generated::Memory {
    fn from(request: generated::CreateMemoryRequest) -> Self {
        generated::Memory {
            id: Uuid::new_v4().to_string(),
            user_id: request.user_id,
            content: request.content,
            priority: request.priority,
            tags: request.tags,
            parent_id: request.parent_id,
            created_at: Utc::now().timestamp(),
        }
    }
}

impl From<generated::Memory> for Payload {
    fn from(value: generated::Memory) -> Self {
        Payload::try_from(json!(value)).expect("Couldn't serialize Memory.")
    }
}

pub mod generated {
    tonic::include_proto!("memory_v1");
}
