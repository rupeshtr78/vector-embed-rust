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
