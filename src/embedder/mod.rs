use anyhow::Context;
use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use log::debug;
use std::str;
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod config;
use crate::lancevectordb::HttpsClient;
use config::{EmbedRequest, EmbedResponse};

/// Fetch the embedding from the embedding service
/// Arguments:
/// - url: &str
/// - embed_data: &Arc<RwLock<EmbedRequest>>
/// Returns:
/// - EmbedResponse
pub async fn fetch_embedding(
    embed_data: &Arc<RwLock<EmbedRequest>>,
    https_client: &HttpsClient,
) -> Result<EmbedResponse> {
    debug!("Running Embedding");
    let embed_data = embed_data.read().await;

    let embed_url = embed_data.get_embed_url();

    let response = create_embed_request(&embed_data, https_client)
        .await
        .with_context(|| format!("Failed to fetch embedding from api url {}", embed_url))?;

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
    req: &EmbedRequest,
    https_client: &HttpsClient,
) -> Result<EmbedResponse> {
    debug!("Creating Embed Request");
    // Get the embed URL and API key from the request
    let embed_url = req.get_embed_url();
    let api_key = req.get_api_key();

    // Serialize the data to a JSON string, handling potential errors
    let json_data = req.to_json()?;

    // Create a Full body from the JSON string
    let body = Full::new(Bytes::from(json_data));
    // let data_body = Body::from(json_data);

    // Build the HTTP GET request using the http crate.
    let request = http::Request::builder()
        .method("POST")
        .uri(&embed_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(body)
        .context("Failed to build request")?;

    // Send the request and await the response.
    let response_body = https_client
        .request(request)
        .await
        .with_context(|| format!("Failed to send request to {}", &embed_url))?;

    debug!("Embedding Response status: {}", response_body.status());

    //collecting body bytes in Hyper 1.0
    let body_bytes = http_body_util::BodyExt::collect(response_body.into_body())
        .await?
        .to_bytes();

    // // convert it to a string JSON
    // let body_str = String::from_utf8(body_bytes.to_vec())?;
    //
    // let response_body = response_body.into_body();
    // let body_bytes = body::to_bytes(response_body)
    //     .await
    //     .context("Failed to read response body")?;
    let body = str::from_utf8(&body_bytes)?;
    let response: EmbedResponse = serde_json::from_str(body).context("Failed to parse response")?;

    debug!("Response: {:?}", response.model);
    debug!("Response Length: {:?}", response.embeddings[0].len());

    Ok(response)
}
