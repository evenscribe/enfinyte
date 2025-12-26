# Enfinyte

> **Accessible memory layer for all AI systems**

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/evenscribe/enfinyte)
[![Rust Build](https://github.com/evenscribe/enfinyte/actions/workflows/rust.yml/badge.svg)](https://github.com/evenscribe/enfinyte/actions/workflows/rust.yml)

Enfinyte is a high-performance memory persistence layer built in Rust for LLMs and AI agents. It provides semantic memory storage, retrieval, and search through MCP (Model Context Protocol) and gRPC interfaces — with AI-powered annotation that automatically classifies, tags, and scores memories.

## Key Features

- **Multi-tenant Memory** — Isolated memory spaces per user with OAuth authentication
- **Semantic Search** — Vector embeddings with Qdrant or pgvector backends
- **Dual Interfaces** — Native MCP support for LLMs + gRPC API for programmatic access
- **AI-Powered Annotation** — Auto-classification, tagging, certainty and salience scoring
- **Rich Memory Types** — Semantic, Episodic, Procedural, Instruction, Relational, Working, Prospective
- **Document Ingestion** — Extract and store content from PDFs and websites

## Quick Start

### Prerequisites

- Rust 1.70+
- Qdrant or PostgreSQL with pgvector
- Cloudflare Workers AI account (embeddings)
- WorkOS account (MCP authentication)

### Configuration

Create config file at `~/.config/enfinyte/enfinyte.toml`:

```toml
[vector_store.qdrant]
url = "http://localhost:6334"
key = ""
collection_name = "enfinyte_memories"
chunk_size = 512
embedding_model_dimensions = 1024

[embedder.cloudflare]
account_id = "your_account_id"
api_token = "your_api_token"
model = "bge-m3"

[language_model]
model_name = "gpt-4o-mini"

[language_model.provider.openai]
api_key = "your_openai_key"
base_url = "https://api.openai.com/v1"

[mcp]
server_addr = "0.0.0.0:3000"
remote_url = "https://your-domain.com"
jwks_url = "https://api.workos.com/.well-known/jwks.json"

[mcp.work_os]
client_id = "your_workos_client_id"
client_secret = "your_workos_client_secret"
authkit_url = "https://your-domain.workos.com"

[grpc]
server_addr = "0.0.0.0:5051"
```

### Run

```bash
# Start Qdrant
docker run -d -p 6333:6333 -p 6334:6334 qdrant/qdrant

# Run servers
cargo run --bin mcp   # MCP server (port 3000)
cargo run --bin grpc  # gRPC server (port 5051)
```

## Usage

### MCP Tools

| Tool | Description |
|------|-------------|
| `add_memory` | Store new memory content |
| `get_all_memory` | Retrieve all user memories |
| `get_memory_by_id` | Get specific memory by ID |
| `search` | Semantic search across memories |

### gRPC API

- `CreateMemory` / `DeleteMemory` — Manage memories
- `GetMemory` / `ListMemories` — Retrieve memories
- `SearchMemories` — Semantic search

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Entry Points                           │
│              mcp (port 3000)    grpc (port 5051)            │
└──────────────────────────┬──────────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────────┐
│                      Controller                             │
│         Memory operations, search, lifecycle mgmt           │
└──────────────────────────┬──────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────┐
│   AI Layer    │  │     Core      │  │ Vector Store  │
│  Annotations  │  │ Memory Types  │  │    Qdrant     │
│  Embeddings   │  │   Lifecycle   │  │   pgvector    │
└───────────────┘  └───────────────┘  └───────────────┘
```

## Development

```bash
cargo build         # Build
cargo test          # Run tests
cargo fmt           # Format code
cargo clippy        # Lint
```

## License

Apache License 2.0 — see [LICENSE](LICENSE) for details.

---

**Enfinyte** — Memory for the AI era.
