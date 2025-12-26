## 1. Error Handling

### Rule 1.1: Use `thiserror` for Error Type Definitions

**What:** Define custom error types using the `thiserror` crate with the `#[derive(Error, Debug)]` macro.

**Why:** Provides consistent, ergonomic error types with automatic `Display` and `Error` trait implementations. Reduces boilerplate while maintaining type safety.

**Good:**
```rust
#[derive(Debug, Error, Clone)]
pub enum MemoryControllerError {
    #[error("memory context failed: {0}")]
    MemoryContextError(#[from] MemoryContextError),

    #[error("vector store action failed with: {0}")]
    VectorStoreError(#[from] VectorStoreError),
}
```

**Bad:**
```rust
pub enum MemoryControllerError {
    MemoryContextError(MemoryContextError),
    VectorStoreError(VectorStoreError),
}

impl std::fmt::Display for MemoryControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryContextError(e) => write!(f, "memory context failed: {}", e),
            Self::VectorStoreError(e) => write!(f, "vector store failed: {}", e),
        }
    }
}
```

---

### Rule 1.2: Use `#[from]` for Error Composition

**What:** Use `#[from]` attribute for automatic error conversion between error types in a hierarchy.

**Why:** Enables seamless use of the `?` operator across error boundaries without manual `map_err` calls.

**Good:**
```rust
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("invalid memory kind: {0}")]
    MemoryKindError(#[from] ParseMemoryKindError),

    #[error("invalid memory content: {0}")]
    ContentError(#[from] MemoryContentError),
}
```

**Bad:**
```rust
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("invalid memory kind: {0}")]
    MemoryKindError(ParseMemoryKindError),
}

// Forces manual conversion everywhere
let result = parse_kind(input).map_err(MemoryError::MemoryKindError)?;
```

---

### Rule 1.3: Define Module-Local Result Type Alias

**What:** Each module with a custom error type should define a local `Result<T>` type alias.

**Why:** Reduces verbosity in function signatures while maintaining clarity about which error type is used.

**Good:**
```rust
// At module level
type Result<T> = std::result::Result<T, MemoryControllerError>;

// In function signatures
pub async fn create(request: CreateMemoryRequest) -> Result<Memory> { ... }
pub async fn delete(id: String) -> Result<()> { ... }
```

**Bad:**
```rust
pub async fn create(request: CreateMemoryRequest) -> std::result::Result<Memory, MemoryControllerError> { ... }
pub async fn delete(id: String) -> Result<(), MemoryControllerError> { ... }
```

---

### Rule 1.4: Error Messages Should Be Lowercase

**What:** Error message strings in `#[error("...")]` should start with lowercase letters.

**Why:** Follows Rust conventions for error messages that may be composed into larger messages. Consistent with standard library error messages.

**Good:**
```rust
#[error("credence must be a finite number")]
NotFinite,

#[error("updated_at ({updated}) cannot be earlier than created_at ({created})")]
UpdatedBeforeCreated { created: i64, updated: i64 },
```

**Bad:**
```rust
#[error("Credence must be a finite number")]
NotFinite,

#[error("Updated_at cannot be earlier than created_at")]
UpdatedBeforeCreated { created: i64, updated: i64 },
```

---

### Rule 1.5: Use `#[error(transparent)]` for Wrapped Errors

**What:** Use `#[error(transparent)]` when an error variant simply wraps another error without adding context.

**Why:** Forwards the wrapped error's `Display` implementation directly, avoiding redundant prefixes.

**Good:**
```rust
#[derive(Error, Debug)]
pub enum ResponseGeneratorError {
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}
```

**Bad:**
```rust
#[derive(Error, Debug)]
pub enum ResponseGeneratorError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),  // Redundant "HTTP error:" prefix
}
```

---

## 2. Module Organization

### Rule 2.1: Re-export Public Types from `lib.rs` or `mod.rs`

**What:** Use glob or selective re-exports to expose public types from the module root.

**Why:** Provides clean public API surface while keeping internal organization flexible. Users import from the crate root, not internal modules.

