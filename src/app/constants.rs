// constants
pub const EMBEDDING_URL: &str = "http://0.0.0.0:11434/api/embed";
pub const EMBEDDING_MODEL: &str = "nomic-embed-text";

pub const VECTOR_DB_HOST: &str = "10.0.0.213";
pub const VECTOR_DB_PORT: u16 = 5555;
pub const VECTOR_DB_USER: &str = "rupesh";
pub const VECTOR_DB_NAME: &str = "vectordb";
pub const VECTOR_DB_TABLE: &str = "from_rust";
pub const VECTOR_DB_DIM: i32 = 768;
pub const VECTOR_DB_DIM_STR: &str = "768";
pub const VERSION: &str = "1.0.0";

pub fn get_dimension(dim: String) -> i32 {
    match dim.parse::<i32>() {
        Ok(d) => d,
        Err(_) => VECTOR_DB_DIM,
    }
}
