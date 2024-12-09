use hyper::body::HttpBody;
use hyper::{Body, Request};
use hyper::{Client, Uri};
use log::{error, info};
use std::error::Error;
use tokio::io::{stdout, AsyncWriteExt as _};

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

async fn simple_get() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let uri = "http://httpbin.org/ip".parse::<Uri>()?;

    let mut resp = client.get(uri).await?;

    println!("Response: {}", resp.status());

    while let Some(chunk) = resp.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
    }

    Ok(())
}

#[derive(serde::Serialize)]
pub struct EmbedRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct EmbedResponse {
    model: String,
    embeddings: Vec<Vec<f32>>,
}

pub async fn hyper_builder_post(
    url: &str,
    req: EmbedRequest,
) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    println!("Response status: {}", response_body.status());
    // let body = hyper::body::to_bytes(response.into_body()).await?;

    let response_body = response_body.into_body();
    let body_bytes = hyper::body::to_bytes(response_body).await?;
    let body = std::str::from_utf8(&body_bytes)?;
    let response: EmbedResponse = serde_json::from_str(body)?;

    println!("Response: {:?}", response.model);
    println!("Response: {:?}", response.embeddings);

    Ok(())
}

async fn hyper_builder_get() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create an HTTP connector.
    let client = Client::new();

    // Construct a URI.
    let url = "http://httpbin.org/ip".parse::<Uri>()?;

    // Build the HTTP GET request using the http crate.
    let request = Request::builder()
        .method("GET")
        .uri(url)
        .header("Content-Type", "application/json")
        .body(hyper::Body::empty())?;

    // Send the request and await the response.
    let mut response = client.request(request).await?;

    println!("Response status: {}", response.status());

    // Print out the response body.
    while let Some(chunk) = response.body_mut().data().await {
        stdout().write_all(&chunk?).await?;
    }

    Ok(())
}