**Good:**
```rust
// crates/umem_core/src/lib.rs
mod lifecycle_state;
mod memory_content;
mod memory_context;

pub use crate::{
    lifecycle_state::*, 
    memory_content::*, 
    memory_context::*,
};
```

**Bad:**
```rust
// Exposing internal module structure to users
pub mod lifecycle_state;
pub mod memory_content;
pub mod memory_context;

// Forces users to write:
use umem_core::lifecycle_state::LifecycleState;
```

---

### Rule 2.2: Use `pub(crate)` for Internal APIs

**What:** Use `pub(crate)` visibility for items that need to be shared across modules within a crate but shouldn't be part of the public API.

**Why:** Minimizes public API surface, making it clearer what is stable/supported and enabling internal refactoring.

**Good:**
```rust
// Internal utility module
pub(crate) mod utils;

// Internal method on public type
impl LLMProvider {
    pub(crate) async fn do_generate_text(&self, request: GenerateTextRequest) -> Result<...> {
        // ...
    }
}
```

**Bad:**
```rust
// Unnecessarily public
pub mod utils;

// All methods public even if internal
impl LLMProvider {
    pub async fn do_generate_text(&self, ...) -> Result<...> { ... }
}
```

---

### Rule 2.3: Submodules Should Use `mod.rs` Pattern

**What:** Directories with multiple related modules should have a `mod.rs` file that declares and re-exports them.

**Why:** Provides a single entry point for the module hierarchy and controls what's exposed publicly.

**Good:**
```
providers/
├── mod.rs              # Declares and re-exports all providers
├── openai.rs
├── anthropic.rs
└── azure_openai.rs
```
```rust
// providers/mod.rs
mod amazon_bedrock;
mod anthropic;
mod azure_openai;
mod openai;

pub use amazon_bedrock::AmazonBedrockProvider;
pub use anthropic::AnthropicProvider;
pub use openai::*;  // Main provider exports everything
```

**Bad:**
```rust
// No mod.rs, forcing parent to declare all submodules
// In lib.rs:
mod providers_amazon_bedrock;
mod providers_anthropic;
mod providers_openai;
```

---

## 3. Struct and Type Patterns

### Rule 3.1: Use TypedBuilder for Complex Structs

**What:** Use the `typed-builder` crate for structs with multiple optional fields or complex construction.

**Why:** Provides compile-time safety, fluent API, and eliminates runtime validation for required fields.

**Good:**
```rust
#[derive(TypedBuilder)]
pub struct Query {
    limit: u32,
    context: MemoryContext,
    #[builder(default = false)]
    include_archived: bool,
    #[builder(default, setter(strip_option))]
    vector: Option<Vec<f32>>,
}

// Usage - compile error if required fields missing
let query = Query::builder()
    .limit(100)
    .context(context)
    .vector(vec![1.0, 2.0])  // strip_option unwraps
    .build();
```

**Bad:**
```rust
pub struct Query {
    pub limit: u32,
    pub context: MemoryContext,
    pub include_archived: bool,
    pub vector: Option<Vec<f32>>,
}

// No compile-time safety, easy to forget fields
let query = Query {
    limit: 100,
    context,
    include_archived: false,  // Must remember default
    vector: Some(vec![1.0, 2.0]),
};
```

---

### Rule 3.2: Use Newtype Pattern for Constrained Values

**What:** Wrap primitive types in newtype structs when the value has domain constraints or special semantics.

**Why:** Enforces invariants at construction time, prevents mixing up similar primitive types, and provides type-safe APIs.

**Good:**
```rust
#[derive(Serialize, Deserialize, Default, Copy, Clone)]
pub struct Credence(f32);

impl Credence {
    pub fn new(value: f32) -> Result<Self, CredenceError> {
        if !value.is_finite() {
            return Err(CredenceError::NotFinite);
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(CredenceError::OutOfRange(value));
        }
        Ok(Self(value))
    }
    
    pub fn get(self) -> f32 {
        self.0
    }
}
```

