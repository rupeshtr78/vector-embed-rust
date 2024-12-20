use hyper::client::HttpConnector;
use hyper::{body, Client, Uri};
use hyper::{Body, Request};

use log::{debug, info};
use std::error::Error;
use std::str;

use crate::app::config::{EmbedRequest, EmbedResponse};

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
) -> Result<EmbedResponse, Box<dyn Error + Send + Sync>> {
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
        .uri(url)
        .header("Content-Type", "application/json")
        .body(data_body)?;

    // Send the request and await the response.
    let response_body = http_client.request(request).await?;

    info!("Embedding Response status: {}", response_body.status());
    // let body = hyper::body::to_bytes(response.into_body()).await?;

    let response_body = response_body.into_body();
    let body_bytes = body::to_bytes(response_body).await?;
    let body = str::from_utf8(&body_bytes)?;
    let response: EmbedResponse = serde_json::from_str(body)?;

    debug!("Response: {:?}", response.model);
    debug!("Response Length: {:?}", response.embeddings[0].len());

    Ok(response)
}
