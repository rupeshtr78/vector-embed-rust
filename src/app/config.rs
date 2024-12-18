// add configs here
use std::sync::RwLock;

#[derive(serde::Serialize, Debug, Clone)]

pub struct EmbedRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct EmbedResponse {
    pub model: String,
    pub embeddings: Vec<Vec<f32>>,
}

pub fn NewEmbedRequest(model: &str, input: Vec<&str>) -> EmbedRequest {
    let input: Vec<String> = input.iter().map(|s| s.to_string()).collect();
    let model = model.to_string();
    let data = EmbedRequest { model, input };

    data
}

/// Create a new EmbedRequest with Arc and RwLock for thread safety
pub fn ArcEmbedRequest(model: &str, input: Vec<&str>) -> std::sync::Arc<RwLock<EmbedRequest>> {
    let input: Vec<String> = input.iter().map(|s| s.to_string()).collect();
    let model = model.to_string();
    let data = EmbedRequest { model, input };

    std::sync::Arc::new(RwLock::new(data))
}

pub fn NewArcEmbedRequest(
    model: &String,
    input: Vec<&String>,
) -> std::sync::Arc<RwLock<EmbedRequest>> {
    let input: Vec<String> = input.iter().map(|s| s.to_string()).collect();
    let model = model.to_string();
    let data = EmbedRequest { model, input };

    std::sync::Arc::new(RwLock::new(data))
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

impl<'a> EmbedRequest {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn add_input(&mut self, input: &'a str) {
        self.input.push(input.to_string());
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

pub struct VectorDbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub dbname: String,
    pub timeout: u64,
}

pub fn NewVectorDbConfig(host: &str, port: u16, user: &str, dbname: &str) -> VectorDbConfig {
    VectorDbConfig {
        host: host.to_string(),
        port,
        user: user.to_string(),
        dbname: dbname.to_string(),
        timeout: 5,
    }
}

impl VectorDbConfig {
    pub fn to_string(&self) -> String {
        format!(
            "host={} port={} user={} dbname={}",
            self.host, self.port, self.user, self.dbname
        )
    }

    pub fn clone(&self) -> VectorDbConfig {
        VectorDbConfig {
            host: self.host.clone(),
            port: self.port,
            user: self.user.clone(),
            dbname: self.dbname.clone(),
            timeout: self.timeout,
        }
    }
}
