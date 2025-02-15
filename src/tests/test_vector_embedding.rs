#[cfg(test)]
mod tests {
    use crate::embedder::config::EmbedRequest;
    use crate::app::constants::{EMBEDDING_MODEL, EMBEDDING_URL};
    use crate::embedder::create_embed_request;
    
    use hyper::{Body, Client, Request, Response};
    use mockall::*;
    use std::error::Error;
    mock! {
        HttpClient {
            async fn request(&self, req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>>;
        }
    }

    #[tokio::test]
    async fn test_create_embed_request_success() {
        let mut http_client = Client::new();

        let req = EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: vec!["test_input".to_string()],
            metadata: None,
        };

        let result = create_embed_request(EMBEDDING_URL, &req, &http_client).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.model, EMBEDDING_MODEL);
        assert_eq!(response.embeddings.len(), 1);
    }

    #[tokio::test]
    async fn test_create_embed_request_invalid_url() {
        let mut http_client = Client::new();

        let req = EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: vec!["test_input".to_string()],
            metadata: None,
        };

        let result = create_embed_request("http://invalid/aapi/embed", &req, &http_client).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_embed_request_wrong_model() {
        let mut http_client = Client::new();

        let req = EmbedRequest {
            model: "WRONG_EMBEDDING_MODEL".to_string(),
            input: vec!["test_input".to_string()],
            metadata: None,
        };

        let result = create_embed_request(EMBEDDING_URL, &req, &http_client).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_embed_request_no_input() {
        let mut http_client = Client::new();

        let req = EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: vec![], // Empty input
            metadata: None,
        };

        let result = create_embed_request(EMBEDDING_URL, &req, &http_client).await;

        assert!(result.is_ok());
    }
}
