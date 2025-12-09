# Implement a backend for all of these

-- Add OR/AND 

- **Azure AI Search**
  - Website: https://learn.microsoft.com/en-us/azure/search/
  - GitHub / SDK: Azure SDK for Python (Search Documents) – https://github.com/Azure/azure-sdk-for-python/tree/main/sdk/search/azure-search-documents

- **Azure MySQL**
  - Website: https://azure.microsoft.com/en-us/services/mysql/
  - (Note: MySQL isn’t a vector-DB itself — this is likely how mem0 persists metadata or embeddings.)

- **Baidu Vector DB**
  - (LangChain community wrapper) – https://github.com/langchain-ai/langchain-community/tree/main/langchain_community/vectorstores/baiduvectordb
  - No official “Baidu vector DB” open-source repo; likely uses Baidu’s internal / proprietary vector service.

- **Cassandra**
  - Apache Cassandra website: https://cassandra.apache.org/
  - GitHub: https://github.com/apache/cassandra

- **Chroma (ChromaDB)**
  - Website: https://trychroma.com/ :contentReference[oaicite:0]{index=0}
  - GitHub: https://github.com/chroma-core/chroma :contentReference[oaicite:1]{index=1}

- **Databricks**
  - Website: https://www.databricks.com/
  - GitHub: (Databricks is a whole platform; no single “vector DB” repo — their GitHub: https://github.com/databricks)

- **Elastic / Elasticsearch**
  - Website: https://www.elastic.co/elasticsearch/
  - GitHub: https://github.com/elastic/elasticsearch

- **FAISS**
  - Website: https://faiss.ai/ :contentReference[oaicite:2]{index=2}
  - GitHub: https://github.com/facebookresearch/faiss :contentReference[oaicite:3]{index=3}

- **LangChain**
  - Website: https://langchain.com/
  - GitHub: https://github.com/langchain-ai/langchain

- **Milvus**
  - Website: https://milvus.io/
  - GitHub: https://github.com/milvus-io/milvus

- **MongoDB**
  - Website: https://www.mongodb.com/
  - GitHub: https://github.com/mongodb/mongo

- **Neptune**
  - (Assuming this means AWS Neptune) Website: https://aws.amazon.com/neptune/
  - GitHub: (AWS Neptune is a managed service — not a typical open-source DB; there’s no official “neptune vector-db” repo)

- **OpenSearch**
  - Website: https://opensearch.org/
  - GitHub: https://github.com/opensearch-project/OpenSearch


- **Pinecone**
  - Website: https://www.pinecone.io/
  - GitHub (client): https://github.com/pinecone-io/pinecone-python-client


- **Redis (Vector Search)**
  - Vector search docs: https://redis.io/solutions/vector-search/ :contentReference[oaicite:7]{index=7}
  - Redis vector-search docs / guide: https://redis.io/docs/latest/develop/ai/search-and-query/query/vector-search/ :contentReference[oaicite:8]{index=8}
  - GitHub: https://github.com/redis/redis (main Redis repo)

- **Supabase**
  - Website: https://supabase.com/
  - GitHub: https://github.com/supabase/supabase

- **Upstash Vector**
  - Website: https://upstash.com/
  - GitHub: https://github.com/upstash

- **Valkey**
  - I couldn't find a dedicated “Valkey vector-DB” open-source GitHub. (If you meant something else by “valkey”, I can dig in.)

- **Vertex AI Vector Search**
  - Website: https://cloud.google.com/vertex-ai
  - Google doesn’t open-source the managed Vertex AI vector search backend (so no official GitHub for that, just client SDKs in Google Cloud repos)

- **Weaviate**
  - Website: https://weaviate.io/
  - GitHub: https://github.com/weaviate/weaviate :contentReference[oaicite:9]{index=9}

## Completed

- **Qdrant**
  - Website: https://qdrant.tech/ :contentReference[oaicite:5]{index=5}
  - GitHub: https://github.com/qdrant/qdrant :contentReference[oaicite:6]{index=6}

- **PGVector (PostgreSQL extension)**
  - GitHub: https://github.com/pgvector/pgvector :contentReference[oaicite:4]{index=4}
