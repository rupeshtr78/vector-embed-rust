// constants
pub const EMBEDDING_URL: &str = "http://10.0.0.213:11434/api/embed";
pub const EMBEDDING_MODEL: &str = "nomic-embed-text";
pub const VECTOR_DB_HOST: &str = "10.0.0.213";
pub const VECTOR_DB_PORT: u16 = 5555;
pub const VECTOR_DB_USER: &str = "rupesh";
pub const VECTOR_DB_NAME: &str = "vectordb";
pub const VECTOR_DB_TABLE: &str = "from_rust";
pub const VECTOR_DB_DIM_STR: &str = "768";
pub const VECTOR_DB_DIM_SIZE: i32 = 768;
pub const VERSION: &str = "1.0.0";
pub const QUERY_LIMIT: i64 = 1;
pub const LANCEDB_DISTANCE_FN: lancedb::DistanceType = lancedb::DistanceType::L2;
pub const CHAT_API_URL: &str = "http://10.0.0.213:11434";
pub const CHAT_API_KEY: &str = "api_key";
pub const CHAT_RESPONSE_FORMAT: &str = "json";
pub const SYSTEM_PROMPT_PATH: &str = "template/spark_prompt.txt"; ///"template/system_prompt.txt";
pub const AI_MODEL: &str = "mistral:latest";