**Bad:**
```rust
pub struct MemorySignals {
    pub certainty: f32,  // No guarantee it's valid
    pub salience: f32,   // Could be any value
}

// Validation scattered everywhere
fn process(signals: MemorySignals) {
    if signals.certainty < 0.0 || signals.certainty > 1.0 {
        // handle error
    }
}
```

---

### Rule 3.3: Derive Macros in Consistent Order

**What:** Order derive macros consistently: Debug first, then schema, Clone/Copy, Serialize/Deserialize, Default, PartialEq/Eq.

**Why:** Consistency makes code more scannable and reduces cognitive load during code review.

**Good:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum LifecycleState { ... }

#[derive(Debug, schemars::JsonSchema, Clone, Serialize, Deserialize)]
pub struct MemoryContent { ... }

#[derive(Debug, Error, Clone)]
pub enum CredenceError { ... }
```

**Bad:**
```rust
#[derive(Serialize, Clone, Debug, Deserialize)]  // Inconsistent ordering
pub enum LifecycleState { ... }

#[derive(Clone, Debug, Serialize)]  // Different order in same codebase
pub struct MemoryContent { ... }
```

---

### Rule 3.4: Use Tagged Enums for Serde Polymorphism

**What:** Use `#[serde(tag = "type")]` for enums that need to be serialized/deserialized polymorphically.

**Why:** Produces cleaner JSON with explicit type discriminators, making API responses self-describing.

**Good:**
```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum OutputItem {
    Message { id: String, content: Vec<Content> },
    FunctionCall { id: String, name: String, arguments: String },
    #[serde(other)]
    Unknown,  // Catch-all for forward compatibility
}
```

**Bad:**
```rust
#[derive(Debug, Serialize, Deserialize)]
enum OutputItem {
    Message { id: String, content: Vec<Content> },
    FunctionCall { id: String, name: String },
}
// Produces: {"Message": {"id": "..."}} instead of {"type": "message", "id": "..."}
```

---

## 4. Async Patterns

### Rule 4.1: Use `#[async_trait]` for Async Trait Methods

**What:** Use the `async-trait` crate when defining traits with async methods.

**Why:** Async methods in traits are not directly supported in stable Rust. This macro provides the standard workaround.

**Good:**
```rust
use async_trait::async_trait;

#[async_trait]
pub trait VectorStoreBase {
    async fn insert(&self, vectors: &[&[f32]], payloads: &[&Memory]) -> Result<()>;
    async fn search(&self, query: Query) -> Result<Vec<Memory>>;
}

#[async_trait]
impl VectorStoreBase for QdrantVectorStore {
    async fn insert(&self, vectors: &[&[f32]], payloads: &[&Memory]) -> Result<()> {
        // implementation
    }
}
```

**Bad:**
```rust
pub trait VectorStoreBase {
    fn insert(&self, vectors: &[&[f32]]) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}
// Manual boxing is verbose and error-prone
```

---

### Rule 4.2: Use `OnceCell` for Async Singletons

**What:** Use `tokio::sync::OnceCell` for lazily-initialized async singletons.

**Why:** Provides thread-safe, async-compatible lazy initialization that runs only once.

**Good:**
```rust
use tokio::sync::OnceCell;

static VECTOR_STORE: OnceCell<Arc<dyn VectorStoreBase + Send + Sync>> = OnceCell::const_new();

pub async fn get_store() -> Result<Arc<dyn VectorStoreBase + Send + Sync>> {
    VECTOR_STORE
        .get_or_try_init(|| async {
            // Expensive async initialization
            initialize_store().await
        })
        .await
        .cloned()
}
```

**Bad:**
```rust
lazy_static! {
    static ref STORE: Mutex<Option<Arc<dyn VectorStoreBase>>> = Mutex::new(None);
}

pub async fn get_store() -> Arc<dyn VectorStoreBase> {
    let mut guard = STORE.lock().unwrap();
    if guard.is_none() {
        *guard = Some(initialize_store().await);  // Can't await with mutex held!
    }
    guard.clone().unwrap()
}
```

---

### Rule 4.3: Use `backon` for Retry Logic

**What:** Use the `backon` crate for implementing retry logic with exponential backoff.

