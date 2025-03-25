// constants
#[allow(dead_code)]
pub const EMBEDDING_URL: &str = "http://10.0.0.213:11434/api/embed"; // @TODO: Change this to the url plus the endpoint
pub const EMBEDDING_MODEL: &str = "nomic-embed-text";

// pub const VECTOR_DB_DIM_STR: &str = "768";
pub const VECTOR_DB_DIM_SIZE: i32 = 768;
pub const VERSION: &str = "1.0.0";
// pub const QUERY_LIMIT: i64 = 1;
pub const LANCEDB_DISTANCE_FN: lancedb::DistanceType = lancedb::DistanceType::L2;
pub const CHAT_API_URL: &str = "http://10.0.0.213:11434";
pub const CHAT_API_KEY: &str = "api_key";
pub const CHAT_RESPONSE_FORMAT: &str = "json";
pub const SYSTEM_PROMPT_PATH: &str = "src/resources/rag_prompt.txt";
pub const AI_MODEL: &str = "qwen2:7b"; //"mistral:latest";

pub const OLLAMA_CHAT_API: &str = "api/chat";
pub const OLLAMA_EMBED_API: &str = "api/embed";
pub const OPEN_AI_URL: &str = "https://api.openai.com";
pub const OPEN_AI_CHAT_API: &str = "v1/chat/completions";
pub const OPEN_AI_EMBED_API: &str = "v1/embeddings";

// pub const DEFAULT_CHUNK_SIZE: usize = 2048;
