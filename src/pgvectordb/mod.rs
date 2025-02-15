pub mod pg_vector;
pub mod query_vector;
pub mod run_embedding;

pub struct VectorDbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub dbname: String,
    pub timeout: u64,
}

impl VectorDbConfig {

    pub fn clone(&self) -> VectorDbConfig {
        VectorDbConfig {
            host: self.host.clone(),
            port: self.port,
            user: self.user.clone(),
            dbname: self.dbname.clone(),
            timeout: self.timeout,
        }
    }

    /// constructor
    pub fn NewVectorDbConfig(host: &str, port: u16, user: &str, dbname: &str) -> VectorDbConfig {
        VectorDbConfig {
            host: host.to_string(),
            port,
            user: user.to_string(),
            dbname: dbname.to_string(),
            timeout: 5,
        }
    }
}