**Why:** Provides clean, composable retry semantics with customizable backoff strategies.

**Good:**
```rust
use backon::{ExponentialBuilder, Retryable};

generation
    .retry(
        ExponentialBuilder::default()
            .with_max_times(max_retries)
            .with_total_delay(Some(total_delay)),
    )
    .sleep(tokio::time::sleep)
    .when(is_retryable_error)
    .notify(|err, dur| {
        tracing::debug!("retrying {:?} after {:?}", err, dur);
    })
    .await
```

**Bad:**
```rust
let mut retries = 0;
loop {
    match do_operation().await {
        Ok(result) => return Ok(result),
        Err(e) if retries < max_retries && is_retryable(&e) => {
            retries += 1;
            tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(retries))).await;
        }
        Err(e) => return Err(e),
    }
}
```

---

## 5. Naming Conventions

### Rule 5.1: Use Descriptive Type Suffixes

**What:** Use consistent suffixes for type names: `*Error`, `*Builder`, `*Request`, `*Response`, `*Config`, `*Base`.

**Why:** Makes type purpose immediately clear and enables pattern matching during code navigation.

**Good:**
```rust
pub struct CreateMemoryRequest { ... }
pub struct CreateMemoryRequestError { ... }
pub struct OpenAIProviderBuilder { ... }
pub struct GenerateTextResponse { ... }
pub trait VectorStoreBase { ... }
pub struct AppConfig { ... }
```

**Bad:**
```rust
pub struct CreateMemory { ... }      // Is it a request? A memory? Unclear
pub struct OpenAIProviderSetup { ... }  // Inconsistent with other builders
pub struct TextResult { ... }        // Not obvious it's a response
```

---

### Rule 5.2: Method Naming Conventions

**What:** Follow these method naming patterns:
- `new()` for constructors
- Getters without `get_` prefix for simple field access: `context()`, `summary()`
- `get_*` for computed/complex accessors: `get_id()`, `get_certainty()`
- `is_*` or `has_*` for boolean queries: `is_active()`, `has_user()`
- `as_str()` for converting to string representation
- `for_*` for factory methods: `for_user()`, `for_agent()`

**Why:** Consistent with Rust ecosystem conventions and standard library patterns.

**Good:**
```rust
impl Memory {
    pub fn new(...) -> Self { ... }
    pub fn context(&self) -> &MemoryContext { &self.context }
    pub fn get_id(&self) -> &Uuid { &self.id }
    pub fn is_archived(&self) -> bool { matches!(self.lifecycle, LifecycleState::Archived) }
}

impl MemoryContext {
    pub fn for_user(user_id: String) -> Result<Self> { ... }
}

impl LifecycleState {
    pub fn as_str(&self) -> &'static str { ... }
}
```

**Bad:**
```rust
impl Memory {
    pub fn create(...) -> Self { ... }           // Should be new()
    pub fn get_context(&self) -> &MemoryContext { ... }  // Unnecessary get_ prefix
    pub fn archived(&self) -> bool { ... }       // Should be is_archived()
}
```

---

### Rule 5.3: Use `impl Into<T>` for Flexible Parameters

**What:** Accept `impl Into<String>` (or similar) for parameters that will be stored as owned types.

**Why:** Allows callers to pass `&str`, `String`, or other convertible types without explicit conversion.

**Good:**
```rust
pub fn new(summary: impl Into<String>, tags: Vec<String>) -> Result<Self> {
    let summary = summary.into();
    // ...
}

pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
    self.api_key = Some(api_key.into());
    self
}
```

**Bad:**
```rust
pub fn new(summary: String, tags: Vec<String>) -> Result<Self> { ... }

// Caller must write:
Content::new(name.to_string(), tags);  // Unnecessary .to_string()
```

---

## 6. Trait Implementations

### Rule 6.1: Implement `FromStr` for Parseable Types

**What:** Implement `FromStr` for types that can be parsed from strings.

**Why:** Enables use of `.parse()` and integrates with standard string parsing idioms.

