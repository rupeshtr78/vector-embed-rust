use anyhow::Context;
use anyhow::Result;
use hyper::client::HttpConnector;
use hyper::Client as HttpClient;
use hyper::{body, Client, Uri};
use hyper::{Body, Request};
use log::{debug, info};
use std::str;
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod config;
use config::{EmbedRequest, EmbedResponse};

/// Fetch the embedding from the embedding service
/// Arguments:
/// - url: &str
/// - embed_data: &Arc<RwLock<EmbedRequest>>
/// Returns:
/// - EmbedResponse
pub async fn fetch_embedding(
    url: &str,
    embed_data: &Arc<RwLock<EmbedRequest>>,
    http_client: &HttpClient<HttpConnector>,
) -> Result<EmbedResponse> {
    debug!("Running Embedding");
    let embed_data = embed_data.read().await;

    let response = create_embed_request(url, &embed_data, http_client)
        .await
        .with_context(|| format!("Failed to fetch embedding from {}", url))?;

    debug!("Finished Running Embedding");
    Ok(response)
}

/// Create an embedding request
/// Arguments:
/// - url: &str
/// - req: &EmbedRequest
/// Returns:
/// - Result<EmbedResponse, Box<dyn Error + Send + Sync>>
pub async fn create_embed_request(
    url: &str,
    req: &EmbedRequest,
    http_client: &Client<HttpConnector>,
) -> Result<EmbedResponse> {
    debug!("Creating Embed Request");
    // Create an HTTP connector.
    // let client = Client::new();
    // Construct a URI.
    let url = url.parse::<Uri>()?;

    // Serialize the data to a JSON string, handling potential errors
    let json_data = req.to_json()?;
    let data_body = Body::from(json_data);

    // Build the HTTP GET request using the http crate.
    let request = Request::builder()
        .method("POST")
        .uri(&url)
        .header("Content-Type", "application/json")
        .body(data_body)
        .context("Failed to build request")?;

    // Send the request and await the response.
    let response_body = http_client
        .request(request)
        .await
        .with_context(|| format!("Failed to send request to {}", &url))?;

    info!("Embedding Response status: {}", response_body.status());
    // let body = hyper::body::to_bytes(response.into_body()).await?;

    let response_body = response_body.into_body();
    let body_bytes = body::to_bytes(response_body)
        .await
        .context("Failed to read response body")?;
    let body = str::from_utf8(&body_bytes)?;
    let response: EmbedResponse = serde_json::from_str(body).context("Failed to parse response")?;

    debug!("Response: {:?}", response.model);
    debug!("Response Length: {:?}", response.embeddings[0].len());

    Ok(response)
}
