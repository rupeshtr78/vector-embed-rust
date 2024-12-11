// add configs here

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct EmbedRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct EmbedResponse {
    pub model: String,
    pub embeddings: Vec<Vec<f32>>,
}

pub fn NewEmbedRequest(model: String, input: Vec<String>) -> EmbedRequest {
    EmbedRequest { model, input }
}

pub fn NewEmbedResponse(model: String, embeddings: Vec<Vec<f32>>) -> EmbedResponse {
    EmbedResponse { model, embeddings }
}

pub fn NewEmbedResponseFromJson(json: &str) -> Result<EmbedResponse, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn EmptyEmbedResponse() -> EmbedResponse {
    EmbedResponse {
        model: "".to_string(),
        embeddings: vec![],
    }
}

pub fn EmptyEmbedRequest() -> EmbedRequest {
    EmbedRequest {
        model: "".to_string(),
        input: vec![],
    }
}

impl EmbedRequest {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<EmbedRequest, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn add_input(&mut self, input: String) {
        self.input.push(input);
    }

    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    pub fn get_input(&self) -> Vec<String> {
        self.input.clone()
    }

    pub fn get_model(&self) -> String {
        self.model.clone()
    }
}

impl EmbedResponse {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<EmbedResponse, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn add_embedding(&mut self, embedding: Vec<f32>) {
        self.embeddings.push(embedding);
    }

    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }

    pub fn get_embeddings(&self) -> Vec<Vec<f32>> {
        self.embeddings.clone()
    }

    pub fn get_model(&self) -> String {
        self.model.clone()
    }
}

// constants
pub const EMBEDDING_URL: &str = "http://0.0.0.0:11434/api/embed";
pub const EMBEDDING_MODEL: &str = "nomic-embed-text";

pub const VECTOR_DB_HOST: &str = "10.0.0.213";
pub const VECTOR_DB_PORT: u16 = 5555;
pub const VECTOR_DB_USER: &str = "rupesh";
pub const VECTOR_DB_NAME: &str = "vectordb";
pub const VECTOR_DB_TABLE: &str = "from_rust";
pub const VECTOR_DB_DIM: i32 = 768;
pub struct VectorDbConfig<'a> {
    pub host: &'a str,
    pub port: u16,
    pub user: &'a str,
    pub dbname: &'a str,
    pub timeout: u64,
}

pub fn NewVectorDbConfig<'a>(
    host: &'a str,
    port: u16,
    user: &'a str,
    dbname: &'a str,
) -> VectorDbConfig<'a> {
    VectorDbConfig {
        host,
        port,
        user,
        dbname,
        timeout: 5,
    }
}