**Good:**
```rust
impl FromStr for LifecycleState {
    type Err = ParseLifecycleStateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            _ => Err(ParseLifecycleStateError { input: s.to_string() }),
        }
    }
}

// Enables: "active".parse::<LifecycleState>()?
```

**Bad:**
```rust
impl LifecycleState {
    pub fn from_string(s: &str) -> Result<Self, ParseError> {
        // ...
    }
}
// Doesn't integrate with .parse()
```

---

### Rule 6.2: Implement `From` for Type Conversions

**What:** Implement `From<T>` for infallible conversions between types.

**Why:** Enables `.into()` ergonomics and integrates with the standard conversion ecosystem.

**Good:**
```rust
impl From<OpenAIProvider> for LLMProvider {
    fn from(provider: OpenAIProvider) -> Self {
        LLMProvider::OpenAI(provider)
    }
}

impl From<String> for UserModelMessage {
    fn from(value: String) -> Self {
        UserModelMessage::Text(value)
    }
}

// Enables: provider.into() or LLMProvider::from(provider)
```

**Bad:**
```rust
impl LLMProvider {
    pub fn from_openai(provider: OpenAIProvider) -> Self {
        LLMProvider::OpenAI(provider)
    }
}
// Doesn't integrate with .into()
```

---

### Rule 6.3: Delegate Default to Builder When Appropriate

**What:** When a type has a builder, implement `Default` by delegating to the builder's defaults.

**Why:** Ensures consistency between builder defaults and `Default::default()` behavior.

**Good:**
```rust
impl Default for ChunkerConfig {
    fn default() -> Self {
        ChunkerConfigBuilder::new().build().unwrap()
    }
}

impl Default for OpenAIProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

**Bad:**
```rust
impl Default for ChunkerConfig {
    fn default() -> Self {
        ChunkerConfig {
            chunk_size: 2000,  // Duplicated from builder
            chunk_overlap: 0,
            // ...
        }
    }
}
```

---

## 7. Builder Pattern

### Rule 7.1: Builder Methods Take `mut self` and Return `Self`

**What:** Builder methods should consume and return self for method chaining.

**Why:** Provides ergonomic fluent API that's idiomatic in Rust.

**Good:**
```rust
pub struct OpenAIProviderBuilder { ... }

impl OpenAIProviderBuilder {
    pub fn new() -> Self { ... }
    
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
    
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }
    
    pub fn build(self) -> Result<OpenAIProvider> { ... }
}

// Usage: Builder::new().api_key("...").base_url("...").build()
```

**Bad:**
```rust
impl OpenAIProviderBuilder {
    pub fn set_api_key(&mut self, key: String) {  // Takes &mut self
        self.api_key = Some(key);
    }
}

// Requires multiple statements:
let mut builder = Builder::new();
builder.set_api_key("...".to_string());
builder.build()
```

---

### Rule 7.2: Validate in `build()` Method

**What:** Perform all validation in the `build()` method, returning `Result<T>`.

**Why:** Defers validation to construction time, allowing flexible builder configuration while ensuring final validity.

**Good:**
```rust
pub fn build(self) -> Result<OpenAIProvider> {
    if self.api_key.is_none() {
        bail!("api_key is required");
    }
    if self.base_url.is_none() && self.resource_name.is_none() {
        bail!("Either base_url or resource_name must be provided");
    }
    
    Ok(OpenAIProvider {
        api_key: self.api_key.unwrap(),
        base_url: self.base_url.unwrap_or_else(|| "https://api.openai.com/v1".into()),
        // ...
    })
}
```

**Bad:**
```rust
pub fn api_key(mut self, key: String) -> Result<Self> {  // Validates too early
    if key.is_empty() {
        return Err(anyhow!("api_key cannot be empty"));
    }
    self.api_key = Some(key);
    Ok(self)
}
```

---

## 8. Service Implementation Patterns

### Rule 8.1: Use Unit Structs for Stateless Services

**What:** Use unit structs (zero-sized types) for services that don't hold state.

**Why:** Clearly communicates that the type is a namespace for methods, not a state container.

**Good:**
```rust
#[derive(Debug, Default)]
pub struct MemoryController;

