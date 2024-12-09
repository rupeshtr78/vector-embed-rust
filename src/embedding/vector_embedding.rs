use hyper::{Body, Request};
use hyper::{Client, Uri};
use log::{error, info};
use std::error::Error;

// #[tokio::main]
// async fn run_embed() {
//     colog::init();
//     info!("Starting");

//     // if let Err(e) = simple_get().await {
//     //     error!("Error: {}", e);
//     // }

//     // if let Err(e) = hyper_builder_get().await {
//     //     error!("Error: {}", e);
//     // }

//     let url = "http://0.0.0.0:11434/api/embed";
//     let model = "nomic-embed-text";
//     let data = EmbedRequest {
//         model: model.to_string(),
//         input: vec!["hello".to_string()],
//     };

//     if let Err(e) = hyper_builder_post(url, data).await {
//         error!("Error: {}", e);
//     }

//     info!("Done");
// }

#[derive(serde::Serialize)]
pub struct EmbedRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct EmbedResponse {
    pub model: String,
    pub embeddings: Vec<Vec<f32>>,
}

pub async fn create_embed_request(
    url: &str,
    req: EmbedRequest,
) -> Result<EmbedResponse, Box<dyn Error + Send + Sync>> {
    // Create an HTTP connector.
    let client = Client::new();
    // Construct a URI.
    let url = url.parse::<Uri>()?;

    // Serialize the data to a JSON string, handling potential errors
    let json_data = serde_json::to_string(&req)?;
    let data_body = Body::from(json_data);

    // Build the HTTP GET request using the http crate.
    let request = Request::builder()
        .method("POST")
        .uri(url)
        .header("Content-Type", "application/json")
        .body(data_body)?;

    // Send the request and await the response.
    let response_body = client.request(request).await?;

    info!("Response status: {}", response_body.status());
    // let body = hyper::body::to_bytes(response.into_body()).await?;

    let response_body = response_body.into_body();
    let body_bytes = hyper::body::to_bytes(response_body).await?;
    let body = std::str::from_utf8(&body_bytes)?;
    let response: EmbedResponse = serde_json::from_str(body)?;

    info!("Response: {:?}", response.model);
    info!("Response Length: {:?}", response.embeddings[0].len());

    Ok(response)
}
