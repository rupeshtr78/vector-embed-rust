#[cfg(test)]
mod test_fetch_embedding {
    use crate::app::constants::EMBEDDING_MODEL;
    use crate::app::constants::EMBEDDING_URL;
    use crate::embedder::config::EmbedRequest;
    use crate::embedder::fetch_embedding;
    use anyhow::Context;
    use hyper::Client;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_fetch_embedding_success() {
        // Setup
        let url = EMBEDDING_URL; // ollama service is running on this URL
        let embed_data = Arc::new(RwLock::new(EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: vec!["test_input".to_string()],
            metadata: None,
            chunk_number: None,
        }));
        let http_client = Client::new();

        // Run the function
        let result = fetch_embedding(url, &embed_data, &http_client)
            .await
            .context("Failed to fetch embedding")
            .unwrap();

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
            chunk_number: None,
        }));
        let http_client = Client::new();

        // Run the function
        let result = fetch_embedding(url, &embed_data, &http_client)
            .await
            .context("Failed to fetch embedding")
            .unwrap();

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
            chunk_number: None,
        }));
        let http_client = Client::new();

        // Run the function
        let result = fetch_embedding(url, &embed_data, &http_client)
            .await
            .context("Failed to fetch embedding")
            .unwrap();

        // Assertions
        assert!(!result.model.is_empty(), "Model should not be empty");
        assert!(
            result.embeddings.is_empty(),
            "Embeddings should be empty for empty input"
        );
    }
}