impl MemoryController {
    pub async fn create(request: CreateMemoryRequest) -> Result<Memory> { ... }
    pub async fn get(id: String) -> Result<Memory> { ... }
    pub async fn delete(id: String) -> Result<()> { ... }
}

pub struct PdfLoader;

impl Loader for PdfLoader {
    async fn load_data(&self, source: &AddSource) -> Result<LoadDataResult> { ... }
}
```

**Bad:**
```rust
pub struct MemoryController {
    _phantom: PhantomData<()>,  // Unnecessary phantom data
}

impl MemoryController {
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}
```

---

### Rule 8.2: Map Errors at Service Boundaries

**What:** Convert internal errors to service-appropriate error types at API boundaries (gRPC, MCP, HTTP).

**Why:** Prevents internal implementation details from leaking through the API and provides consistent error handling.

**Good:**
```rust
// gRPC service
async fn create_memory(&self, request: Request<CreateMemoryRequest>) -> Result<Response<()>, Status> {
    MemoryController::create(...)
        .await
        .map_err(|e| Status::new(Code::Internal, e.to_string()))?;
    Ok(Response::new(()))
}

// MCP service
async fn add_memory(&self, ...) -> Result<CallToolResult, McpError> {
    MemoryController::create(...)
        .await
        .map_err(|e| McpError::new(ErrorCode::INTERNAL_ERROR, e.to_string(), None))?;
    Ok(CallToolResult::success(...))
}
```

**Bad:**
```rust
async fn create_memory(&self, request: Request<CreateMemoryRequest>) -> Result<Response<()>, Status> {
    MemoryController::create(...).await?;  // Wrong error type, won't compile
    Ok(Response::new(()))
}
```

---

## 9. Configuration Patterns

### Rule 9.1: Use `lazy_static` for Global Configuration

**What:** Load configuration once at startup using `lazy_static!`.

**Why:** Ensures configuration is loaded exactly once and is available throughout the application lifetime.

**Good:**
```rust
lazy_static! {
    pub static ref CONFIG: AppConfig = AppConfig::new();
}

// Usage throughout codebase
match CONFIG.vector_store.clone() {
    VectorStore::Qdrant(qdrant) => { ... }
    VectorStore::PgVector(pgvector) => { ... }
}
```

**Bad:**
```rust
// Loading config on every call
fn get_vector_store() -> VectorStore {
    let config = AppConfig::new();  // Parses file every time!
    config.vector_store
}
```

---

### Rule 9.2: Use Tagged Enums for Config Variants

**What:** Use serde-tagged enums for configuration that can have multiple implementations.

**Why:** Enables type-safe configuration selection with clear TOML/JSON representation.

**Good:**
```rust
#[derive(Debug, Deserialize, Clone)]
pub enum VectorStore {
    #[serde(rename = "qdrant")]
    Qdrant(QdrantConfig),
    #[serde(rename = "pgvector")]
    PgVector(PgVectorConfig),
}

// In config.toml:
// [vector_store.qdrant]
// url = "http://localhost:6334"
```

**Bad:**
```rust
#[derive(Debug, Deserialize)]
pub struct VectorStoreConfig {
    pub store_type: String,  // "qdrant" or "pgvector"
    pub qdrant_url: Option<String>,
    pub pgvector_connection: Option<String>,
}
// No compile-time guarantee of valid configuration
```

---

## 10. Import Organization

### Rule 10.1: Group and Order Imports

**What:** Organize imports in groups: std library, external crates (alphabetical), workspace crates.

**Why:** Makes dependencies clear and imports easy to scan.

**Good:**
```rust
use std::sync::Arc;

use chrono::Utc;
use thiserror::Error;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use umem_ai::{LLMProvider, OpenAIProviderBuilder};
use umem_core::{Memory, MemoryContext, Query};
```

**Bad:**
```rust
use umem_core::Memory;
use std::sync::Arc;
use uuid::Uuid;
use umem_ai::LLMProvider;
use thiserror::Error;
use chrono::Utc;
// Unorganized, hard to scan
```

---

## 11. Documentation and Comments

### Rule 11.1: Use Comments for "Why", Not "What"

**What:** Comments should explain rationale or non-obvious behavior, not describe what code does.

**Why:** Code is self-documenting for "what"; comments add value by explaining "why".

**Good:**
```rust
// NOTE: unreachable because validated before
unreachable!()

