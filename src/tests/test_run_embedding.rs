#[cfg(test)]
mod test_fetch_embedding {
    use crate::app::config::{EmbedRequest, EmbedResponse};
    use crate::app::constants::EMBEDDING_MODEL;
    use crate::app::constants::EMBEDDING_URL;
    use crate::embedding::run_embedding::fetch_embedding;
    use hyper::client::HttpConnector;
    use hyper::Client;
    use log::{debug, error};
    use std::panic;
    use std::sync::Arc;
    use std::sync::RwLock;

    #[tokio::test]
    async fn test_fetch_embedding_success() {
        // Setup
        let url = EMBEDDING_URL; // ollama service is running on this URL
        let embed_data = Arc::new(RwLock::new(EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: vec!["test_input".to_string()],
            metadata: None,
        }));
        let http_client = Client::new();

        // Run the function
        let result = fetch_embedding(url, &embed_data, &http_client).await;

        // Assertions
        assert!(!result.model.is_empty(), "Model should not be empty");
        assert!(
            !result.embeddings.is_empty(),
            "Embeddings should not be empty"
        );
        assert_eq!(
            result.embeddings.len(),
            1,
            "Embeddings should contain one vector"
        );
    }

    #[tokio::test]
    async fn test_fetch_embedding_http_request_failure() {
        // Setup
        let url = "http://nonexistent.service/embed"; // Non-existent URL to simulate HTTP failure
        let embed_data = Arc::new(RwLock::new(EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: vec!["test_input".to_string()],
            metadata: None,
        }));
        let http_client = Client::new();

        // Run the function
        let result = fetch_embedding(url, &embed_data, &http_client).await;

        // Assertions
        assert!(
            result.model.is_empty(),
            "Model should be empty on HTTP failure"
        );
        assert!(
            result.embeddings.is_empty(),
            "Embeddings should be empty on HTTP failure"
        );
    }

    #[tokio::test]
    async fn test_fetch_embedding_empty_input() {
        // Setup
        let url = EMBEDDING_URL; // ollama service is running on this URL
        let embed_data = Arc::new(RwLock::new(EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: vec![], // Empty input
            metadata: None,
        }));
        let http_client = Client::new();

        // Run the function
        let result = fetch_embedding(url, &embed_data, &http_client).await;

        // Assertions
        assert!(!result.model.is_empty(), "Model should not be empty");
        assert!(
            result.embeddings.is_empty(),
            "Embeddings should be empty for empty input"
        );
    }
}
