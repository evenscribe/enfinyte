use chrono::Utc;
use umem_proto::generated;
use uuid::Uuid;

#[derive(Default)]
pub struct CreateMemoryRequestBuilder {
    pub user_id: String,
    pub content: String,
    pub priority: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub parent_id: Option<String>,
}

impl CreateMemoryRequestBuilder {
    pub fn new(user_id: String, content: String) -> Self {
        Self {
            user_id,
            content,
            ..Default::default()
        }
    }

    pub fn priority(self, priority: Option<i32>) -> Self {
        Self { priority, ..self }
    }

    pub fn tags(self, tags: Option<Vec<String>>) -> Self {
        Self { tags, ..self }
    }

    pub fn parent_id(self, parent_id: Option<String>) -> Self {
        Self { parent_id, ..self }
    }

    pub fn build(self) -> generated::Memory {
        self.into()
    }
}

#[derive(Default)]
pub struct UpdateMemoryRequestBuilder {
    pub id: String,
    pub content: Option<String>,
    pub priority: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub parent_id: Option<String>,
}

impl UpdateMemoryRequestBuilder {
    pub fn new(id: String) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn content(self, content: Option<String>) -> Self {
        Self { content, ..self }
    }

    pub fn priority(self, priority: Option<i32>) -> Self {
        Self { priority, ..self }
    }

    pub fn tags(self, tags: Option<Vec<String>>) -> Self {
        Self { tags, ..self }
    }

    pub fn parent_id(self, parent_id: Option<String>) -> Self {
        Self { parent_id, ..self }
    }
}

impl From<CreateMemoryRequestBuilder> for generated::Memory {
    fn from(request: CreateMemoryRequestBuilder) -> Self {
        generated::Memory {
            id: Uuid::new_v4().to_string(),
            user_id: request.user_id,
            content: request.content,
            priority: request.priority,
            tags: request.tags.unwrap_or_default(),
            parent_id: request.parent_id,
            created_at: Utc::now().timestamp(),
        }
    }
}