// TODO: remove this allow once the module is fully implemented
#![allow(dead_code)]

// Retry on serialization errors because the API occasionally returns
// malformed JSON during high load
ResponseGeneratorError::Serialization(_) => true,
```

**Bad:**
```rust
// Create a new vector store
let store = VectorStore::new();

// Check if the memory is archived
if memory.is_archived() {
    // Return early if archived
    return;
}
```

---

### Rule 11.2: Use Const Strings for Long Prompts/Templates

**What:** Store long string literals (prompts, templates) as `const` items.

**Why:** Improves code readability and makes the string content easily findable.

**Good:**
```rust
const ANNOTATION_PROMPT: &str = r#"
You are a memory annotation system. Your task is to analyze the given content
and extract structured information including:
- Summary: A concise description
- Tags: Relevant keywords
- Importance: A score from 0-1
"#;

let result = generate_with_prompt(ANNOTATION_PROMPT, content).await?;
```

**Bad:**
```rust
let result = generate_with_prompt(
    "You are a memory annotation system. Your task is to analyze the given content and extract structured information including: - Summary: A concise description - Tags: Relevant keywords - Importance: A score from 0-1",
    content
).await?;
```

---

## 12. Utility Patterns

### Rule 12.1: Use `lazy_static` for Compiled Regex

**What:** Store compiled regex patterns in `lazy_static!` blocks.

**Why:** Regex compilation is expensive; doing it once at program start improves runtime performance.

**Good:**
```rust
lazy_static! {
    static ref WHITESPACE_RE: Regex = Regex::new(r"\s+").unwrap();
    static ref CONSECUTIVE_RE: Regex = Regex::new(r"([^\w\s])\1*").unwrap();
}

pub fn clean_string(text: String) -> String {
    let cleaned = WHITESPACE_RE.replace_all(text.trim(), " ");
    CONSECUTIVE_RE.replace_all(&cleaned, "$1").into_owned()
}
```

**Bad:**
```rust
pub fn clean_string(text: String) -> String {
    let whitespace_re = Regex::new(r"\s+").unwrap();  // Compiled on every call!
    let consecutive_re = Regex::new(r"([^\w\s])\1*").unwrap();
    // ...
}
```

---

### Rule 12.2: Use `rustc_hash` for Performance-Critical HashMaps

**What:** Use `rustc_hash::FxHashMap` instead of `std::collections::HashMap` when hashing performance matters.

**Why:** FxHash is significantly faster for small keys (common in caches, lookups).

**Good:**
```rust
// In lib.rs
pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;
pub type HashSet<T> = rustc_hash::FxHashSet<T>;

// Usage
use crate::HashMap;
let mut cache: HashMap<String, Memory> = HashMap::default();
```

**Bad:**
```rust
use std::collections::HashMap;
// Using std HashMap for high-frequency operations where FxHash would be faster
```

---

## 13. Testing Patterns

### Rule 13.1: Use `#[tokio::test]` for Async Tests

**What:** Use `#[tokio::test]` with appropriate flavor for async test functions.

**Why:** Properly initializes the async runtime for tests.

**Good:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_generate_object() {
        let response = generate_object(request).await.unwrap();
        assert!(response.is_valid());
    }
}
```

**Bad:**
```rust
#[test]
fn test_generate_object() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // ...
    });
}
```

---

### Rule 13.2: Test Modules at End of File

**What:** Place `#[cfg(test)] mod tests` at the end of the source file.

**Why:** Keeps production code together and tests clearly separated.

**Good:**
```rust
// ... all production code ...

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() { ... }
}
```

**Bad:**
```rust
#[cfg(test)]
mod tests { ... }

pub struct MyType { ... }  // Production code after tests

impl MyType { ... }
```

---
